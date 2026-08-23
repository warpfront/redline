// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Host device manifest: named aliases and two refusal tiers for one machine.
//!
//! # Why this exists
//!
//! A pin written as `index:1` or a bare ROCr ordinal is a footgun: ROCr and
//! rocm-smi disagree on order, and on a heterogeneous host one of those
//! ordinals is the integrated APU whose device reset takes the whole machine
//! down. The manifest lets an operator give each GPU a stable hostname-local
//! name once (`@dev0`, `@navi10`) and refuse the dangerous devices by default.
//!
//! # Deny and fragile — two tiers
//!
//! Danger cannot be auto-detected. On a host that carries a Strix Halo APU
//! next to discrete boards, both plausible sysfs signals fail: the APU's KFD
//! node reports `cpu_cores_count = 0` exactly like the discrete cards, and
//! every node reports `heap_type = 1` because the APU's unified memory
//! presents as public framebuffer. There is therefore no integrated/
//! is_dangerous bit in [`DeviceIdentity`] — only this file, written by a
//! human who knows which BDF is dangerous. The manifest offers two tiers:
//!
//! - `deny` — never selectable, for any operation. Use for devices that must
//!   never be touched (e.g. a board reserved for display).
//! - `fragile` — selectable for normal work, but refused when the caller
//!   declares a reset-risk operation. The Strix Halo APU is the canonical
//!   example: a device reset on that APU takes the whole host down, which
//!   only matters for harnesses that deliberately provoke resets. Normal
//!   dispatch and enumeration stay allowed so the APU remains a legitimate
//!   inference target (96 GB unified carveout, first-class architecture).
//!
//! Both lists are evaluated on the **resolved device**, not the query string,
//! so `uuid:`, `bdf:`, `slot:`, `name:`, `index:`, and `@alias` cannot bypass
//! either tier. [`HostManifest::check_denied`] is the legacy deny-only entry
//! point (kept for `Normal` callers); [`HostManifest::check_with_risk`] adds
//! the [`RiskClass`] discriminator.
//!
//! Values are selector strings in the shared grammar. Alias values stay raw
//! until the caller parses them with [`crate::selector::parse`]; deny and
//! fragile entries are parsed at load time so a malformed entry cannot
//! silently no-op.
//!
//! # File discovery
//!
//! First match wins per key; later files override earlier ones:
//!
//! 1. `/etc/redline/devices.toml`
//! 2. `$XDG_CONFIG_HOME/redline/devices.toml` (else `~/.config/…`)
//! 3. `./.redline/devices.toml` walking up from the cwd to `/` (repo-local wins)
//!
//! The active section is `[host.<hostname>]`, overridable with `REDLINE_HOST`.
//! A missing manifest is not an error — it yields an empty one.
//!
//! # Notes on deny / fragile entries
//!
//! Trailing `#` comments on `deny` and `fragile` array entries are captured as
//! the human reason quoted in [`RuntimeError::DeviceDenied`] /
//! [`RuntimeError::DeviceFragile`]. That string is the single most valuable
//! line in the file ("why is this device forbidden / fragile").

use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::identity::{DeviceIdentity, DeviceQuery};
use crate::runtime::RuntimeError;
use crate::selector;

/// Reserved key inside a `[host.*]` section — never treated as an alias name.
const DENY_KEY: &str = "deny";
/// Reserved key for the weaker tier — also never an alias name.
const FRAGILE_KEY: &str = "fragile";

/// Risk classification passed at resolution time.
///
/// `Normal` is the default for existing call sites. `ResetProvoking` is for
/// harnesses that intentionally provoke device resets and therefore must not
/// land on a fragile device.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum RiskClass {
    #[default]
    Normal,
    ResetProvoking,
}

/// One host's aliases plus the devices that must never (or conditionally) be selected.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HostManifest {
    /// Active hostname section that was selected (empty when no section matched).
    host: String,
    /// Alias name → selector string (`uuid:…`, `bdf:…`, …).
    aliases: BTreeMap<String, String>,
    /// Denied devices with optional trailing-comment notes.
    deny: Vec<DeniedDevice>,
    /// Fragile devices with optional trailing-comment notes.
    fragile: Vec<FragileDevice>,
}

/// A single deny-list entry, already grammar-checked at load time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeniedDevice {
    /// Selector string as written in the file (shared grammar).
    pub selector: String,
    /// Parsed form used for fail-closed matching against a resolved device.
    pub query: DeviceQuery,
    /// Trailing `#` comment on that array entry, if any.
    pub note: Option<String>,
}

/// A single fragile-list entry, already grammar-checked at load time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FragileDevice {
    /// Selector string as written in the file (shared grammar).
    pub selector: String,
    /// Parsed form used for fail-closed matching against a resolved device.
    pub query: DeviceQuery,
    /// Trailing `#` comment on that array entry, if any.
    pub note: Option<String>,
}

impl HostManifest {
    /// Load the manifest for this host from the default search roots.
    ///
    /// Missing files are skipped. A completely absent manifest yields an empty
    /// result, not an error.
    pub fn load() -> Result<Self, RuntimeError> {
        let hostname = active_hostname();
        let paths = default_search_paths();
        Self::load_from(&paths, &hostname)
    }

