//! Code-object image sizing and AMDGPU HSA metadata kernarg layout.
//!
//! Parses little-endian ELF64 and clang offload bundles without external crates.
//! All raw-pointer reads live in documented `unsafe` blocks; malformed input
//! yields `None` or the documented pointer-field fallback — never panics.

use std::ffi::c_void;
use std::sync::Arc;

/// Maximum code-object image we will copy from a raw pointer (1 GiB).
const MAX_IMAGE_BYTES: usize = 1 << 30;
/// Maximum MessagePack nesting depth (maps/arrays).
const MSGPACK_MAX_DEPTH: u32 = 32;
/// Hard cap on MessagePack array/map entry counts.
const MSGPACK_MAX_ENTRIES: usize = 65_536;

const ELF_MAGIC: &[u8; 4] = b"\x7fELF";
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const ELF64_EHDR_SIZE: usize = 64;
const ELF64_SHDR_SIZE: usize = 64;
const SHT_NOTE: u32 = 7;
const NT_AMDGPU_METADATA: u32 = 32;
const CLANG_OFFLOAD_BUNDLE_MAGIC: &[u8] = b"__CLANG_OFFLOAD_BUNDLE__";
const CCOB_MAGIC: &[u8; 4] = b"CCOB";
/// Diagnostic revision for clang's classic uncompressed binary bundle.
///
/// This format has no serialized version word: the u64 immediately after the
/// magic is the entry count. CCOB, by contrast, serializes its version.
const CLANG_OFFLOAD_BUNDLE_VERSION: u64 = 2;

/// One kernarg field from AMDGPU metadata `.args` (or the sequential fallback).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArgField {
    pub offset: usize,
    pub size: usize,
    pub value_kind: String,
}

/// Resolved kernarg layout for packing `hipKernelNodeParams::kernelParams`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernargLayout {
    pub segment_size: usize,
    pub fields: Vec<ArgField>,
    /// `true` when fields came from an AMDGPU metadata note; `false` for fallback.
    pub from_metadata: bool,
    /// Loader symbol (prefer `.kd` form) for `Executable::kernel`.
    pub symbol: String,
}

/// Infer total image bytes from a raw little-endian ELF64 or clang offload
/// bundle header, then copy that many bytes into an owned buffer.
///
/// # Safety
///
/// `image` must be non-null and point at a readable header. If length inference
/// succeeds, the first `N` bytes at `image` must be a valid readable region of
/// size `N` (with `N <= 1 GiB`). Truncated or non-ELF/non-bundle headers return
/// `None` without reading past the header fields required for sizing.
pub(crate) unsafe fn copy_code_object_image(image: *const c_void) -> Option<Arc<[u8]>> {
    if image.is_null() {
        return None;
    }
    // SAFETY: caller guarantees `image` is readable for at least the probe header.
    let len = unsafe { infer_image_len(image)? };
    if len == 0 || len > MAX_IMAGE_BYTES {
        return None;
    }
    // SAFETY: caller guarantees the first `len` bytes at `image` are readable.
    let slice = unsafe { std::slice::from_raw_parts(image.cast::<u8>(), len) };
    Some(Arc::<[u8]>::from(slice.to_vec()))
}

/// Resolve kernarg field layout for `requested_name` from a code-object image.
///
/// Unwraps a clang offload bundle when present, finds ELF `NT_AMDGPU_METADATA`
/// (type 32) notes named `AMDGPU`/`AMDGPU\0`, parses MessagePack `amdhsa.kernels`,
/// and matches `.name` or `.symbol` (including bare / `.kd` variants).
///
/// When `loader_segment_size > 0` it is authoritative: metadata is accepted only
/// if every field end is within that size, and `segment_size` equals the loader
/// size. When the loader size is 0, `segment_size` may come from metadata
/// (symbol discovery). On any parse/match/bounds failure, returns sequential
/// pointer-sized `by_value` fields covering `loader_segment_size` with
/// `from_metadata = false` and a normalized loader symbol (`requested` if
/// already `.kd`, else `requested + ".kd"`).
pub(crate) fn kernarg_layout(
    code: &[u8],
    requested_name: &str,
    loader_segment_size: usize,
) -> KernargLayout {
    match try_kernarg_layout_from_metadata(code, requested_name, loader_segment_size) {
        Some(layout) => layout,
        None => fallback_layout(requested_name, loader_segment_size),
    }
}

fn fallback_layout(requested_name: &str, loader_segment_size: usize) -> KernargLayout {
    let pointer_size = std::mem::size_of::<usize>();
    let mut fields = Vec::new();
    let mut offset = 0usize;
    while offset < loader_segment_size {
        let remaining = loader_segment_size - offset;
        let size = remaining.min(pointer_size);
        fields.push(ArgField {
            offset,
            size,
            value_kind: "by_value".to_owned(),
        });
        offset = offset.saturating_add(size);
        if size == 0 {
            break;
        }
    }
    KernargLayout {
        segment_size: loader_segment_size,
        fields,
        from_metadata: false,
        symbol: normalize_loader_symbol(requested_name),
    }
}

fn normalize_loader_symbol(requested_name: &str) -> String {
    if requested_name.ends_with(".kd") {
        requested_name.to_owned()
    } else {
        format!("{requested_name}.kd")
    }
}

fn try_kernarg_layout_from_metadata(
    code: &[u8],
    requested_name: &str,
    loader_segment_size: usize,
) -> Option<KernargLayout> {
    let elf = unwrap_offload_bundle_for_parse(code)?;
    let notes = amdgpu_metadata_notes(elf)?;
    for note in notes {
        if let Some(layout) = layout_from_metadata_blob(note, requested_name, loader_segment_size) {
            return Some(layout);
        }
    }
    None
}

