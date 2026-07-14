// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Radiowave owns the policy boundary between HIP source and LLVM/AMDGPU.
//! It injects reviewed source-level lowering rules, invokes hipcc, inspects the
//! emitted code object, and records enough evidence to reproduce the build.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const HIP_SUPPORT_HEADER: &str = include_str!("../include/radiowave/hip.h");

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("source file does not exist: {0}")]
    MissingSource(PathBuf),
    #[error(
        "Radiowave's buffer-resource policy does not support architecture {0}; expected gfx11* or gfx12*"
    )]
    UnsupportedArchitecture(String),
    #[error("{tool} exited with {status}\nstdout:\n{stdout}\nstderr:\n{stderr}")]
    ToolFailed {
        tool: String,
        status: String,
        stdout: String,
        stderr: String,
    },
    #[error("no HIP offload bundle target containing {arch} was found in {input}")]
    MissingBundleTarget { arch: String, input: PathBuf },
    #[error("Radiowave code-object certification failed: {0}")]
    InvalidCertification(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Wavefront {
    Wave32,
    Wave64,
}

impl Wavefront {
    pub const fn width(self) -> u32 {
        match self {
            Self::Wave32 => 32,
            Self::Wave64 => 64,
        }
    }
}

/// AMDGPU machine-scheduler policies which Radiowave can reproduce and
/// correctness-gate per code object. These are deliberately explicit rather
/// than hidden in an arbitrary `-mllvm` string.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerProfile {
    #[default]
    Default,
    MaxIlp,
    IterativeIlp,
    MemoryClause,
    PipelineIlp,
}