    /// Load from an explicit ordered path list and hostname.
    ///
    /// Later files override earlier ones for the same alias / deny / fragile list.
    /// Parameterized so unit tests never touch real `/etc` or `$HOME`: pass a
    /// temp-dir fixture list instead of [`default_search_paths`].
    pub fn load_from(paths: &[PathBuf], hostname: &str) -> Result<Self, RuntimeError> {
        let mut aliases = BTreeMap::new();
        let mut deny: Vec<DeniedDevice> = Vec::new();
        let mut fragile: Vec<FragileDevice> = Vec::new();

        for path in paths {
            if !path.is_file() {
                continue;
            }
            let text = fs::read_to_string(path).map_err(|source| RuntimeError::ManifestIo {
                path: path.display().to_string(),
                message: source.to_string(),
            })?;
            let file = parse_devices_toml(&text).map_err(|err| RuntimeError::ManifestParse {
                path: path.display().to_string(),
                line: err.line,
                message: err.message,
            })?;
            let Some(section) = file.hosts.get(hostname) else {
                continue;
            };
            for (name, value) in &section.aliases {
                aliases.insert(name.clone(), value.clone());
            }
            // Later files replace the whole deny/fragile lists for this host so a
            // repo-local manifest can both add and clear system entries.
            if section.deny_seen {
                deny = section.deny.clone();
            }
            if section.fragile_seen {
                fragile = section.fragile.clone();
            }
        }

        Ok(Self {
            host: hostname.to_owned(),
            aliases,
            deny,
            fragile,
        })
    }

    /// Hostname section this manifest was resolved against.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Look up an alias for the active host. `deny` and `fragile` are never aliases.
    pub fn lookup_alias(&self, name: &str) -> Option<&str> {
        if name == DENY_KEY || name == FRAGILE_KEY {
            return None;
        }
        self.aliases.get(name).map(String::as_str)
    }

    /// Expand `@alias` to a [`DeviceQuery`], or error if undefined.
    pub fn resolve_alias(&self, name: &str) -> Result<DeviceQuery, RuntimeError> {
        let Some(raw) = self.lookup_alias(name) else {
            return Err(RuntimeError::AliasNotFound {
                alias: name.to_owned(),
                host: self.host.clone(),
            });
        };
        selector::parse(raw)
    }

    /// All aliases for the active host (never includes `deny`/`fragile`).
    pub fn aliases(&self) -> &BTreeMap<String, String> {
        &self.aliases
    }

    /// Deny-list entries for the active host.
    pub fn deny_list(&self) -> &[DeniedDevice] {
        &self.deny
    }

    /// Fragile-list entries for the active host.
    pub fn fragile_list(&self) -> &[FragileDevice] {
        &self.fragile
    }

    /// Refuse a **resolved** device that appears on the deny-list.
    ///
    /// Matching is against the device's identity fields (UUID, BDF, slot, name,
    /// ROCr index), not against the caller's query string. That is what makes
    /// deny fail-closed across every selector form: resolving `index:1`,
    /// `name:gfx1151`, or `@apu` to the Strix Halo still hits the same
    /// `bdf:0000:bf:00.0` deny entry. This is the legacy entry point and is
    /// equivalent to [`Self::check_with_risk`] with [`RiskClass::Normal`].
    pub fn check_denied(&self, device: &DeviceIdentity) -> Result<(), RuntimeError> {
        self.check_with_risk(device, RiskClass::Normal)
    }