fn layout_from_metadata_blob(
    note: &[u8],
    requested_name: &str,
    loader_segment_size: usize,
) -> Option<KernargLayout> {
    let root = msgpack_parse(note)?;
    let map = root.as_map()?;
    let kernels = map_get(map, "amdhsa.kernels")?.as_array()?;
    let kernel = kernels
        .iter()
        .find_map(|entry| match_kernel(entry.as_map()?, requested_name))?;

    let symbol = map_get(kernel, ".symbol")
        .and_then(MsgValue::as_str)
        .map(str::to_owned)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| normalize_loader_symbol(requested_name));

    let args = map_get(kernel, ".args")
        .and_then(MsgValue::as_array)
        .unwrap_or(&[]);

    let mut fields = Vec::with_capacity(args.len());
    let mut max_end = 0usize;
    for arg in args {
        let arg_map = arg.as_map()?;
        let offset = map_get(arg_map, ".offset").and_then(MsgValue::as_usize)?;
        let size = map_get(arg_map, ".size").and_then(MsgValue::as_usize)?;
        let value_kind = map_get(arg_map, ".value_kind")
            .and_then(MsgValue::as_str)
            .unwrap_or("by_value")
            .to_owned();
        let end = offset.checked_add(size)?;
        // Loader size is authoritative when non-zero: reject fields past it.
        if loader_segment_size > 0 && end > loader_segment_size {
            return None;
        }
        max_end = max_end.max(end);
        fields.push(ArgField {
            offset,
            size,
            value_kind,
        });
    }

    let meta_segment = map_get(kernel, ".kernarg_segment_size")
        .and_then(MsgValue::as_usize)
        .unwrap_or(0);

    // loader > 0: segment_size is exactly the loader size.
    // loader == 0: allow metadata-derived size for symbol discovery.
    let segment_size = if loader_segment_size > 0 {
        loader_segment_size
    } else {
        meta_segment.max(max_end)
    };

    Some(KernargLayout {
        segment_size,
        fields,
        from_metadata: true,
        symbol,
    })
}

fn match_kernel<'a>(
    kernel: &'a [(String, MsgValue)],
    requested_name: &str,
) -> Option<&'a [(String, MsgValue)]> {
    let name = map_get(kernel, ".name")
        .and_then(MsgValue::as_str)
        .unwrap_or("");
    let symbol = map_get(kernel, ".symbol")
        .and_then(MsgValue::as_str)
        .unwrap_or("");
    if names_match(requested_name, name) || names_match(requested_name, symbol) {
        Some(kernel)
    } else {
        None
    }
}

fn names_match(requested: &str, candidate: &str) -> bool {
    if candidate.is_empty() {
        return false;
    }
    if requested == candidate {
        return true;
    }
    let req_base = requested.strip_suffix(".kd").unwrap_or(requested);
    let cand_base = candidate.strip_suffix(".kd").unwrap_or(candidate);
    req_base == cand_base
        || requested == format!("{cand_base}.kd")
        || candidate == format!("{req_base}.kd")
}

fn gfx_arch_prefix(name: &str) -> Option<&str> {
    let suffix = name.strip_prefix("gfx")?;
    let suffix_len = suffix
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_alphanumeric())
        .count();
    (suffix_len > 0).then(|| &name[..3 + suffix_len])
}

pub(crate) struct BundleDebugInfo<'a> {
    pub(crate) magic: &'static str,
    pub(crate) version: u64,
    pub(crate) entries: u64,
    pub(crate) selected: Option<&'a str>,
}

pub(crate) fn bundle_debug_info<'a>(
    bundle: &'a [u8],
    device_name: &str,
) -> Option<BundleDebugInfo<'a>> {
    if bundle.starts_with(CCOB_MAGIC) {
        return Some(BundleDebugInfo {
            magic: "ccob",
            version: u64::from(read_u16_at(bundle, CCOB_MAGIC.len())?),
            entries: 0,
            selected: None,
        });
    }
    if !bundle.starts_with(CLANG_OFFLOAD_BUNDLE_MAGIC) {
        return Some(BundleDebugInfo {
            magic: "other",
            version: 0,
            entries: 0,
            selected: None,
        });
    }

    let device_arch = gfx_arch_prefix(device_name);
    let selected_image = select_bundle_code_object(bundle, device_name);
    let mut cursor = CLANG_OFFLOAD_BUNDLE_MAGIC.len();
    let entries = read_u64(bundle, &mut cursor)?;
    if entries > 1024 {
        return None;
    }
    let mut selected = None;
    for _ in 0..entries {
        let _offset = read_u64(bundle, &mut cursor)?;
        let _size = read_u64(bundle, &mut cursor)?;
        let id_len = usize::try_from(read_u64(bundle, &mut cursor)?).ok()?;
        let id_end = cursor.checked_add(id_len)?;
        let id = std::str::from_utf8(bundle.get(cursor..id_end)?).ok()?;
        cursor = id_end;
        let Some((kind, target)) = id.rsplit_once("--") else {
            continue;
        };
        if selected_image.is_some() && kind.contains("amdgcn-amd-amdhsa") {
            let target_arch = gfx_arch_prefix(target);
            if target_arch == device_arch {
                selected = target_arch;
            }
        }
    }
    Some(BundleDebugInfo {
        magic: "clang",
        version: CLANG_OFFLOAD_BUNDLE_VERSION,
        entries,
        selected,
    })
}

/// Select the AMDGPU code object whose bundle target architecture matches the
/// device's leading `gfx...` architecture name. Target feature suffixes such as
/// `:xnack+` do not affect architecture matching.
pub(crate) fn select_bundle_code_object<'a>(
    bundle: &'a [u8],
    device_name: &str,
) -> Option<&'a [u8]> {
    if !bundle.starts_with(CLANG_OFFLOAD_BUNDLE_MAGIC) {
        return None;
    }
    let device_arch = gfx_arch_prefix(device_name)?;
    let mut cursor = CLANG_OFFLOAD_BUNDLE_MAGIC.len();
    let bundle_count = read_u64(bundle, &mut cursor)?;
    if bundle_count > 1024 {
        return None;
    }
    let mut selected = None;
    for _ in 0..bundle_count {
        let offset = usize::try_from(read_u64(bundle, &mut cursor)?).ok()?;
        let size = usize::try_from(read_u64(bundle, &mut cursor)?).ok()?;
        let id_len = usize::try_from(read_u64(bundle, &mut cursor)?).ok()?;
        let id_end = cursor.checked_add(id_len)?;
        let id = std::str::from_utf8(bundle.get(cursor..id_end)?).ok()?;
        cursor = id_end;
        let end = offset.checked_add(size)?;
        let payload = bundle.get(offset..end)?;
        let Some((kind, target)) = id.rsplit_once("--") else {
            continue;
        };
        if !kind.contains("amdgcn-amd-amdhsa") {
            continue;
        }
        let target_arch = gfx_arch_prefix(target);
        if target_arch == Some(device_arch) && selected.replace(payload).is_some() {
            return None;
        }
    }
    selected
}