impl SchedulerProfile {
    pub const ALL: [Self; 5] = [
        Self::Default,
        Self::MaxIlp,
        Self::IterativeIlp,
        Self::MemoryClause,
        Self::PipelineIlp,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::MaxIlp => "max_ilp",
            Self::IterativeIlp => "iterative_ilp",
            Self::MemoryClause => "memory_clause",
            Self::PipelineIlp => "pipeline_ilp",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "default" => Some(Self::Default),
            "max-ilp" | "max_ilp" => Some(Self::MaxIlp),
            "iterative-ilp" | "iterative_ilp" => Some(Self::IterativeIlp),
            "memory-clause" | "memory_clause" => Some(Self::MemoryClause),
            "pipeline-ilp" | "pipeline_ilp" => Some(Self::PipelineIlp),
            _ => None,
        }
    }

    const fn llvm_args(self) -> &'static [&'static str] {
        match self {
            Self::Default => &[],
            Self::MaxIlp => &["-mllvm", "-misched=gcn-max-ilp"],
            Self::IterativeIlp => &["-mllvm", "-misched=gcn-iterative-ilp"],
            Self::MemoryClause => &[
                "-mllvm",
                "-misched=gcn-max-memory-clause",
                "-mllvm",
                "-amdgpu-max-memory-clause=4",
            ],
            Self::PipelineIlp => &[
                "-mllvm",
                "-misched=gcn-max-ilp",
                "-mllvm",
                "-amdgpu-schedule-relaxed-occupancy",
                "-mllvm",
                "-amdgpu-igrouplp-exact-solver",
                "-mllvm",
                "-enable-pipeliner",
            ],
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompileRequest {
    pub source: PathBuf,
    pub output: PathBuf,
    pub arch: String,
    pub wavefront: Wavefront,
    pub hipcc: PathBuf,
    pub working_directory: Option<PathBuf>,
    pub optimization_level: u8,
    pub fast_math: bool,
    pub scheduler_profile: SchedulerProfile,
    pub defines: Vec<String>,
    pub extra_args: Vec<OsString>,
    pub manifest: Option<PathBuf>,
    pub inspect: bool,
}

impl CompileRequest {
    pub fn new(
        source: impl Into<PathBuf>,
        output: impl Into<PathBuf>,
        arch: impl Into<String>,
    ) -> Self {
        Self {
            source: source.into(),
            output: output.into(),
            arch: arch.into(),
            wavefront: Wavefront::Wave32,
            hipcc: env::var_os("HIPCC")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("hipcc")),
            working_directory: None,
            optimization_level: 3,
            fast_math: true,
            scheduler_profile: SchedulerProfile::Default,
            defines: Vec::new(),
            extra_args: Vec::new(),
            manifest: None,
            inspect: true,
        }
    }

    pub fn wavefront(mut self, wavefront: Wavefront) -> Self {
        self.wavefront = wavefront;
        self
    }

    pub fn hipcc(mut self, hipcc: impl Into<PathBuf>) -> Self {
        self.hipcc = hipcc.into();
        self
    }

    pub fn scheduler_profile(mut self, profile: SchedulerProfile) -> Self {
        self.scheduler_profile = profile;
        self
    }

    pub fn define(mut self, define: impl Into<String>) -> Self {
        self.defines.push(define.into());
        self
    }

    pub fn manifest(mut self, path: impl Into<PathBuf>) -> Self {
        self.manifest = Some(path.into());
        self
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct InstructionStats {
    pub static_instructions: u32,
    pub buffer_loads: u32,
    pub buffer_stores: u32,
    pub global_loads: u32,
    pub global_stores: u32,
    pub flat_loads: u32,
    pub flat_stores: u32,
    pub scalar_loads: u32,
    pub kernarg_scalar_loads: u32,
    pub scalar_buffer_loads: u32,
    pub unknown_scalar_loads: u32,
    pub wait_instructions: u32,
    pub delay_alu_instructions: u32,
    pub memory_clause_instructions: u32,
    pub max_consecutive_vmem_instructions: u32,
}

/// Shader read caches which may contain data read from a resource written by
/// an earlier dispatch.
///
/// This is deliberately fail-closed. Only code objects whose scalar loads are
/// proven to be kernarg-prologue loads are certified as `VmemOnly`; every
/// ambiguous scalar-memory access retains the scalar-cache invalidation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutableReadCache {
    #[default]
    ScalarOrUnknown,
    VmemOnly,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KernelReport {
    pub name: String,
    pub wavefront_size: u32,
    pub vgpr_count: u32,
    pub sgpr_count: u32,
    pub vgpr_spill_count: u32,
    pub sgpr_spill_count: u32,
    pub private_segment_fixed_size: u32,
    #[serde(default)]
    pub mutable_read_cache: MutableReadCache,
    pub instructions: InstructionStats,
}

impl KernelReport {
    pub fn has_spills(&self) -> bool {
        self.vgpr_spill_count != 0
            || self.sgpr_spill_count != 0
            || self.private_segment_fixed_size != 0
    }

    pub fn certifies_vmem_only_rmw(&self) -> bool {
        self.mutable_read_cache == MutableReadCache::VmemOnly
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CodeObjectInspection {
    pub bundle_target: String,
    pub kernels: Vec<KernelReport>,
}

impl CodeObjectInspection {
    pub fn kernel(&self, name: &str) -> Option<&KernelReport> {
        self.kernels.iter().find(|kernel| kernel.name == name)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompileManifest {
    pub schema_version: u32,
    pub compiler: String,
    pub generated_unix_seconds: u64,
    pub source: PathBuf,
    pub output: PathBuf,
    pub arch: String,
    pub wavefront: Wavefront,
    #[serde(default)]
    pub scheduler_profile: SchedulerProfile,
    pub hipcc: PathBuf,
    pub hipcc_version: String,
    pub command: Vec<String>,
    pub source_sha256: String,
    pub support_header_sha256: String,
    pub output_sha256: String,
    pub inspection: Option<CodeObjectInspection>,
}

/// A manifest whose compiler identity, schema, inspection payload, and
/// code-object hash have been verified. Runtime submission layers consume this
/// instead of trusting an adjacent JSON file by path.
#[derive(Clone, Debug)]
pub struct CodeObjectCertification {
    manifest: CompileManifest,
}

impl CodeObjectCertification {
    pub fn from_json(code_object: &[u8], encoded_manifest: &str) -> Result<Self> {
        let manifest: CompileManifest = serde_json::from_str(encoded_manifest)?;
        if manifest.compiler != "radiowave" {
            return Err(Error::InvalidCertification(format!(
                "compiler is {:?}, expected \"radiowave\"",
                manifest.compiler
            )));
        }
        if manifest.schema_version < 3 {
            return Err(Error::InvalidCertification(format!(
                "schema {} predates the current cache-footprint certification",
                manifest.schema_version
            )));
        }
        if manifest.inspection.is_none() {
            return Err(Error::InvalidCertification(
                "manifest contains no code-object inspection".to_owned(),
            ));
        }
        let actual_sha256 = sha256_bytes(code_object);
        if manifest.output_sha256 != actual_sha256 {
            return Err(Error::InvalidCertification(format!(
                "code-object SHA-256 mismatch: manifest {}, actual {}",
                manifest.output_sha256, actual_sha256
            )));
        }
        Ok(Self { manifest })
    }

    pub fn manifest(&self) -> &CompileManifest {
        &self.manifest
    }

    pub fn inspection(&self) -> &CodeObjectInspection {
        self.manifest
            .inspection
            .as_ref()
            .expect("certification construction requires inspection")
    }

    pub fn kernel(&self, symbol: &str) -> Option<&KernelReport> {
        let canonical = symbol.strip_suffix(".kd").unwrap_or(symbol);
        self.inspection()
            .kernel(symbol)
            .or_else(|| self.inspection().kernel(canonical))
    }

    pub fn mutable_read_cache(&self, symbol: &str) -> MutableReadCache {
        self.kernel(symbol)
            .map_or(MutableReadCache::ScalarOrUnknown, |kernel| {
                kernel.mutable_read_cache
            })
    }
}

#[derive(Clone, Debug)]
pub struct CompileArtifact {
    pub output: PathBuf,
    pub output_sha256: String,
    pub manifest: Option<PathBuf>,
    pub inspection: Option<CodeObjectInspection>,
}

#[derive(Clone, Debug, Default)]
pub struct Compiler;

impl Compiler {
    pub fn compile(&self, request: &CompileRequest) -> Result<CompileArtifact> {
        if !supports_buffer_policy(&request.arch) {
            return Err(Error::UnsupportedArchitecture(request.arch.clone()));
        }
        let cwd = request
            .working_directory
            .clone()
            .unwrap_or(env::current_dir()?);
        let source = absolute_from(&cwd, &request.source);
        let output = absolute_from(&cwd, &request.output);
        if !source.is_file() {
            return Err(Error::MissingSource(source));
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }

        let header = support_header_path();
        let args = compile_args(request, &source, &output, &header);
        let mut command = Command::new(&request.hipcc);
        command.args(&args).current_dir(&cwd);
        checked_output(&mut command, "hipcc")?;

        let output_sha256 = sha256_file(&output)?;
        let inspection = if request.inspect {
            Some(Inspector::from_hipcc(&request.hipcc).inspect(&output, &request.arch)?)
        } else {
            None
        };

        let manifest_path = request
            .manifest
            .as_ref()
            .map(|path| absolute_from(&cwd, path));
        if let Some(path) = &manifest_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut rendered_command = Vec::with_capacity(args.len() + 1);
            rendered_command.push(request.hipcc.to_string_lossy().into_owned());
            rendered_command.extend(args.iter().map(|arg| arg.to_string_lossy().into_owned()));
            let manifest = CompileManifest {
                schema_version: 3,
                compiler: "radiowave".to_owned(),
                generated_unix_seconds: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                source: source.clone(),
                output: output.clone(),
                arch: request.arch.clone(),
                wavefront: request.wavefront,
                scheduler_profile: request.scheduler_profile,
                hipcc: request.hipcc.clone(),
                hipcc_version: tool_version(&request.hipcc),
                command: rendered_command,
                source_sha256: sha256_file(&source)?,
                support_header_sha256: sha256_bytes(HIP_SUPPORT_HEADER.as_bytes()),
                output_sha256: output_sha256.clone(),
                inspection: inspection.clone(),
            };
            fs::write(path, serde_json::to_vec_pretty(&manifest)?)?;
        }

        Ok(CompileArtifact {
            output,
            output_sha256,
            manifest: manifest_path,
            inspection,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Inspector {
    pub bundler: PathBuf,
    pub readobj: PathBuf,
    pub objdump: PathBuf,
}

impl Inspector {
    pub fn from_hipcc(hipcc: &Path) -> Self {
        let root = env::var_os("ROCM_PATH")
            .map(PathBuf::from)
            .or_else(|| {
                hipcc
                    .is_absolute()
                    .then(|| hipcc.parent()?.parent().map(Path::to_owned))
                    .flatten()
            })
            .unwrap_or_else(|| PathBuf::from("/opt/rocm"));
        Self {
            bundler: configured_tool("RADIOWAVE_BUNDLER", &root, "clang-offload-bundler"),
            readobj: configured_tool("RADIOWAVE_READOBJ", &root, "llvm-readobj"),
            objdump: configured_tool("RADIOWAVE_OBJDUMP", &root, "llvm-objdump"),
        }
    }

    pub fn inspect(&self, input: &Path, arch: &str) -> Result<CodeObjectInspection> {
        let target = self.bundle_target(input, arch)?;
        let extracted = env::temp_dir().join(format!(
            "radiowave-{}-{}.co",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));

        let mut unbundle = Command::new(&self.bundler);
        unbundle
            .arg("--type=o")
            .arg("--unbundle")
            .arg(format!("--input={}", input.display()))
            .arg(format!("--targets={target}"))
            .arg(format!("--output={}", extracted.display()));
        checked_output(&mut unbundle, "clang-offload-bundler")?;

        let result = (|| {
            let mut notes = Command::new(&self.readobj);
            notes.arg("--notes").arg(&extracted);
            let notes = checked_output(&mut notes, "llvm-readobj")?;

            let mut disassembly = Command::new(&self.objdump);
            disassembly
                .arg("--disassemble")
                .arg(format!("--mcpu={arch}"))
                .arg(&extracted);
            let disassembly = checked_output(&mut disassembly, "llvm-objdump")?;

            Ok(CodeObjectInspection {
                bundle_target: target,
                kernels: combine_reports(
                    &parse_metadata(&String::from_utf8_lossy(&notes.stdout)),
                    &parse_disassembly(&String::from_utf8_lossy(&disassembly.stdout)),
                ),
            })
        })();
        let _ = fs::remove_file(extracted);
        result
    }

    fn bundle_target(&self, input: &Path, arch: &str) -> Result<String> {
        let mut list = Command::new(&self.bundler);
        list.arg("--type=o")
            .arg("--list")
            .arg(format!("--input={}", input.display()));
        let output = checked_output(&mut list, "clang-offload-bundler")?;
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .find(|line| line.starts_with("hip") && line.contains(arch))
            .map(str::to_owned)
            .ok_or_else(|| Error::MissingBundleTarget {
                arch: arch.to_owned(),
                input: input.to_owned(),
            })
    }
}

pub fn support_header_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("include/radiowave/hip.h")
}

pub fn supports_buffer_policy(arch: &str) -> bool {
    arch.starts_with("gfx11") || arch.starts_with("gfx12")
}

fn compile_args(
    request: &CompileRequest,
    source: &Path,
    output: &Path,
    header: &Path,
) -> Vec<OsString> {
    let mut args = vec![
        "--genco".into(),
        format!("--offload-arch={}", request.arch).into(),
        format!("-O{}", request.optimization_level).into(),
    ];
    if request.fast_math {
        args.push("-ffast-math".into());
    }
    args.extend([
        "-fno-exceptions".into(),
        "-fno-rtti".into(),
        "-include".into(),
        header.as_os_str().to_owned(),
        "-DRADIOWAVE_ACTIVE=1".into(),
    ]);
    if request.wavefront == Wavefront::Wave64 {
        args.push("-mwavefrontsize64".into());
    }
    args.extend(
        request
            .scheduler_profile
            .llvm_args()
            .iter()
            .map(OsString::from),
    );
    args.extend(
        request
            .defines
            .iter()
            .map(|define| format!("-D{define}").into()),
    );
    args.extend(request.extra_args.iter().cloned());
    args.extend([
        "-o".into(),
        output.as_os_str().to_owned(),
        source.as_os_str().to_owned(),
    ]);
    args
}

fn absolute_from(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    }
}

fn configured_tool(variable: &str, root: &Path, name: &str) -> PathBuf {
    if let Some(path) = env::var_os(variable) {
        return path.into();
    }
    for candidate in [
        root.join("lib/llvm/bin").join(name),
        root.join("llvm/bin").join(name),
        root.join("bin").join(name),
    ] {
        if candidate.is_file() {
            return candidate;
        }
    }
    name.into()
}

fn checked_output(command: &mut Command, label: &str) -> Result<Output> {
    let output = command.output()?;
    if output.status.success() {
        return Ok(output);
    }
    Err(Error::ToolFailed {
        tool: label.to_owned(),
        status: output
            .status
            .code()
            .map_or_else(|| "signal".to_owned(), |code| code.to_string()),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn tool_version(tool: &Path) -> String {
    Command::new(tool)
        .arg("--version")
        .output()
        .ok()
        .map(|output| {
            let bytes = if output.stdout.is_empty() {
                output.stderr
            } else {
                output.stdout
            };
            String::from_utf8_lossy(&bytes).trim().to_owned()
        })
        .unwrap_or_default()
}

fn sha256_file(path: &Path) -> Result<String> {
    Ok(sha256_bytes(&fs::read(path)?))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn parse_metadata(notes: &str) -> Vec<KernelReport> {
    let mut reports = Vec::new();
    let mut current: Option<KernelReport> = None;
    for line in notes.lines() {
        if let Some(name) = line.strip_prefix("    .name:") {
            if let Some(report) = current.take() {
                reports.push(report);
            }
            current = Some(KernelReport {
                name: name.trim().trim_matches('"').to_owned(),
                ..KernelReport::default()
            });
            continue;
        }
        let Some(report) = current.as_mut() else {
            continue;
        };
        for (field, destination) in [
            ("    .wavefront_size:", &mut report.wavefront_size),
            ("    .vgpr_count:", &mut report.vgpr_count),
            ("    .sgpr_count:", &mut report.sgpr_count),
            ("    .vgpr_spill_count:", &mut report.vgpr_spill_count),
            ("    .sgpr_spill_count:", &mut report.sgpr_spill_count),
            (
                "    .private_segment_fixed_size:",
                &mut report.private_segment_fixed_size,
            ),
        ] {
            if let Some(value) = line.strip_prefix(field) {
                *destination = value.trim().parse().unwrap_or_default();
                break;
            }
        }
    }
    if let Some(report) = current {
        reports.push(report);
    }
    reports.retain(|report| !report.name.is_empty());
    reports
}

#[derive(Clone, Debug)]
struct DisassemblyKernel {
    instructions: InstructionStats,
    mutable_read_cache: MutableReadCache,
    kernarg_pointer_live: bool,
    consecutive_vmem_instructions: u32,
}

impl Default for DisassemblyKernel {
    fn default() -> Self {
        Self {
            instructions: InstructionStats::default(),
            mutable_read_cache: MutableReadCache::VmemOnly,
            kernarg_pointer_live: true,
            consecutive_vmem_instructions: 0,
        }
    }
}

fn parse_disassembly(disassembly: &str) -> BTreeMap<String, DisassemblyKernel> {
    let mut kernels = BTreeMap::<String, DisassemblyKernel>::new();
    let mut current = None::<String>;
    for line in disassembly.lines() {
        if let (Some(left), Some(right)) = (line.find('<'), line.find(">:")) {
            if left < right {
                let symbol = line[left + 1..right].to_owned();
                kernels.entry(symbol.clone()).or_default();
                current = Some(symbol);
                continue;
            }
        }
        let Some(symbol) = &current else {
            continue;
        };
        let instruction = line.split("//").next().unwrap_or_default().trim();
        let mnemonic = instruction.split_whitespace().next().unwrap_or_default();
        if mnemonic.is_empty() {
            continue;
        }
        let operands = instruction
            .strip_prefix(mnemonic)
            .unwrap_or_default()
            .trim();
        let entry = kernels.entry(symbol.clone()).or_default();
        let is_vmem = mnemonic.starts_with("buffer_")
            || mnemonic.starts_with("global_")
            || mnemonic.starts_with("flat_")
            || mnemonic.starts_with("scratch_")
            || mnemonic.starts_with("image_");
        if is_vmem {
            entry.consecutive_vmem_instructions += 1;
        } else if mnemonic != "s_clause" {
            entry.consecutive_vmem_instructions = 0;
        }
        let consecutive_vmem_instructions = entry.consecutive_vmem_instructions;
        let stats = &mut entry.instructions;
        if is_machine_instruction(mnemonic) {
            stats.static_instructions += 1;
        }
        stats.max_consecutive_vmem_instructions = stats
            .max_consecutive_vmem_instructions
            .max(consecutive_vmem_instructions);
        if mnemonic.starts_with("buffer_load") {
            stats.buffer_loads += 1;
        } else if mnemonic.starts_with("buffer_store") {
            stats.buffer_stores += 1;
        } else if mnemonic.starts_with("global_load") {
            stats.global_loads += 1;
        } else if mnemonic.starts_with("global_store") {
            stats.global_stores += 1;
        } else if mnemonic.starts_with("flat_load") {
            stats.flat_loads += 1;
        } else if mnemonic.starts_with("flat_store") {
            stats.flat_stores += 1;
        }

        if mnemonic.starts_with("s_buffer_load") {
            stats.scalar_loads += 1;
            stats.scalar_buffer_loads += 1;
            stats.unknown_scalar_loads += 1;
            entry.mutable_read_cache = MutableReadCache::ScalarOrUnknown;
        } else if mnemonic.starts_with("s_load") {
            stats.scalar_loads += 1;
            let base = operands.split(',').nth(1).map(str::trim);
            if entry.kernarg_pointer_live && base == Some("s[0:1]") {
                stats.kernarg_scalar_loads += 1;
            } else {
                stats.unknown_scalar_loads += 1;
                entry.mutable_read_cache = MutableReadCache::ScalarOrUnknown;
            }
        } else if mnemonic.starts_with("s_") && mnemonic.contains("_load_") {
            stats.scalar_loads += 1;
            stats.unknown_scalar_loads += 1;
            entry.mutable_read_cache = MutableReadCache::ScalarOrUnknown;
        }
        if mnemonic == "s_delay_alu" {
            stats.delay_alu_instructions += 1;
        }
        if mnemonic == "s_clause" {
            stats.memory_clause_instructions += 1;
        }
        if mnemonic.contains("wait") || mnemonic == "s_delay_alu" {
            stats.wait_instructions += 1;
        }

        let destination = operands
            .split(',')
            .next()
            .map(str::trim)
            .unwrap_or_default();
        if mnemonic.starts_with("s_") && touches_kernarg_pointer(destination) {
            entry.kernarg_pointer_live = false;
        }
    }
    kernels
}

fn is_machine_instruction(mnemonic: &str) -> bool {
    mnemonic.starts_with("s_")
        || mnemonic.starts_with("v_")
        || mnemonic.starts_with("buffer_")
        || mnemonic.starts_with("global_")
        || mnemonic.starts_with("flat_")
        || mnemonic.starts_with("scratch_")
        || mnemonic.starts_with("image_")
        || mnemonic.starts_with("ds_")
        || mnemonic.starts_with("exp")
}

fn touches_kernarg_pointer(operand: &str) -> bool {
    if operand == "s0" || operand == "s1" {
        return true;
    }
    let Some(range) = operand
        .strip_prefix("s[")
        .and_then(|value| value.strip_suffix(']'))
    else {
        return false;
    };
    let mut bounds = range
        .split(':')
        .filter_map(|value| value.parse::<u32>().ok());
    let Some(first) = bounds.next() else {
        return false;
    };
    let _last = bounds.next().unwrap_or(first);
    first <= 1
}

fn combine_reports(
    metadata: &[KernelReport],
    instructions: &BTreeMap<String, DisassemblyKernel>,
) -> Vec<KernelReport> {
    metadata
        .iter()
        .cloned()
        .map(|mut report| {
            if let Some(inspection) = instructions.get(&report.name) {
                report.instructions = inspection.instructions.clone();
                report.mutable_read_cache = inspection.mutable_read_cache;
            }
            report
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn certification_manifest(code_object: &[u8], cache: MutableReadCache) -> String {
        serde_json::to_string(&CompileManifest {
            schema_version: 3,
            compiler: "radiowave".to_owned(),
            generated_unix_seconds: 0,
            source: "kernel.hip".into(),
            output: "kernel.hsaco".into(),
            arch: "gfx1201".to_owned(),
            wavefront: Wavefront::Wave64,
            scheduler_profile: SchedulerProfile::Default,
            hipcc: "hipcc".into(),
            hipcc_version: "test".to_owned(),
            command: Vec::new(),
            source_sha256: "source".to_owned(),
            support_header_sha256: "header".to_owned(),
            output_sha256: sha256_bytes(code_object),
            inspection: Some(CodeObjectInspection {
                bundle_target: "hipv4-amdgcn-amd-amdhsa--gfx1201".to_owned(),
                kernels: vec![KernelReport {
                    name: "consumer".to_owned(),
                    wavefront_size: 64,
                    mutable_read_cache: cache,
                    ..KernelReport::default()
                }],
            }),
        })
        .unwrap()
    }

    #[test]
    fn parses_kernel_metadata_and_spills() {
        let notes = r#"
    .name:           memory_gather
    .private_segment_fixed_size: 0
    .sgpr_count:     21
    .sgpr_spill_count: 0
    .vgpr_count:     8
    .vgpr_spill_count: 0
    .wavefront_size: 32
      - .name:       hidden_arg
    .name:           spilled
    .private_segment_fixed_size: 16
    .vgpr_spill_count: 1
"#;
        let reports = parse_metadata(notes);
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].name, "memory_gather");
        assert_eq!(reports[0].vgpr_count, 8);
        assert!(!reports[0].has_spills());
        assert!(reports[1].has_spills());
    }

    #[test]
    fn counts_addressing_instruction_classes() {
        let disassembly = r#"
0000000000001000 <memory_gather>:
    s_load_b64 s[4:5], s[0:1], 0x0
    buffer_load_b32 v0, v1, s[0:3], null offen
    s_wait_loadcnt 0x0
    global_store_b32 v[0:1], v2, off
0000000000001100 <dot_q8>:
    global_load_b32 v2, v[0:1], off
    s_delay_alu instid0(VALU_DEP_1)
"#;
        let stats = parse_disassembly(disassembly);
        assert_eq!(stats["memory_gather"].instructions.static_instructions, 4);
        assert_eq!(stats["memory_gather"].instructions.buffer_loads, 1);
        assert_eq!(stats["memory_gather"].instructions.global_stores, 1);
        assert_eq!(stats["memory_gather"].instructions.scalar_loads, 1);
        assert_eq!(stats["memory_gather"].instructions.kernarg_scalar_loads, 1);
        assert_eq!(stats["memory_gather"].instructions.wait_instructions, 1);
        assert_eq!(
            stats["memory_gather"]
                .instructions
                .max_consecutive_vmem_instructions,
            1
        );
        assert_eq!(
            stats["memory_gather"].mutable_read_cache,
            MutableReadCache::VmemOnly
        );
        assert_eq!(stats["dot_q8"].instructions.global_loads, 1);
        assert_eq!(stats["dot_q8"].instructions.delay_alu_instructions, 1);
    }

    #[test]
    fn mutable_or_unknown_scalar_loads_fail_closed() {
        let disassembly = r#"
0000000000001000 <scalar_pointer>:
    s_load_b64 s[4:5], s[0:1], 0x10
    s_load_b32 s2, s[4:5], 0x0
0000000000001100 <scalar_buffer>:
    s_buffer_load_b32 s2, s[4:7], 0x0
"#;
        let stats = parse_disassembly(disassembly);
        for kernel in ["scalar_pointer", "scalar_buffer"] {
            assert_eq!(
                stats[kernel].mutable_read_cache,
                MutableReadCache::ScalarOrUnknown
            );
            assert_eq!(stats[kernel].instructions.unknown_scalar_loads, 1);
        }
        assert_eq!(stats["scalar_buffer"].instructions.scalar_buffer_loads, 1);
    }

    #[test]
    fn overwritten_kernarg_pair_cannot_certify_later_scalar_loads() {
        let disassembly = r#"
0000000000001000 <overwritten>:
    s_load_b64 s[0:1], s[0:1], 0x10
    s_load_b32 s2, s[0:1], 0x0
"#;
        let stats = parse_disassembly(disassembly);
        assert_eq!(stats["overwritten"].instructions.kernarg_scalar_loads, 1);
        assert_eq!(stats["overwritten"].instructions.unknown_scalar_loads, 1);
        assert_eq!(
            stats["overwritten"].mutable_read_cache,
            MutableReadCache::ScalarOrUnknown
        );
    }

    #[test]
    fn wavefront_width_is_manifest_stable() {
        assert_eq!(Wavefront::Wave32.width(), 32);
        assert_eq!(Wavefront::Wave64.width(), 64);
        assert_eq!(
            serde_json::to_string(&Wavefront::Wave64).unwrap(),
            "\"wave64\""
        );
    }

    #[test]
    fn scheduler_profiles_emit_reproducible_llvm_arguments() {
        let mut request = CompileRequest::new("kernel.hip", "kernel.hsaco", "gfx1201")
            .scheduler_profile(SchedulerProfile::PipelineIlp);
        request.extra_args.push("-mllvm".into());
        request.extra_args.push("-user-override".into());
        let args = compile_args(
            &request,
            Path::new("/tmp/kernel.hip"),
            Path::new("/tmp/kernel.hsaco"),
            Path::new("/tmp/radiowave.h"),
        );
        let args = args
            .iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>();
        assert!(args.iter().any(|arg| arg == "-misched=gcn-max-ilp"));
        assert!(
            args.iter()
                .any(|arg| arg == "-amdgpu-igrouplp-exact-solver")
        );
        let scheduler = args
            .iter()
            .position(|arg| arg == "-misched=gcn-max-ilp")
            .unwrap();
        let override_arg = args.iter().position(|arg| arg == "-user-override").unwrap();
        assert!(scheduler < override_arg);
    }

    #[test]
    fn scheduler_profile_names_are_manifest_stable() {
        assert_eq!(
            SchedulerProfile::parse("max-ilp"),
            Some(SchedulerProfile::MaxIlp)
        );
        assert_eq!(
            serde_json::to_string(&SchedulerProfile::MemoryClause).unwrap(),
            "\"memory_clause\""
        );
    }

    #[test]
    fn certification_binds_manifest_to_code_and_normalizes_kernel_symbols() {
        let code = b"test code object";
        let manifest = certification_manifest(code, MutableReadCache::VmemOnly);
        let certified = CodeObjectCertification::from_json(code, &manifest).unwrap();
        assert_eq!(
            certified.mutable_read_cache("consumer.kd"),
            MutableReadCache::VmemOnly
        );
        assert_eq!(
            certified.mutable_read_cache("missing.kd"),
            MutableReadCache::ScalarOrUnknown
        );
    }

    #[test]
    fn certification_rejects_stale_code_object() {
        let manifest = certification_manifest(b"original code object", MutableReadCache::VmemOnly);
        assert!(CodeObjectCertification::from_json(b"changed", &manifest).is_err());
    }

    #[test]
    fn certification_rejects_pre_cache_contract_schema() {
        let code = b"test code object";
        let mut manifest: serde_json::Value =
            serde_json::from_str(&certification_manifest(code, MutableReadCache::VmemOnly))
                .unwrap();
        manifest["schema_version"] = 2.into();
        assert!(CodeObjectCertification::from_json(code, &manifest.to_string()).is_err());
    }

    #[test]
    fn buffer_policy_is_fail_closed_by_architecture() {
        assert!(supports_buffer_policy("gfx1151"));
        assert!(supports_buffer_policy("gfx1201"));
        assert!(!supports_buffer_policy("gfx942"));
    }
}