    /// Refuse a **resolved** device according to the caller's risk class.
    ///
    /// - `Normal` refuses only `deny` matches (fragile devices are allowed).
    /// - `ResetProvoking` refuses both `deny` and `fragile` matches.
    ///
    /// Matching is on the resolved device identity so no selector form can
    /// bypass either tier. This keeps the existing `check_denied` working
    /// unchanged for `Normal` callers while giving reset-risk harnesses a
    /// single discriminator.
    pub fn check_with_risk(
        &self,
        device: &DeviceIdentity,
        risk: RiskClass,
    ) -> Result<(), RuntimeError> {
        for entry in &self.deny {
            if device_matches_query(device, &entry.query, self) {
                return Err(RuntimeError::DeviceDenied {
                    device: format!("{} ({})", device.anchor(), device.bdf),
                    selector: entry.selector.clone(),
                    note: entry.note.clone(),
                    host: self.host.clone(),
                });
            }
        }
        if risk == RiskClass::ResetProvoking {
            for entry in &self.fragile {
                if device_matches_query(device, &entry.query, self) {
                    return Err(RuntimeError::DeviceFragile {
                        device: format!("{} ({})", device.anchor(), device.bdf),
                        selector: entry.selector.clone(),
                        note: entry.note.clone(),
                        host: self.host.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Legacy alias for [`Self::check_with_risk`] — prefer the latter.
    pub fn check_denied_with_risk(
        &self,
        device: &DeviceIdentity,
        risk: RiskClass,
    ) -> Result<(), RuntimeError> {
        self.check_with_risk(device, risk)
    }
}

/// True when `device` is the target of this query (shared for deny + fragile).
fn device_matches_query(
    device: &DeviceIdentity,
    query: &DeviceQuery,
    manifest: &HostManifest,
) -> bool {
    match query {
        DeviceQuery::Uuid(want) => device
            .uuid
            .as_ref()
            .is_some_and(|u| u.eq_ignore_ascii_case(want)),
        DeviceQuery::Bdf(want) => device.bdf == *want,
        DeviceQuery::Slot(want) => device.pci_slot.as_deref() == Some(want.as_str()),
        DeviceQuery::Name(needle) => {
            let needle_l = needle.to_ascii_lowercase();
            device.agent_name.to_ascii_lowercase().contains(&needle_l)
        }
        DeviceQuery::Index(want) => device.rocr_index == *want,
        DeviceQuery::Alias(name) => match manifest.lookup_alias(name) {
            Some(raw) => match selector::parse(raw) {
                Ok(inner) => device_matches_query(device, &inner, manifest),
                Err(_) => false,
            },
            None => false,
        },
    }
}
#[allow(dead_code)]
/// True when `device` is the target of this deny query (legacy alias).
fn device_matches_deny(
    device: &DeviceIdentity,
    query: &DeviceQuery,
    manifest: &HostManifest,
) -> bool {
    device_matches_query(device, query, manifest)
}

/// Hostname used to pick the `[host.*]` section: `REDLINE_HOST`, else kernel hostname.
pub fn active_hostname() -> String {
    if let Ok(override_host) = env::var("REDLINE_HOST") {
        let trimmed = override_host.trim();
        if !trimmed.is_empty() {
            return trimmed.to_owned();
        }
    }
    hostname_from_system().unwrap_or_default()
}

fn hostname_from_system() -> Option<String> {
    // /proc is more reliable than libc in constrained test environments.
    if let Ok(name) = fs::read_to_string("/proc/sys/kernel/hostname") {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_owned());
        }
    }
    if let Ok(name) = fs::read_to_string("/etc/hostname") {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_owned());
        }
    }
    None
}

/// Default discovery order. Later paths override earlier ones.
pub fn default_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    paths.push(PathBuf::from("/etc/redline/devices.toml"));

    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        let mut p = PathBuf::from(xdg);
        p.push("redline");
        p.push("devices.toml");
        paths.push(p);
    } else if let Some(home) = env::var_os("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".config");
        p.push("redline");
        p.push("devices.toml");
        paths.push(p);
    }

    // Walk up from cwd so a repo-local `.redline/devices.toml` wins.
    if let Ok(cwd) = env::current_dir() {
        let mut dir: &Path = cwd.as_path();
        loop {
            paths.push(dir.join(".redline").join("devices.toml"));
            match dir.parent() {
                Some(parent) if parent != dir => dir = parent,
                _ => break,
            }
        }
    }

    paths
}

// ── TOML-subset parser ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
struct ParsedFile {
    hosts: BTreeMap<String, HostSection>,
}

#[derive(Clone, Debug, Default)]
struct HostSection {
    aliases: BTreeMap<String, String>,
    deny: Vec<DeniedDevice>,
    /// Whether this section set `deny` at all (so empty `deny = []` can clear).
    deny_seen: bool,
    fragile: Vec<FragileDevice>,
    /// Whether this section set `fragile` at all (so empty `fragile = []` can clear).
    fragile_seen: bool,
}

/// A `devices.toml` syntax error, carrying the offending line number so an
/// operator can fix the file rather than guess which entry is malformed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    line: usize,
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

/// Fully parsed multi-host file (before hostname selection).
#[derive(Clone, Debug, Default)]
pub struct ParsedManifest {
    hosts: BTreeMap<String, HostSection>,
}

impl ParsedManifest {
    /// Hostnames present in the file.
    pub fn host_names(&self) -> impl Iterator<Item = &str> {
        self.hosts.keys().map(String::as_str)
    }

    /// Build a [`HostManifest`] for one hostname without touching the filesystem.
    pub fn for_host(&self, hostname: &str) -> HostManifest {
        let Some(section) = self.hosts.get(hostname) else {
            return HostManifest {
                host: hostname.to_owned(),
                aliases: BTreeMap::new(),
                deny: Vec::new(),
                fragile: Vec::new(),
            };
        };
        HostManifest {
            host: hostname.to_owned(),
            aliases: section.aliases.clone(),
            deny: section.deny.clone(),
            fragile: section.fragile.clone(),
        }
    }
}

/// Parse the devices.toml subset. Public for direct unit testing.
pub fn parse_devices_toml(text: &str) -> Result<ParsedManifest, ParseError> {
    let file = parse_toml_subset(text)?;
    Ok(ParsedManifest { hosts: file.hosts })
}