/// Return the AMDGPU ELF bytes inside a clang offload bundle, or `code` itself
/// when it is already an ELF image. Malformed bundles yield `None`.
fn unwrap_offload_bundle_for_parse(code: &[u8]) -> Option<&[u8]> {
    if !code.starts_with(CLANG_OFFLOAD_BUNDLE_MAGIC) {
        if code.len() >= 4 && &code[..4] == ELF_MAGIC {
            return Some(code);
        }
        return None;
    }
    let mut cursor = CLANG_OFFLOAD_BUNDLE_MAGIC.len();
    let bundle_count = read_u64(code, &mut cursor)?;
    let mut amdgpu: Option<(usize, usize)> = None;
    for _ in 0..bundle_count {
        let offset = usize::try_from(read_u64(code, &mut cursor)?).ok()?;
        let size = usize::try_from(read_u64(code, &mut cursor)?).ok()?;
        let id_len = usize::try_from(read_u64(code, &mut cursor)?).ok()?;
        let id_end = cursor.checked_add(id_len)?;
        let id = code.get(cursor..id_end)?;
        cursor = id_end;
        let end = offset.checked_add(size)?;
        if end > code.len() {
            return None;
        }
        if id
            .windows(b"amdgcn-amd-amdhsa".len())
            .any(|window| window == b"amdgcn-amd-amdhsa")
            && amdgpu.replace((offset, end)).is_some()
        {
            // Multiple AMDGPU objects: refuse to guess.
            return None;
        }
    }
    let (start, end) = amdgpu?;
    code.get(start..end)
}

fn amdgpu_metadata_notes(elf: &[u8]) -> Option<Vec<&[u8]>> {
    if elf.len() < ELF64_EHDR_SIZE || elf.get(0..4)? != ELF_MAGIC {
        return None;
    }
    if *elf.get(4)? != ELFCLASS64 || *elf.get(5)? != ELFDATA2LSB {
        return None;
    }
    let shoff = read_u64_at(elf, 40)? as usize;
    let shentsize = read_u16_at(elf, 58)? as usize;
    let shnum = read_u16_at(elf, 60)? as usize;
    if shentsize < ELF64_SHDR_SIZE || shnum == 0 {
        return None;
    }
    let mut notes = Vec::new();
    for index in 0..shnum {
        let base = shoff.checked_add(index.checked_mul(shentsize)?)?;
        let sh_type = read_u32_at(elf, base.checked_add(4)?)?;
        if sh_type != SHT_NOTE {
            continue;
        }
        let sh_offset = read_u64_at(elf, base.checked_add(24)?)? as usize;
        let sh_size = read_u64_at(elf, base.checked_add(32)?)? as usize;
        let section_end = sh_offset.checked_add(sh_size)?;
        let section = elf.get(sh_offset..section_end)?;
        let mut off = 0usize;
        while off.checked_add(12)? <= section.len() {
            let namesz = read_u32_at(section, off)? as usize;
            let descsz = read_u32_at(section, off + 4)? as usize;
            let ntype = read_u32_at(section, off + 8)?;
            let name_start = off.checked_add(12)?;
            let name_end = name_start.checked_add(namesz)?;
            let name_padded = align4(name_end)?;
            let desc_end = name_padded.checked_add(descsz)?;
            let next = align4(desc_end)?;
            if next > section.len() {
                break;
            }
            if ntype == NT_AMDGPU_METADATA {
                let name = section.get(name_start..name_end)?;
                if is_amdgpu_note_name(name) {
                    if let Some(desc) = section.get(name_padded..desc_end) {
                        notes.push(desc);
                    }
                }
            }
            off = next;
        }
    }
    if notes.is_empty() { None } else { Some(notes) }
}

fn is_amdgpu_note_name(name: &[u8]) -> bool {
    name == b"AMDGPU" || name == b"AMDGPU\0"
}

/// Probe image length from a raw pointer. Only the header fields needed for
/// sizing are read.
///
/// # Safety
///
/// `image` must be non-null and readable for the magic / ELF / bundle header
/// extents touched below (at most a few hundred bytes for a well-formed header;
/// truncated inputs return `None` as soon as a bounds check fails).
unsafe fn infer_image_len(image: *const c_void) -> Option<usize> {
    // SAFETY: read a small fixed probe window to classify the image.
    let probe = unsafe { std::slice::from_raw_parts(image.cast::<u8>(), 64) };
    if probe.starts_with(CLANG_OFFLOAD_BUNDLE_MAGIC) {
        return unsafe { infer_bundle_len(image) };
    }
    if probe.len() >= 4 && &probe[..4] == ELF_MAGIC {
        return unsafe { infer_elf64_len(image) };
    }
    None
}

/// # Safety
/// `image` points at a clang offload bundle whose magic has already been observed.
unsafe fn infer_bundle_len(image: *const c_void) -> Option<usize> {
    // Read magic + count, then walk entry headers. We grow a temporary view.
    let magic_len = CLANG_OFFLOAD_BUNDLE_MAGIC.len();
    // SAFETY: magic already matched; 8-byte count follows.
    let hdr = unsafe { std::slice::from_raw_parts(image.cast::<u8>(), magic_len.checked_add(8)?) };
    let mut cursor = magic_len;
    let bundle_count = read_u64(hdr, &mut cursor)?;
    if bundle_count > 1024 {
        return None;
    }

    // First pass: read all TOC entries by extending the view as needed.
    // TOC layout after magic: u64 count, then per entry (offset, size, id_len, id bytes).
    // We do not know TOC size a priori; walk carefully with checked growth.
    let mut need = magic_len.checked_add(8)?;
    let mut entries: Vec<(u64, u64, u64)> = Vec::new();
    let mut id_cursor = need;
    for _ in 0..bundle_count {
        // Ensure room for three u64s.
        need = id_cursor.checked_add(24)?;
        if need > MAX_IMAGE_BYTES {
            return None;
        }
        // SAFETY: caller guarantees header region is readable; we stop at MAX.
        let view = unsafe { std::slice::from_raw_parts(image.cast::<u8>(), need) };
        let mut c = id_cursor;
        let offset = read_u64(view, &mut c)?;
        let size = read_u64(view, &mut c)?;
        let id_len = read_u64(view, &mut c)?;
        if id_len > MAX_IMAGE_BYTES as u64 {
            return None;
        }
        let id_end = c.checked_add(id_len as usize)?;
        if id_end > MAX_IMAGE_BYTES {
            return None;
        }
        entries.push((offset, size, id_len));
        id_cursor = id_end;
        need = id_end;
    }

    let mut total = need;
    for (offset, size, _) in entries {
        let end = (offset as u128).checked_add(size as u128)?;
        if end > MAX_IMAGE_BYTES as u128 {
            return None;
        }
        total = total.max(end as usize);
    }
    if total == 0 || total > MAX_IMAGE_BYTES {
        return None;
    }
    Some(total)
}

/// # Safety
/// `image` points at a little-endian ELF64 whose magic has already been observed.
unsafe fn infer_elf64_len(image: *const c_void) -> Option<usize> {
    // SAFETY: ELF64 header is 64 bytes; magic already matched.
    let ehdr = unsafe { std::slice::from_raw_parts(image.cast::<u8>(), ELF64_EHDR_SIZE) };
    if *ehdr.get(4)? != ELFCLASS64 || *ehdr.get(5)? != ELFDATA2LSB {
        return None;
    }
    let phoff = read_u64_at(ehdr, 32)? as usize;
    let shoff = read_u64_at(ehdr, 40)? as usize;
    let phentsize = read_u16_at(ehdr, 54)? as usize;
    let phnum = read_u16_at(ehdr, 56)? as usize;
    let shentsize = read_u16_at(ehdr, 58)? as usize;
    let shnum = read_u16_at(ehdr, 60)? as usize;

    let mut total = ELF64_EHDR_SIZE;

    if phnum > 0 {
        if phentsize < 56 {
            return None;
        }
        let ph_table_end = phoff.checked_add(phnum.checked_mul(phentsize)?)?;
        if ph_table_end > MAX_IMAGE_BYTES {
            return None;
        }
        total = total.max(ph_table_end);
        // SAFETY: program header table extent is within MAX_IMAGE_BYTES.
        let view = unsafe { std::slice::from_raw_parts(image.cast::<u8>(), ph_table_end) };
        for i in 0..phnum {
            let base = phoff.checked_add(i.checked_mul(phentsize)?)?;
            let p_offset = read_u64_at(view, base.checked_add(8)?)? as usize;
            let p_filesz = read_u64_at(view, base.checked_add(32)?)? as usize;
            let end = p_offset.checked_add(p_filesz)?;
            if end > MAX_IMAGE_BYTES {
                return None;
            }
            total = total.max(end);
        }
    }

    if shnum > 0 {
        if shentsize < ELF64_SHDR_SIZE {
            return None;
        }
        let sh_table_end = shoff.checked_add(shnum.checked_mul(shentsize)?)?;
        if sh_table_end > MAX_IMAGE_BYTES {
            return None;
        }
        total = total.max(sh_table_end);
        // SAFETY: section header table extent is within MAX_IMAGE_BYTES.
        let view = unsafe { std::slice::from_raw_parts(image.cast::<u8>(), sh_table_end) };
        for i in 0..shnum {
            let base = shoff.checked_add(i.checked_mul(shentsize)?)?;
            let sh_offset = read_u64_at(view, base.checked_add(24)?)? as usize;
            let sh_size = read_u64_at(view, base.checked_add(32)?)? as usize;
            // SHT_NOBITS (8) occupies no file bytes.
            let sh_type = read_u32_at(view, base.checked_add(4)?)?;
            if sh_type == 8 {
                continue;
            }
            let end = sh_offset.checked_add(sh_size)?;
            if end > MAX_IMAGE_BYTES {
                return None;
            }
            total = total.max(end);
        }
    }

    if total == 0 || total > MAX_IMAGE_BYTES {
        None
    } else {
        Some(total)
    }
}

fn align4(value: usize) -> Option<usize> {
    let rem = value % 4;
    if rem == 0 {
        Some(value)
    } else {
        value.checked_add(4 - rem)
    }
}