fn parse_toml_subset(text: &str) -> Result<ParsedFile, ParseError> {
    let mut file = ParsedFile::default();
    let mut current_host: Option<String> = None;

    for (idx, raw_line) in text.lines().enumerate() {
        let line_no = idx + 1;
        // Section headers and blank detection strip `#` comments. Key lines
        // keep trailing comments so deny/fragile notes (`deny = […]  # reason`)
        // survive into DeviceDenied/DeviceFragile — that string is the
        // load-bearing human context.
        let stripped = strip_full_line_comment(raw_line).trim();
        if stripped.is_empty() {
            continue;
        }

        if stripped.starts_with('[') {
            current_host = Some(parse_section_header(stripped, line_no)?);
            file.hosts
                .entry(current_host.clone().expect("just set"))
                .or_default();
            continue;
        }

        let host_name = current_host.as_ref().ok_or_else(|| ParseError {
            line: line_no,
            message: "key outside of a [host.*] section".into(),
        })?;

        // Use the raw (whitespace-trimmed) line so trailing `#` notes remain.
        let key_line = raw_line.trim();
        let (key, value_src) = split_key_value(key_line, line_no)?;
        let section = file.hosts.get_mut(host_name).expect("section inserted");

        if key == DENY_KEY {
            let entries = parse_deny_array(value_src, line_no)?;
            section.deny = entries;
            section.deny_seen = true;
        } else if key == FRAGILE_KEY {
            let entries = parse_fragile_array(value_src, line_no)?;
            section.fragile = entries;
            section.fragile_seen = true;
        } else {
            let value = parse_quoted_string(value_src, line_no)?;
            section.aliases.insert(key.to_owned(), value);
        }
    }

    Ok(file)
}

/// Strip a whole-line `#` comment, but not `#` inside quotes.
fn strip_full_line_comment(line: &str) -> &str {
    let mut in_string = false;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' if !in_string => in_string = true,
            b'"' if in_string => {
                in_string = false;
            }
            b'\\' if in_string && i + 1 < bytes.len() => {
                i += 1; // skip escaped char
            }
            b'#' if !in_string => {
                return &line[..i];
            }
            _ => {}
        }
        i += 1;
    }
    line
}

fn parse_section_header(line: &str, line_no: usize) -> Result<String, ParseError> {
    if !line.ends_with(']') {
        return Err(ParseError {
            line: line_no,
            message: format!("malformed section header {line:?}"),
        });
    }
    let inner = line[1..line.len() - 1].trim();
    let Some(name) = inner.strip_prefix("host.") else {
        return Err(ParseError {
            line: line_no,
            message: format!("unsupported section [{inner}]; expected [host.<hostname>]"),
        });
    };
    let name = name.trim();
    if name.is_empty()
        || name.contains('[')
        || name.contains(']')
        || name.contains(' ')
        || name.contains('"')
    {
        return Err(ParseError {
            line: line_no,
            message: format!("invalid host section name {name:?}"),
        });
    }
    Ok(name.to_owned())
}

fn split_key_value(line: &str, line_no: usize) -> Result<(&str, &str), ParseError> {
    let Some((key, rest)) = line.split_once('=') else {
        return Err(ParseError {
            line: line_no,
            message: format!("expected key = value, got {line:?}"),
        });
    };
    let key = key.trim();
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ParseError {
            line: line_no,
            message: format!("invalid key {key:?}"),
        });
    }
    Ok((key, rest.trim()))
}

fn parse_quoted_string(src: &str, line_no: usize) -> Result<String, ParseError> {
    let src = src.trim();
    // Allow a trailing full-line comment after the value (alias lines).
    let src = trailing_comment_split(src).0;
    let src = src.trim();
    if !src.starts_with('"') || !src.ends_with('"') || src.len() < 2 {
        return Err(ParseError {
            line: line_no,
            message: format!("expected quoted string, got {src:?}"),
        });
    }
    let inner = &src[1..src.len() - 1];
    unescape_basic(inner, line_no)
}

/// Split `value  # note` outside of strings. Returns (value, optional note).
fn trailing_comment_split(src: &str) -> (&str, Option<&str>) {
    let mut in_string = false;
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' if !in_string => in_string = true,
            b'"' if in_string => in_string = false,
            b'\\' if in_string && i + 1 < bytes.len() => i += 1,
            b'#' if !in_string => {
                let value = src[..i].trim_end();
                let note = src[i + 1..].trim();
                return (value, if note.is_empty() { None } else { Some(note) });
            }
            _ => {}
        }
        i += 1;
    }
    (src, None)
}

fn unescape_basic(inner: &str, line_no: usize) -> Result<String, ParseError> {
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(other) => {
                    return Err(ParseError {
                        line: line_no,
                        message: format!("unsupported escape \\{other}"),
                    });
                }
                None => {
                    return Err(ParseError {
                        line: line_no,
                        message: "trailing backslash in string".into(),
                    });
                }
            }
        } else if c == '"' {
            return Err(ParseError {
                line: line_no,
                message: "unescaped quote in string".into(),
            });
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

/// Parse `["a", "b" # note]` on one line; trailing comments attach to the entry.
///
/// Two shapes are accepted for the note (both round-trip into DeviceDenied):
/// - after the closing bracket: `deny = ["bdf:…"]  # note` (documented form)
/// - inside the array token: `deny = ["bdf:…" # note]`
fn parse_deny_array(src: &str, line_no: usize) -> Result<Vec<DeniedDevice>, ParseError> {
    parse_selector_array(src, line_no, "deny").map(|entries| {
        entries
            .into_iter()
            .map(|(selector, query, note)| DeniedDevice {
                selector,
                query,
                note,
            })
            .collect()
    })
}

fn parse_fragile_array(src: &str, line_no: usize) -> Result<Vec<FragileDevice>, ParseError> {
    parse_selector_array(src, line_no, "fragile").map(|entries| {
        entries
            .into_iter()
            .map(|(selector, query, note)| FragileDevice {
                selector,
                query,
                note,
            })
            .collect()
    })
}

fn parse_selector_array(
    src: &str,
    line_no: usize,
    kind: &str,
) -> Result<Vec<(String, DeviceQuery, Option<String>)>, ParseError> {
    let src = src.trim();
    if !src.starts_with('[') {
        return Err(ParseError {
            line: line_no,
            message: format!("expected array, got {src:?}"),
        });
    }

    // Capture a trailing `# note` that sits *after* the closing `]`, which is
    // the shape used in the documented example:
    //   deny = ["bdf:0000:bf:00.0"]          # Strix Halo APU: …
    let (array_src, after_bracket_note) = {
        let mut in_string = false;
        let bytes = src.as_bytes();
        let mut close = None;
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'"' if !in_string => in_string = true,
                b'"' if in_string => in_string = false,
                b'\\' if in_string && i + 1 < bytes.len() => i += 1,
                b']' if !in_string => {
                    close = Some(i);
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        let close = close.ok_or_else(|| ParseError {
            line: line_no,
            message: "unterminated array (closing ] not found on same line)".into(),
        })?;
        let after = src[close + 1..].trim();
        let note = if let Some(rest) = after.strip_prefix('#') {
            let n = rest.trim();
            if n.is_empty() {
                None
            } else {
                Some(n.to_owned())
            }
        } else if after.is_empty() {
            None
        } else {
            return Err(ParseError {
                line: line_no,
                message: format!("trailing junk after array: {after:?}"),
            });
        };
        (&src[..=close], note)
    };

    let inner = array_src[1..array_src.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    let mut buf = String::new();
    let mut in_string = false;
    let mut chars = inner.chars().peekable();
    let mut pending_note: Option<String> = None;

    let flush = |buf: &mut String,
                 pending_note: &mut Option<String>,
                 entries: &mut Vec<(String, DeviceQuery, Option<String>)>,
                 line_no: usize|
     -> Result<(), ParseError> {
        let piece = buf.trim();
        if piece.is_empty() {
            if pending_note.is_some() {
                return Err(ParseError {
                    line: line_no,
                    message: format!("{kind} note without a selector"),
                });
            }
            return Ok(());
        }
        let (value_src, inline_note) = trailing_comment_split(piece);
        let selector = parse_quoted_string(value_src, line_no)?;
        let note = inline_note
            .map(str::to_owned)
            .or_else(|| pending_note.take());
        // Fail closed at load: an entry that does not parse cannot protect
        // anyone, so refuse the file rather than silently dropping it.
        let query = selector::parse(&selector).map_err(|err| ParseError {
            line: line_no,
            message: format!("{kind} entry {selector:?} is not a valid selector: {err}"),
        })?;
        entries.push((selector, query, note));
        buf.clear();
        Ok(())
    };

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_string = !in_string;
                buf.push(c);
            }
            '\\' if in_string => {
                buf.push(c);
                if let Some(n) = chars.next() {
                    buf.push(n);
                }
            }
            '#' if !in_string => {
                let rest: String = chars.by_ref().collect();
                let note = rest.trim();
                if !note.is_empty() {
                    pending_note = Some(note.to_owned());
                }
                break;
            }
            ',' if !in_string => {
                flush(&mut buf, &mut pending_note, &mut entries, line_no)?;
            }
            _ => buf.push(c),
        }
    }
    flush(&mut buf, &mut pending_note, &mut entries, line_no)?;

    // If the only note was written after `]`, attach it to the sole entry
    // (documented single-entry form). Multi-entry arrays should put notes
    // next to each item inside the brackets.
    if let Some(note) = after_bracket_note {
        match entries.as_mut_slice() {
            [(_selector, _query, existing_note)] if existing_note.is_none() => {
                *existing_note = Some(note)
            }
            _ => {
                let _ = note;
            }
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::PciBusId;
    use std::io::Write;
    use std::sync::Mutex;

    /// Serialize env-var tests so parallel test threads cannot clobber each other.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    const EXAMPLE: &str = r#"
[host.hipx]
dev0    = "uuid:GPU-43390a851e296ee5"   # gfx1100, the ROCm#6529 card
navi10  = "bdf:0000:6e:00.0"            # no UUID; BDF-anchored, slot 1
rx6800  = "uuid:GPU-c7ff6b154d0128bc"
deny    = ["bdf:0000:bf:00.0"]          # Strix Halo APU: a device reset takes the host down

[host.hiptrx]
dev0 = "uuid:GPU-9eb7aeda51c88ffd"
dev1 = "uuid:GPU-05f92432f2312a0e"
dev2 = "uuid:GPU-085289909a86cc63"
dev3 = "uuid:GPU-e475645fe0200397"
"#;

    fn bdf(s: &str) -> PciBusId {
        s.parse().unwrap()
    }

    fn device(
        uuid: Option<&str>,
        bdf_s: &str,
        agent: &str,
        rocr_index: usize,
        slot: Option<&str>,
    ) -> DeviceIdentity {
        DeviceIdentity {
            uuid: uuid.map(str::to_owned),
            bdf: bdf(bdf_s),
            agent_name: agent.to_owned(),
            product_name: agent.to_owned(),
            kfd_node: rocr_index as u32,
            rocr_index,
            hip_ordinal: None,
            pci_slot: slot.map(str::to_owned),
            drm_card: None,
        }
    }

    #[test]
    fn parse_example_aliases_and_deny() {
        let parsed = parse_devices_toml(EXAMPLE).expect("example parses");
        let hipx = parsed.for_host("hipx");
        assert_eq!(hipx.lookup_alias("dev0"), Some("uuid:GPU-43390a851e296ee5"));
        assert_eq!(hipx.lookup_alias("navi10"), Some("bdf:0000:6e:00.0"));
        assert_eq!(
            hipx.lookup_alias("rx6800"),
            Some("uuid:GPU-c7ff6b154d0128bc")
        );
        // deny is reserved — never an alias
        assert_eq!(hipx.lookup_alias("deny"), None);
        assert!(!hipx.aliases().contains_key("deny"));

        assert_eq!(hipx.deny_list().len(), 1);
        assert_eq!(hipx.deny_list()[0].selector, "bdf:0000:bf:00.0");
        assert_eq!(
            hipx.deny_list()[0].note.as_deref(),
            Some("Strix Halo APU: a device reset takes the host down")
        );
        assert_eq!(
            hipx.deny_list()[0].query,
            DeviceQuery::Bdf(bdf("0000:bf:00.0"))
        );

        let hiptrx = parsed.for_host("hiptrx");
        assert_eq!(
            hiptrx.lookup_alias("dev0"),
            Some("uuid:GPU-9eb7aeda51c88ffd")
        );
        assert_eq!(
            hiptrx.lookup_alias("dev3"),
            Some("uuid:GPU-e475645fe0200397")
        );
        assert!(hiptrx.deny_list().is_empty());
    }

    #[test]
    fn deny_check_on_resolved_device_quotes_note() {
        let hipx = parse_devices_toml(EXAMPLE).unwrap().for_host("hipx");

        // The Strix Halo as it appears on hipx (ROCr index 1, no UUID).
        let apu = device(None, "0000:bf:00.0", "gfx1151", 1, None);
        let err = hipx.check_denied(&apu).expect_err("must deny");
        let text = err.to_string();
        // Caller-visible message (paste target for acceptance):
        // device bdf:0000:bf:00.0 (0000:bf:00.0) is denied by host manifest [hipx] \
        // (deny entry bdf:0000:bf:00.0): Strix Halo APU: a device reset takes the host down
        assert!(
            text.contains("bdf:0000:bf:00.0"),
            "error must quote selector/bdf: {text}"
        );
        assert!(
            text.contains("Strix Halo APU: a device reset takes the host down"),
            "error must quote deny note: {text}"
        );
        assert!(
            text.contains("hipx"),
            "error should name the host profile: {text}"
        );
        assert!(text.contains("denied"), "error must say denied: {text}");

        // Same device reached via any other identity view still denied.
        let apu_as_if_named = device(None, "0000:bf:00.0", "gfx1151", 99, Some("none"));
        assert!(hipx.check_denied(&apu_as_if_named).is_err());

        // Discrete card on the same host is allowed.
        let dev0 = device(
            Some("GPU-43390a851e296ee5"),
            "0000:66:00.0",
            "gfx1100",
            0,
            None,
        );
        hipx.check_denied(&dev0).expect("not denied");
    }

    #[test]
    fn deny_matches_regardless_of_query_form_fields() {
        // Deny by UUID; device resolved via index still blocked.
        let text = r#"
[host.h]
deny = ["uuid:GPU-DEAD"]  # do not touch
"#;
        let m = parse_devices_toml(text).unwrap().for_host("h");
        let d = device(Some("GPU-DEAD"), "0000:01:00.0", "gfx1201", 3, None);
        let err = m.check_denied(&d).unwrap_err().to_string();
        assert!(err.contains("do not touch"), "{err}");
        assert!(err.contains("GPU-DEAD") || err.contains("uuid:"), "{err}");
    }

    #[test]
    fn malformed_line_reports_line_number() {
        let bad = "\n\nnot a key value\n";
        let err = parse_devices_toml(bad).expect_err("must fail");
        assert_eq!(err.line, 3, "{err}");

        let bad_header = "[host.hipx]\ndev0 = 123\n";
        let err = parse_devices_toml(bad_header).expect_err("must fail");
        assert_eq!(err.line, 2, "{err}");

        let junk = "[other.section]\n";
        let err = parse_devices_toml(junk).expect_err("must fail");
        assert_eq!(err.line, 1, "{err}");

        let bad_deny = "[host.x]\ndeny = [\"not-a-selector\"]\n";
        let err = parse_devices_toml(bad_deny).expect_err("must fail");
        assert_eq!(err.line, 2, "{err}");
        assert!(
            err.message.contains("deny entry") || err.message.contains("selector"),
            "{err}"
        );
    }

    #[test]
    fn missing_file_yields_empty_not_error() {
        let paths = vec![PathBuf::from("/no/such/redline-manifest-test.toml")];
        let m = HostManifest::load_from(&paths, "hipx").expect("missing is ok");
        assert!(m.aliases().is_empty());
        assert!(m.deny_list().is_empty());
        assert_eq!(m.host(), "hipx");
    }

    #[test]
    fn override_precedence_across_two_files() {
        let dir = std::env::temp_dir().join(format!(
            "redline-manifest-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let early = dir.join("early.toml");
        let late = dir.join("late.toml");

        write(
            &early,
            r#"
[host.hipx]
dev0 = "uuid:GPU-EARLY"
navi10 = "bdf:0000:6e:00.0"
deny = ["bdf:0000:bf:00.0"]
"#,
        );
        write(
            &late,
            r#"
[host.hipx]
dev0 = "uuid:GPU-LATE"
deny = ["bdf:0000:99:00.0"]  # replaced deny list
"#,
        );

        // Parameterized roots: tests pass explicit paths, never default_search_paths.
        let m = HostManifest::load_from(&[early.clone(), late.clone()], "hipx").unwrap();
        assert_eq!(m.lookup_alias("dev0"), Some("uuid:GPU-LATE"));
        assert_eq!(m.lookup_alias("navi10"), Some("bdf:0000:6e:00.0"));
        assert_eq!(m.deny_list().len(), 1);
        assert_eq!(m.deny_list()[0].selector, "bdf:0000:99:00.0");
        assert_eq!(m.deny_list()[0].note.as_deref(), Some("replaced deny list"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn redline_host_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: single-threaded under ENV_LOCK for this test process region.
        unsafe {
            env::set_var("REDLINE_HOST", "hiptrx");
        }
        assert_eq!(active_hostname(), "hiptrx");
        unsafe {
            env::set_var("REDLINE_HOST", "  custom-lab  ");
        }
        assert_eq!(active_hostname(), "custom-lab");
        unsafe {
            env::remove_var("REDLINE_HOST");
        }
        let _ = active_hostname();
    }

    #[test]
    fn deny_never_returned_as_alias() {
        let text = r#"
[host.x]
deny = ["bdf:0000:01:00.0"]
ok = "uuid:GPU-1"
"#;
        let m = parse_devices_toml(text).unwrap().for_host("x");
        assert_eq!(m.lookup_alias("deny"), None);
        assert_eq!(m.lookup_alias("ok"), Some("uuid:GPU-1"));
    }

    #[test]
    fn empty_deny_array_clears_on_override() {
        let dir = std::env::temp_dir().join(format!(
            "redline-manifest-empty-deny-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.toml");
        let b = dir.join("b.toml");
        write(
            &a,
            "[host.h]\ndeny = [\"bdf:0000:01:00.0\"]\ndev0 = \"uuid:GPU-1\"\n",
        );
        write(&b, "[host.h]\ndeny = []\n");
        let m = HostManifest::load_from(&[a, b], "h").unwrap();
        assert!(m.deny_list().is_empty());
        assert_eq!(m.lookup_alias("dev0"), Some("uuid:GPU-1"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_alias_parses_selector() {
        let hipx = parse_devices_toml(EXAMPLE).unwrap().for_host("hipx");
        let q = hipx.resolve_alias("navi10").unwrap();
        assert_eq!(q, DeviceQuery::Bdf(bdf("0000:6e:00.0")));
        let err = hipx.resolve_alias("missing").unwrap_err().to_string();
        assert!(err.contains("@missing"), "{err}");
        assert!(err.contains("hipx"), "{err}");
    }

    // ── fragile tier tests ────────────────────────────────────────────────

    #[test]
    fn fragile_parses_including_reason_capture() {
        let text = r#"
[host.hipx]
fragile = ["bdf:0000:bf:00.0"]  # Strix Halo APU — a device reset takes the whole host down
"#;
        let m = parse_devices_toml(text).unwrap().for_host("hipx");
        assert_eq!(m.fragile_list().len(), 1);
        assert_eq!(m.fragile_list()[0].selector, "bdf:0000:bf:00.0");
        assert_eq!(
            m.fragile_list()[0].note.as_deref(),
            Some("Strix Halo APU — a device reset takes the whole host down")
        );
        // fragile is reserved — never an alias
        assert_eq!(m.lookup_alias("fragile"), None);
        assert!(!m.aliases().contains_key("fragile"));
        assert!(m.deny_list().is_empty());
    }

    #[test]
    fn fragile_malformed_errors_with_line_number() {
        let bad = "[host.x]\nfragile = [\"not-a-selector\"]\n";
        let err = parse_devices_toml(bad).expect_err("must fail");
        assert_eq!(err.line, 2, "{err}");
        assert!(
            err.message.contains("fragile entry") || err.message.contains("selector"),
            "{err}"
        );

        let bad2 = "[host.x]\nfragile = [\"bdf:0000:01:00.0\" # ok]\n trailing junk";
        // trailing junk after array should already have been caught at line 2
        let _ = bad2;
    }

    #[test]
    fn fragile_and_deny_are_independent_lists() {
        let text = r#"
[host.h]
deny = ["bdf:0000:01:00.0"]  # always denied
fragile = ["bdf:0000:02:00.0"]  # only reset-risk
"#;
        let m = parse_devices_toml(text).unwrap().for_host("h");
        assert_eq!(m.deny_list().len(), 1);
        assert_eq!(m.fragile_list().len(), 1);
        assert_eq!(m.deny_list()[0].selector, "bdf:0000:01:00.0");
        assert_eq!(m.fragile_list()[0].selector, "bdf:0000:02:00.0");
    }

    #[test]
    fn normal_allows_fragile_device() {
        let text = r#"
[host.h]
fragile = ["bdf:0000:bf:00.0"]  # fragile reason
"#;
        let m = parse_devices_toml(text).unwrap().for_host("h");
        let apu = device(None, "0000:bf:00.0", "gfx1151", 1, None);
        m.check_with_risk(&apu, RiskClass::Normal)
            .expect("fragile allowed for Normal");
        // legacy check_denied also allows it
        m.check_denied(&apu).expect("deny-only allows fragile");
    }

    #[test]
    fn reset_provoking_refuses_fragile() {
        let text = r#"
[host.h]
fragile = ["bdf:0000:bf:00.0"]  # Strix Halo APU — a device reset takes the whole host down
"#;
        let m = parse_devices_toml(text).unwrap().for_host("h");
        let apu = device(None, "0000:bf:00.0", "gfx1151", 1, None);
        let err = m
            .check_with_risk(&apu, RiskClass::ResetProvoking)
            .expect_err("must refuse fragile under reset risk");
        let text_out = err.to_string();
        assert!(text_out.contains("fragile"), "must say fragile: {text_out}");
        assert!(
            text_out.contains("reset-provoking"),
            "must mention reset-provoking: {text_out}"
        );
        assert!(
            text_out.contains("Strix Halo APU — a device reset takes the whole host down"),
            "must quote note: {text_out}"
        );
        assert!(text_out.contains("bdf:0000:bf:00.0"), "{text_out}");
        assert!(text_out.contains("h"), "{text_out}");
    }

    #[test]
    fn deny_refuses_under_both_classes() {
        let text = r#"
[host.h]
deny = ["bdf:0000:01:00.0"]  # always denied
"#;
        let m = parse_devices_toml(text).unwrap().for_host("h");
        let d = device(None, "0000:01:00.0", "gfx1101", 0, None);
        assert!(m.check_with_risk(&d, RiskClass::Normal).is_err());
        assert!(m.check_with_risk(&d, RiskClass::ResetProvoking).is_err());
        // fragile empty, deny still fires
        let err = m
            .check_with_risk(&d, RiskClass::ResetProvoking)
            .unwrap_err()
            .to_string();
        assert!(err.contains("denied"), "{err}");
    }

    #[test]
    fn device_neither_is_allowed_under_both() {
        let text = r#"
[host.h]
deny = ["bdf:0000:01:00.0"]
fragile = ["bdf:0000:02:00.0"]
"#;
        let m = parse_devices_toml(text).unwrap().for_host("h");
        let ok = device(None, "0000:03:00.0", "gfx1201", 2, None);
        m.check_with_risk(&ok, RiskClass::Normal).expect("allowed");
        m.check_with_risk(&ok, RiskClass::ResetProvoking)
            .expect("allowed");
    }

    #[test]
    fn fragile_evaluated_on_resolved_device_like_deny() {
        // fragile by name substring should still hit a device reached via bdf/alias
        let text = r#"
[host.h]
myapu = "bdf:0000:bf:00.0"
fragile = ["@myapu"]  # via alias
"#;
        let m = parse_devices_toml(text).unwrap().for_host("h");
        let apu = device(None, "0000:bf:00.0", "gfx1151", 5, None);
        // Normal still allows (fragile via alias)
        m.check_with_risk(&apu, RiskClass::Normal)
            .expect("normal allows");
        // ResetProvoking must expand alias and refuse
        let err = m
            .check_with_risk(&apu, RiskClass::ResetProvoking)
            .expect_err("must refuse via alias");
        assert!(err.to_string().contains("fragile"), "{}", err.to_string());
    }

    fn write(path: &Path, body: &str) {
        let mut f = fs::File::create(path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
    }
}