fn read_u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    let end = offset.checked_add(2)?;
    Some(u16::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

fn read_u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    Some(u32::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

fn read_u64_at(bytes: &[u8], offset: usize) -> Option<u64> {
    let end = offset.checked_add(8)?;
    Some(u64::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Option<u64> {
    let value = read_u64_at(bytes, *cursor)?;
    *cursor = cursor.checked_add(8)?;
    Some(value)
}

// ---------------------------------------------------------------------------
// Minimal MessagePack reader (maps, arrays, strings, integers, bool, nil, skip)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum MsgValue {
    Nil,
    Bool(bool),
    Int(i64),
    UInt(u64),
    F64(f64),
    Str(String),
    Bin(Vec<u8>),
    Array(Vec<MsgValue>),
    Map(Vec<(String, MsgValue)>),
}

impl MsgValue {
    fn as_map(&self) -> Option<&[(String, MsgValue)]> {
        match self {
            MsgValue::Map(m) => Some(m.as_slice()),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[MsgValue]> {
        match self {
            MsgValue::Array(a) => Some(a.as_slice()),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            MsgValue::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_usize(&self) -> Option<usize> {
        match *self {
            MsgValue::UInt(v) => usize::try_from(v).ok(),
            MsgValue::Int(v) if v >= 0 => usize::try_from(v).ok(),
            _ => None,
        }
    }
}

fn map_get<'a>(map: &'a [(String, MsgValue)], key: &str) -> Option<&'a MsgValue> {
    map.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn msgpack_parse(bytes: &[u8]) -> Option<MsgValue> {
    let mut cursor = 0usize;
    msgpack_read(bytes, &mut cursor, 0)
}

fn msgpack_read(bytes: &[u8], cursor: &mut usize, depth: u32) -> Option<MsgValue> {
    if depth > MSGPACK_MAX_DEPTH {
        return None;
    }
    let tag = *bytes.get(*cursor)?;
    *cursor = cursor.checked_add(1)?;

    // positive fixint
    if tag <= 0x7f {
        return Some(MsgValue::UInt(u64::from(tag)));
    }
    // fixmap
    if (0x80..=0x8f).contains(&tag) {
        let n = (tag & 0x0f) as usize;
        return msgpack_read_map(bytes, cursor, n, depth);
    }
    // fixarray
    if (0x90..=0x9f).contains(&tag) {
        let n = (tag & 0x0f) as usize;
        return msgpack_read_array(bytes, cursor, n, depth);
    }
    // fixstr
    if (0xa0..=0xbf).contains(&tag) {
        let n = (tag & 0x1f) as usize;
        return msgpack_read_str(bytes, cursor, n);
    }

    match tag {
        0xc0 => Some(MsgValue::Nil),
        0xc2 => Some(MsgValue::Bool(false)),
        0xc3 => Some(MsgValue::Bool(true)),
        0xc4 => {
            let n = *bytes.get(*cursor)? as usize;
            *cursor = cursor.checked_add(1)?;
            msgpack_read_bin(bytes, cursor, n)
        }
        0xc5 => {
            let n = read_u16_at(bytes, *cursor)? as usize;
            *cursor = cursor.checked_add(2)?;
            msgpack_read_bin(bytes, cursor, n)
        }
        0xc6 => {
            let n = read_u32_at(bytes, *cursor)? as usize;
            *cursor = cursor.checked_add(4)?;
            msgpack_read_bin(bytes, cursor, n)
        }
        0xca => {
            let end = cursor.checked_add(4)?;
            let raw: [u8; 4] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            // MessagePack float32 is big-endian.
            Some(MsgValue::F64(f64::from(f32::from_bits(
                u32::from_be_bytes(raw),
            ))))
        }
        0xcb => {
            let end = cursor.checked_add(8)?;
            let raw: [u8; 8] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            Some(MsgValue::F64(f64::from_bits(u64::from_be_bytes(raw))))
        }
        0xcc => {
            let v = *bytes.get(*cursor)?;
            *cursor = cursor.checked_add(1)?;
            Some(MsgValue::UInt(u64::from(v)))
        }
        0xcd => {
            let end = cursor.checked_add(2)?;
            let raw: [u8; 2] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            Some(MsgValue::UInt(u64::from(u16::from_be_bytes(raw))))
        }
        0xce => {
            let end = cursor.checked_add(4)?;
            let raw: [u8; 4] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            Some(MsgValue::UInt(u64::from(u32::from_be_bytes(raw))))
        }
        0xcf => {
            let end = cursor.checked_add(8)?;
            let raw: [u8; 8] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            Some(MsgValue::UInt(u64::from_be_bytes(raw)))
        }
        0xd0 => {
            let v = *bytes.get(*cursor)? as i8;
            *cursor = cursor.checked_add(1)?;
            Some(MsgValue::Int(i64::from(v)))
        }
        0xd1 => {
            let end = cursor.checked_add(2)?;
            let raw: [u8; 2] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            Some(MsgValue::Int(i64::from(i16::from_be_bytes(raw))))
        }
        0xd2 => {
            let end = cursor.checked_add(4)?;
            let raw: [u8; 4] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            Some(MsgValue::Int(i64::from(i32::from_be_bytes(raw))))
        }
        0xd3 => {
            let end = cursor.checked_add(8)?;
            let raw: [u8; 8] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            Some(MsgValue::Int(i64::from_be_bytes(raw)))
        }
        0xd9 => {
            let n = *bytes.get(*cursor)? as usize;
            *cursor = cursor.checked_add(1)?;
            msgpack_read_str(bytes, cursor, n)
        }
        0xda => {
            let end = cursor.checked_add(2)?;
            let raw: [u8; 2] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            msgpack_read_str(bytes, cursor, u16::from_be_bytes(raw) as usize)
        }
        0xdb => {
            let end = cursor.checked_add(4)?;
            let raw: [u8; 4] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            msgpack_read_str(bytes, cursor, u32::from_be_bytes(raw) as usize)
        }
        0xdc => {
            let end = cursor.checked_add(2)?;
            let raw: [u8; 2] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            msgpack_read_array(bytes, cursor, u16::from_be_bytes(raw) as usize, depth)
        }
        0xdd => {
            let end = cursor.checked_add(4)?;
            let raw: [u8; 4] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            msgpack_read_array(bytes, cursor, u32::from_be_bytes(raw) as usize, depth)
        }
        0xde => {
            let end = cursor.checked_add(2)?;
            let raw: [u8; 2] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            msgpack_read_map(bytes, cursor, u16::from_be_bytes(raw) as usize, depth)
        }
        0xdf => {
            let end = cursor.checked_add(4)?;
            let raw: [u8; 4] = bytes.get(*cursor..end)?.try_into().ok()?;
            *cursor = end;
            msgpack_read_map(bytes, cursor, u32::from_be_bytes(raw) as usize, depth)
        }
        // negative fixint
        t if t >= 0xe0 => Some(MsgValue::Int(i64::from(t as i8))),
        // ext / fixext — skip payload
        0xc7 => {
            let n = *bytes.get(*cursor)? as usize;
            *cursor = cursor.checked_add(1)?.checked_add(1)?.checked_add(n)?; // n + type
            if *cursor > bytes.len() {
                return None;
            }
            Some(MsgValue::Nil)
        }
        0xc8 => {
            let end = cursor.checked_add(2)?;
            let raw: [u8; 2] = bytes.get(*cursor..end)?.try_into().ok()?;
            let n = u16::from_be_bytes(raw) as usize;
            *cursor = end.checked_add(1)?.checked_add(n)?;
            if *cursor > bytes.len() {
                return None;
            }
            Some(MsgValue::Nil)
        }
        0xc9 => {
            let end = cursor.checked_add(4)?;
            let raw: [u8; 4] = bytes.get(*cursor..end)?.try_into().ok()?;
            let n = u32::from_be_bytes(raw) as usize;
            *cursor = end.checked_add(1)?.checked_add(n)?;
            if *cursor > bytes.len() {
                return None;
            }
            Some(MsgValue::Nil)
        }
        0xd4 => {
            *cursor = cursor.checked_add(2)?; // type + 1 byte
            Some(MsgValue::Nil)
        }
        0xd5 => {
            *cursor = cursor.checked_add(3)?;
            Some(MsgValue::Nil)
        }
        0xd6 => {
            *cursor = cursor.checked_add(5)?;
            Some(MsgValue::Nil)
        }
        0xd7 => {
            *cursor = cursor.checked_add(9)?;
            Some(MsgValue::Nil)
        }
        0xd8 => {
            *cursor = cursor.checked_add(17)?;
            Some(MsgValue::Nil)
        }
        _ => None,
    }
}

fn msgpack_read_str(bytes: &[u8], cursor: &mut usize, n: usize) -> Option<MsgValue> {
    let end = cursor.checked_add(n)?;
    let s = std::str::from_utf8(bytes.get(*cursor..end)?).ok()?;
    *cursor = end;
    Some(MsgValue::Str(s.to_owned()))
}

fn msgpack_read_bin(bytes: &[u8], cursor: &mut usize, n: usize) -> Option<MsgValue> {
    let end = cursor.checked_add(n)?;
    let b = bytes.get(*cursor..end)?.to_vec();
    *cursor = end;
    Some(MsgValue::Bin(b))
}

fn msgpack_entry_limit(bytes: &[u8], cursor: usize) -> usize {
    let remaining = bytes.len().saturating_sub(cursor);
    MSGPACK_MAX_ENTRIES.min(remaining.saturating_add(1))
}

fn msgpack_read_array(bytes: &[u8], cursor: &mut usize, n: usize, depth: u32) -> Option<MsgValue> {
    if depth >= MSGPACK_MAX_DEPTH {
        return None;
    }
    if n > msgpack_entry_limit(bytes, *cursor) {
        return None;
    }
    let child = depth + 1;
    let mut items = Vec::with_capacity(n.min(64));
    for _ in 0..n {
        items.push(msgpack_read(bytes, cursor, child)?);
    }
    Some(MsgValue::Array(items))
}

fn msgpack_read_map(bytes: &[u8], cursor: &mut usize, n: usize, depth: u32) -> Option<MsgValue> {
    if depth >= MSGPACK_MAX_DEPTH {
        return None;
    }
    if n > msgpack_entry_limit(bytes, *cursor) {
        return None;
    }
    let child = depth + 1;
    let mut items = Vec::with_capacity(n.min(64));
    for _ in 0..n {
        let key = match msgpack_read(bytes, cursor, child)? {
            MsgValue::Str(s) => s,
            // AMDGPU metadata keys are always strings; tolerate bin.
            MsgValue::Bin(b) => String::from_utf8(b).ok()?,
            _ => return None,
        };
        let value = msgpack_read(bytes, cursor, child)?;
        items.push((key, value));
    }
    Some(MsgValue::Map(items))
}

// ---------------------------------------------------------------------------
// Unit tests (synthetic ELF / msgpack; no GPU)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u32(buf: &mut Vec<u8>, v: u32) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    fn push_u64(buf: &mut Vec<u8>, v: u64) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Build a minimal LE ELF64 with one SHT_NOTE carrying AMDGPU metadata.
    fn synthetic_elf_with_note(msgpack: &[u8], note_name: &[u8]) -> Vec<u8> {
        let mut note = Vec::new();
        push_u32(&mut note, note_name.len() as u32);
        push_u32(&mut note, msgpack.len() as u32);
        push_u32(&mut note, NT_AMDGPU_METADATA);
        note.extend_from_slice(note_name);
        while note.len() % 4 != 0 {
            note.push(0);
        }
        note.extend_from_slice(msgpack);
        while note.len() % 4 != 0 {
            note.push(0);
        }

        let ehdr_size = 64usize;
        let shdr_size = 64usize;
        // Layout: ehdr | note | shdr[0 null] | shdr[1 note]
        let note_off = ehdr_size;
        let shoff = note_off + note.len();
        let total = shoff + shdr_size * 2;

        let mut elf = vec![0u8; total];
        elf[0..4].copy_from_slice(ELF_MAGIC);
        elf[4] = ELFCLASS64;
        elf[5] = ELFDATA2LSB;
        elf[6] = 1; // version
        // e_type ET_REL=1, e_machine EM_AMDGPU=224
        elf[16] = 1;
        elf[18] = 224;
        elf[20] = 1; // e_version
        // e_phoff=0, e_shoff
        elf[40..48].copy_from_slice(&(shoff as u64).to_le_bytes());
        elf[52] = 64; // e_ehsize
        // e_phentsize=0, e_phnum=0
        elf[58..60].copy_from_slice(&(shdr_size as u16).to_le_bytes());
        elf[60..62].copy_from_slice(&2u16.to_le_bytes()); // shnum
        elf[62..64].copy_from_slice(&0u16.to_le_bytes()); // shstrndx

        elf[note_off..note_off + note.len()].copy_from_slice(&note);

        // shdr[0] null — already zero
        // shdr[1] SHT_NOTE
        let s1 = shoff + shdr_size;
        elf[s1 + 4..s1 + 8].copy_from_slice(&SHT_NOTE.to_le_bytes());
        elf[s1 + 24..s1 + 32].copy_from_slice(&(note_off as u64).to_le_bytes());
        elf[s1 + 32..s1 + 40].copy_from_slice(&(note.len() as u64).to_le_bytes());

        elf
    }

    fn synthetic_elf_with_metadata(msgpack: &[u8]) -> Vec<u8> {
        synthetic_elf_with_note(msgpack, b"AMDGPU\0")
    }

    /// MessagePack for a single-kernel amdhsa metadata blob.
    fn synthetic_metadata_msgpack(
        name: &str,
        symbol: &str,
        kernarg_size: u64,
        args: &[(&str, u64, u64)],
    ) -> Vec<u8> {
        fn write_str(buf: &mut Vec<u8>, s: &str) {
            let b = s.as_bytes();
            if b.len() <= 31 {
                buf.push(0xa0 | (b.len() as u8));
            } else if b.len() <= 255 {
                buf.push(0xd9);
                buf.push(b.len() as u8);
            } else {
                buf.push(0xda);
                buf.extend_from_slice(&(b.len() as u16).to_be_bytes());
            }
            buf.extend_from_slice(b);
        }
        fn write_u64(buf: &mut Vec<u8>, v: u64) {
            if v <= 0x7f {
                buf.push(v as u8);
            } else if v <= 0xff {
                buf.push(0xcc);
                buf.push(v as u8);
            } else if v <= 0xffff {
                buf.push(0xcd);
                buf.extend_from_slice(&(v as u16).to_be_bytes());
            } else if v <= 0xffff_ffff {
                buf.push(0xce);
                buf.extend_from_slice(&(v as u32).to_be_bytes());
            } else {
                buf.push(0xcf);
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        fn write_map_header(buf: &mut Vec<u8>, n: usize) {
            if n <= 15 {
                buf.push(0x80 | (n as u8));
            } else {
                buf.push(0xde);
                buf.extend_from_slice(&(n as u16).to_be_bytes());
            }
        }
        fn write_array_header(buf: &mut Vec<u8>, n: usize) {
            if n <= 15 {
                buf.push(0x90 | (n as u8));
            } else {
                buf.push(0xdc);
                buf.extend_from_slice(&(n as u16).to_be_bytes());
            }
        }

        let mut arg_bytes = Vec::new();
        write_array_header(&mut arg_bytes, args.len());
        for &(kind, offset, size) in args {
            write_map_header(&mut arg_bytes, 3);
            write_str(&mut arg_bytes, ".offset");
            write_u64(&mut arg_bytes, offset);
            write_str(&mut arg_bytes, ".size");
            write_u64(&mut arg_bytes, size);
            write_str(&mut arg_bytes, ".value_kind");
            write_str(&mut arg_bytes, kind);
        }

        let mut kernel = Vec::new();
        write_map_header(&mut kernel, 4);
        write_str(&mut kernel, ".name");
        write_str(&mut kernel, name);
        write_str(&mut kernel, ".symbol");
        write_str(&mut kernel, symbol);
        write_str(&mut kernel, ".kernarg_segment_size");
        write_u64(&mut kernel, kernarg_size);
        write_str(&mut kernel, ".args");
        kernel.extend_from_slice(&arg_bytes);

        let mut kernels = Vec::new();
        write_array_header(&mut kernels, 1);
        kernels.extend_from_slice(&kernel);

        let mut root = Vec::new();
        write_map_header(&mut root, 1);
        write_str(&mut root, "amdhsa.kernels");
        root.extend_from_slice(&kernels);
        root
    }

    fn wrap_bundle(elf: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(CLANG_OFFLOAD_BUNDLE_MAGIC);
        push_u64(&mut out, 1); // one entry
        // Placeholder offset/size — fill after id.
        let toc_pos = out.len();
        push_u64(&mut out, 0); // offset
        push_u64(&mut out, elf.len() as u64);
        let id = b"hipv4-amdgcn-amd-amdhsa--gfx1100";
        push_u64(&mut out, id.len() as u64);
        out.extend_from_slice(id);
        let data_off = out.len() as u64;
        out[toc_pos..toc_pos + 8].copy_from_slice(&data_off.to_le_bytes());
        out.extend_from_slice(elf);
        out
    }
    fn wrap_bundle_entries(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
        let toc_len = CLANG_OFFLOAD_BUNDLE_MAGIC.len()
            + 8
            + entries.iter().map(|(id, _)| 24 + id.len()).sum::<usize>();
        let mut out = Vec::new();
        out.extend_from_slice(CLANG_OFFLOAD_BUNDLE_MAGIC);
        push_u64(&mut out, entries.len() as u64);
        let mut offset = toc_len;
        for &(id, payload) in entries {
            push_u64(&mut out, offset as u64);
            push_u64(&mut out, payload.len() as u64);
            push_u64(&mut out, id.len() as u64);
            out.extend_from_slice(id);
            offset += payload.len();
        }
        for &(_, payload) in entries {
            out.extend_from_slice(payload);
        }
        out
    }

    #[test]
    fn metadata_layout_from_synthetic_elf() {
        let msg = synthetic_metadata_msgpack(
            "dispatch_tiny",
            "dispatch_tiny.kd",
            24,
            &[
                ("global_buffer", 0, 8),
                ("global_buffer", 8, 8),
                ("by_value", 16, 4),
                ("by_value", 20, 4),
            ],
        );
        let elf = synthetic_elf_with_metadata(&msg);
        // loader size authoritative and covers all fields
        let layout = kernarg_layout(&elf, "dispatch_tiny", 24);
        assert!(layout.from_metadata);
        assert_eq!(layout.symbol, "dispatch_tiny.kd");
        assert_eq!(layout.segment_size, 24);
        assert_eq!(layout.fields.len(), 4);
        assert_eq!(layout.fields[0].value_kind, "global_buffer");
        assert_eq!(layout.fields[2].offset, 16);
        assert_eq!(layout.fields[2].size, 4);
    }

    #[test]
    fn loader_size_authoritative_rejects_overflow_fields() {
        let msg = synthetic_metadata_msgpack(
            "k",
            "k.kd",
            24,
            &[
                ("global_buffer", 0, 8),
                ("by_value", 16, 4),
                ("by_value", 20, 4),
            ],
        );
        let elf = synthetic_elf_with_metadata(&msg);
        // field end 24 > loader 16 => fallback, segment_size == loader
        let layout = kernarg_layout(&elf, "k", 16);
        assert!(!layout.from_metadata);
        assert_eq!(layout.segment_size, 16);
        assert_eq!(layout.symbol, "k.kd");
        assert!(layout.fields.iter().all(|f| f.value_kind == "by_value"));
    }

    #[test]
    fn loader_zero_uses_metadata_segment_for_discovery() {
        let msg = synthetic_metadata_msgpack(
            "disc",
            "disc.kd",
            32,
            &[("global_buffer", 0, 8), ("by_value", 24, 8)],
        );
        let elf = synthetic_elf_with_metadata(&msg);
        let layout = kernarg_layout(&elf, "disc", 0);
        assert!(layout.from_metadata);
        assert_eq!(layout.segment_size, 32);
        assert_eq!(layout.symbol, "disc.kd");
        assert_eq!(layout.fields.len(), 2);
    }

    #[test]
    fn note_name_must_be_amdgpu() {
        let msg = synthetic_metadata_msgpack("n", "n.kd", 8, &[("by_value", 0, 8)]);
        let bad = synthetic_elf_with_note(&msg, b"NOTAMD\0");
        let layout = kernarg_layout(&bad, "n", 8);
        assert!(!layout.from_metadata);

        let good_nul = synthetic_elf_with_note(&msg, b"AMDGPU\0");
        assert!(kernarg_layout(&good_nul, "n", 8).from_metadata);

        let good_bare = synthetic_elf_with_note(&msg, b"AMDGPU");
        assert!(kernarg_layout(&good_bare, "n", 8).from_metadata);
    }

    #[test]
    fn msgpack_depth_cap_rejects_deep_nesting() {
        // Build 40 nested single-element arrays: [[[[[...]]]]]
        let mut bytes = Vec::new();
        bytes.extend(std::iter::repeat_n(0x91u8, 40)); // fixarray of 1
        bytes.push(0x00); // fixint 0
        assert!(msgpack_parse(&bytes).is_none());

        // 16 nested arrays is fine (well under 32)
        let mut shallow = Vec::new();
        shallow.extend(std::iter::repeat_n(0x91u8, 16));
        shallow.push(0x01);
        assert!(msgpack_parse(&shallow).is_some());
    }

    #[test]
    fn msgpack_entry_cap_rejects_huge_array_header() {
        // array 32 with claimed length 100_000 but no payload
        let mut bytes = vec![0xdd];
        bytes.extend_from_slice(&100_000u32.to_be_bytes());
        assert!(msgpack_parse(&bytes).is_none());
    }

    #[test]
    fn fallback_pointer_fields_cover_loader_size() {
        let layout = kernarg_layout(b"not-an-elf", "my_kernel", 24);
        assert!(!layout.from_metadata);
        assert_eq!(layout.symbol, "my_kernel.kd");
        assert_eq!(layout.segment_size, 24);
        let ptr = std::mem::size_of::<usize>();
        assert_eq!(layout.fields.len(), 24usize.div_ceil(ptr));
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[0].size, ptr.min(24));
        assert_eq!(layout.fields[0].value_kind, "by_value");
        let end = layout
            .fields
            .iter()
            .map(|f| f.offset + f.size)
            .max()
            .unwrap_or(0);
        assert_eq!(end, 24);
    }

    #[test]
    fn fallback_keeps_kd_suffix() {
        let layout = kernarg_layout(&[], "foo.kd", 8);
        assert_eq!(layout.symbol, "foo.kd");
        assert!(!layout.from_metadata);
    }

    #[test]
    fn metadata_matches_kd_and_bare_names() {
        let msg = synthetic_metadata_msgpack("k", "k.kd", 8, &[("global_buffer", 0, 8)]);
        let elf = synthetic_elf_with_metadata(&msg);
        let a = kernarg_layout(&elf, "k", 8);
        let b = kernarg_layout(&elf, "k.kd", 8);
        assert!(a.from_metadata && b.from_metadata);
        assert_eq!(a.symbol, "k.kd");
        assert_eq!(b.symbol, "k.kd");
    }

    #[test]
    fn bundle_unwrap_feeds_metadata() {
        let msg = synthetic_metadata_msgpack(
            "bundled",
            "bundled.kd",
            16,
            &[("by_value", 0, 8), ("by_value", 8, 8)],
        );
        let elf = synthetic_elf_with_metadata(&msg);
        let bundle = wrap_bundle(&elf);
        let unwrapped = unwrap_offload_bundle_for_parse(&bundle).expect("unwrap");
        assert_eq!(&unwrapped[..4], ELF_MAGIC);
        let layout = kernarg_layout(&bundle, "bundled", 16);
        assert!(layout.from_metadata);
        assert_eq!(layout.fields.len(), 2);
        assert_eq!(layout.symbol, "bundled.kd");
    }

    #[test]
    fn bundle_selects_exact_device_arch_and_ignores_host_entry() {
        let bundle = wrap_bundle_entries(&[
            (b"host-x86_64-unknown-linux-gnu", b"host"),
            (b"host-x86_64--gfx1201", b"not-device-code"),
            (b"hipv4-amdgcn-amd-amdhsa--gfx1100", b"gfx11"),
            (
                b"hipv4-amdgcn-amd-amdhsa--gfx1201:sramecc-:xnack+",
                b"gfx12",
            ),
        ]);

        assert_eq!(
            select_bundle_code_object(&bundle, "gfx1201:sramecc+"),
            Some(&b"gfx12"[..])
        );
        assert!(select_bundle_code_object(&bundle, "gfx120").is_none());
        assert!(select_bundle_code_object(&bundle, "gfx942").is_none());

        let host_only = wrap_bundle_entries(&[(b"host-x86_64-unknown-linux-gnu", b"host")]);
        assert!(select_bundle_code_object(&host_only, "gfx1201").is_none());
    }

    #[test]
    fn pristine_clang_v2_bundle_toc_and_length() {
        let host_id = b"host-x86_64-unknown-linux-gnu-";
        let device_id = b"hipv4-amdgcn-amd-amdhsa--gfx1201";
        let image_offset = 0x1000_usize;
        let image_len = 8040_usize;
        let mut bundle = Vec::with_capacity(image_offset + image_len);
        bundle.extend_from_slice(CLANG_OFFLOAD_BUNDLE_MAGIC);
        push_u64(&mut bundle, 2);
        push_u64(&mut bundle, image_offset as u64);
        push_u64(&mut bundle, 0);
        push_u64(&mut bundle, host_id.len() as u64);
        bundle.extend_from_slice(host_id);
        push_u64(&mut bundle, image_offset as u64);
        push_u64(&mut bundle, image_len as u64);
        push_u64(&mut bundle, device_id.len() as u64);
        bundle.extend_from_slice(device_id);
        assert_eq!(bundle.len(), 142);
        bundle.resize(image_offset + image_len, 0);
        bundle[image_offset..image_offset + ELF_MAGIC.len()].copy_from_slice(ELF_MAGIC);

        let info = bundle_debug_info(&bundle, "gfx1201").expect("bundle info");
        assert_eq!(info.magic, "clang");
        assert_eq!(info.version, 2);
        assert_eq!(info.entries, 2);
        assert_eq!(info.selected, Some("gfx1201"));
        let selected = select_bundle_code_object(&bundle, "gfx1201").expect("gfx1201 code object");
        assert_eq!(selected.len(), image_len);
        assert!(selected.starts_with(ELF_MAGIC));
        // The first u64 after the 24-byte magic is the entry count, not an
        // additional TOC word. The first descriptor begins immediately after it.
        assert_eq!(
            read_u64_at(&bundle, CLANG_OFFLOAD_BUNDLE_MAGIC.len()),
            Some(2)
        );
        assert_eq!(
            read_u64_at(&bundle, CLANG_OFFLOAD_BUNDLE_MAGIC.len() + 8),
            Some(image_offset as u64)
        );
        // SAFETY: `bundle` owns all bytes described by its two-entry TOC.
        let inferred = unsafe { infer_image_len(bundle.as_ptr().cast()) }.expect("bundle length");
        assert_eq!(inferred, bundle.len());
        assert_eq!(bundle.len(), 12_136);
    }

    #[test]
    fn infer_elf_length_matches_synthetic() {
        let msg = synthetic_metadata_msgpack("n", "n.kd", 8, &[("by_value", 0, 8)]);
        let elf = synthetic_elf_with_metadata(&msg);
        // SAFETY: elf is a live local buffer.
        let len = unsafe { infer_image_len(elf.as_ptr().cast()) }.expect("len");
        assert_eq!(len, elf.len());
    }

    #[test]
    fn infer_bundle_length_matches_wrapper() {
        let msg = synthetic_metadata_msgpack("n", "n.kd", 8, &[("by_value", 0, 8)]);
        let elf = synthetic_elf_with_metadata(&msg);
        let bundle = wrap_bundle(&elf);
        // SAFETY: bundle is a live local buffer.
        let len = unsafe { infer_image_len(bundle.as_ptr().cast()) }.expect("len");
        assert_eq!(len, bundle.len());
    }

    #[test]
    fn copy_code_object_image_roundtrips() {
        let msg = synthetic_metadata_msgpack("n", "n.kd", 8, &[("by_value", 0, 8)]);
        let elf = synthetic_elf_with_metadata(&msg);
        // SAFETY: elf is a live local buffer for its full length.
        let copied = unsafe { copy_code_object_image(elf.as_ptr().cast()) }.expect("copy");
        assert_eq!(&copied[..], &elf[..]);
    }

    #[test]
    fn copy_rejects_null_and_garbage() {
        // SAFETY: null is explicitly handled.
        assert!(unsafe { copy_code_object_image(std::ptr::null()) }.is_none());
        let garbage = b"not-elf-or-bundle!!!!!!!!!!!!!!!!";
        // SAFETY: garbage is a live local; inference fails after magic probe.
        assert!(unsafe { copy_code_object_image(garbage.as_ptr().cast()) }.is_none());
    }

    #[test]
    fn unknown_kernel_falls_back() {
        let msg = synthetic_metadata_msgpack("other", "other.kd", 8, &[("by_value", 0, 8)]);
        let elf = synthetic_elf_with_metadata(&msg);
        let layout = kernarg_layout(&elf, "missing", 16);
        assert!(!layout.from_metadata);
        assert_eq!(layout.symbol, "missing.kd");
        assert_eq!(layout.segment_size, 16);
    }
}
