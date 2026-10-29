//! Indexed, read-only access to the pinned corpus method graph.
//!
//! The index is a deterministic derivative of `methods.jsonl`: it stores only
//! raw method-id bytes, byte offsets, row lengths, and row hashes. It never
//! stores source bytes, provenance, or a second authority. The accepted graph
//! and manifests remain authoritative.

use super::*;
use crate::provenance::{EvidenceClass, Precision, SourceProvenance};
use crate::{CurrentMethodAccess, CurrentMethodMatch};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};

const INDEX_MAGIC: &[u8; 8] = b"WUWAIDX1";
const INDEX_VERSION: u32 = 1;
const INDEX_HEADER_BYTES: usize = 60;
const INDEX_ENTRY_BYTES: usize = 76;
const DEFAULT_INDEX_REL: &str = "tools/re/local-index/source-access.methods.v1.idx";
const ACCEPTED_METHODS_BYTES: u64 = 764197857;
const ACCEPTED_METHODS_SHA256: &str =
    "86441818f198af7bb220f3675098aaeb48f6f3ed6c47f2e1baa99b6aa20d4c03";
const ACCEPTED_METHOD_ROWS: u64 = 305856;
const MAX_INDEX_BYTES: u64 = 25 * 1024 * 1024;
// Primary-authorized exact DEV index pin for the frozen RE-CORPUS-ACCESS01
// candidate. There is intentionally no CLI hash override.
const PRODUCTION_INDEX_SHA256: Option<&str> =
    Some("42102369cc43a9ff6c60cab75bb00d9ade8a149ce186b19338790607de81cdbe");

#[derive(Clone)]
struct IndexTrust {
    expected_sha256: Option<String>,
}

fn production_index_trust() -> IndexTrust {
    IndexTrust {
        expected_sha256: PRODUCTION_INDEX_SHA256.map(str::to_string),
    }
}

#[cfg(test)]
fn fixture_index_trust(sha256: impl Into<String>) -> IndexTrust {
    IndexTrust {
        expected_sha256: Some(sha256.into()),
    }
}

#[cfg(test)]
fn missing_index_trust() -> IndexTrust {
    IndexTrust {
        expected_sha256: None,
    }
}

fn index_trust_status(trust: &IndexTrust) -> &'static str {
    if trust.expected_sha256.is_some() {
        "INDEX_PIN_AUTHORIZED"
    } else {
        "PRIMARY_REVIEW_PENDING"
    }
}

#[derive(Debug, Clone)]
struct IndexEntry {
    method_id: [u8; 32],
    offset: u64,
    length: u32,
    row_sha256: [u8; 32],
}

#[derive(Debug, Clone)]
struct IndexHeader {
    graph_methods_sha256: String,
    graph_methods_bytes: u64,
    row_count: u64,
}

#[derive(Debug, Clone)]
struct LoadedIndex {
    rel_path: String,
    sha256: String,
    bytes: u64,
    header: IndexHeader,
    entries: Vec<IndexEntry>,
}

fn hex32(value: &str) -> Option<[u8; 32]> {
    if !valid_hex_ci(value, 64) {
        return None;
    }
    let mut output = [0_u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(chunk).ok()?;
        output[index] = u8::from_str_radix(text, 16).ok()?;
    }
    Some(output)
}

fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset + 8)?.try_into().ok()?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileObservation {
    size: u64,
    modified: Option<std::time::SystemTime>,
}

fn observe_file(path: &Path) -> RegistryResult<FileObservation> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| reg("INDEX_FILE_METADATA", "file metadata"))?;
    if super::is_reparse(&metadata) {
        return Err(reg(
            "UNSAFE_REPARSE_OR_SYMLINK",
            "file must not be a reparse object",
        ));
    }
    if !metadata.is_file() {
        return Err(reg("PATH_NOT_FILE", "path is not a regular file"));
    }
    Ok(FileObservation {
        size: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

fn valid_hex_ci(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_uuid_ci(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && [8, 13, 18, 23]
            .iter()
            .all(|index| bytes.get(*index) == Some(&b'-'))
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| [8, 13, 18, 23].contains(&index) || byte.is_ascii_hexdigit())
}

fn valid_token_ci(value: &str, prefix: &str) -> bool {
    if !value.is_ascii()
        || value.len() != 10
        || !value[..2].eq_ignore_ascii_case("0x")
        || !value[2..4].eq_ignore_ascii_case(prefix)
    {
        return false;
    }
    let tail = &value[4..];
    tail.bytes().all(|byte| byte.is_ascii_hexdigit()) && tail.bytes().any(|byte| byte != b'0')
}

fn validate_graph_row(row: &GraphMethodRow) -> RegistryResult<()> {
    if !valid_hex_ci(&row.assembly_sha256, 64)
        || !valid_uuid_ci(&row.mvid)
        || !valid_token_ci(&row.type_token, "02")
        || !valid_token_ci(&row.method_token, "06")
        || !valid_hex_ci(&row.method_id, 64)
    {
        return Err(reg(
            "GRAPH_METHOD_IDENTITY_INVALID",
            "method row identity fields are not well formed",
        ));
    }
    let computed = method_id(
        &row.assembly_sha256,
        &row.mvid,
        &row.type_token,
        &row.method_token,
    );
    if computed != row.method_id {
        return Err(reg(
            "GRAPH_METHOD_ID_INVALID",
            "method id is not the full-key digest of the row identity",
        ));
    }
    Ok(())
}

fn serialize_index(
    graph_sha256: &str,
    graph_bytes: u64,
    row_count: u64,
    entries: &[IndexEntry],
) -> RegistryResult<Vec<u8>> {
    let graph_id = hex32(graph_sha256).ok_or_else(|| {
        reg(
            "INDEX_GRAPH_HASH_INVALID",
            "graph methods hash is not lower-hex",
        )
    })?;
    let capacity = INDEX_HEADER_BYTES
        .checked_add(
            entries
                .len()
                .checked_mul(INDEX_ENTRY_BYTES)
                .ok_or_else(|| reg("INDEX_SIZE_OVERFLOW", "entry count overflow"))?,
        )
        .ok_or_else(|| reg("INDEX_SIZE_OVERFLOW", "index size overflow"))?;
    let mut output = Vec::with_capacity(capacity);
    output.extend_from_slice(INDEX_MAGIC);
    write_u32(&mut output, INDEX_VERSION);
    output.extend_from_slice(&graph_id);
    write_u64(&mut output, graph_bytes);
    write_u64(&mut output, row_count);
    for entry in entries {
        output.extend_from_slice(&entry.method_id);
        write_u64(&mut output, entry.offset);
        write_u32(&mut output, entry.length);
        output.extend_from_slice(&entry.row_sha256);
    }
    if output.len() as u64 > MAX_INDEX_BYTES {
        return Err(reg(
            "INDEX_SIZE_LIMIT",
            "derived index exceeds the 25 MiB access-index budget",
        ));
    }
    Ok(output)
}

fn build_index_bytes<R: BufRead>(
    reader: &mut R,
    expected_bytes: u64,
    expected_sha256: &str,
    expected_rows: u64,
) -> RegistryResult<Vec<u8>> {
    let mut hasher = Sha256::new();
    let mut entries: Vec<IndexEntry> = Vec::with_capacity(expected_rows as usize);
    let mut seen: BTreeSet<[u8; 32]> = BTreeSet::new();
    let mut offset = 0_u64;
    let mut line = Vec::new();
    loop {
        line.clear();
        let read = reader
            .read_until(b'\n', &mut line)
            .map_err(|_| reg("GRAPH_METHODS_INVALID", "methods read"))?;
        if read == 0 {
            break;
        }
        if !line.ends_with(b"\n") {
            return Err(reg(
                "GRAPH_METHODS_LINE_TERMINATION",
                "methods rows must be newline-terminated",
            ));
        }
        hasher.update(&line);
        let length = u32::try_from(line.len())
            .map_err(|_| reg("GRAPH_METHODS_LINE_TOO_LONG", "methods row exceeds u32"))?;
        let row_start = offset;
        offset = offset
            .checked_add(line.len() as u64)
            .ok_or_else(|| reg("GRAPH_METHODS_OFFSET_OVERFLOW", "methods offset overflow"))?;
        let json_bytes = &line[..line.len() - 1];
        if json_bytes.is_empty() {
            return Err(reg("GRAPH_METHODS_INVALID", "empty methods row"));
        }
        let value = crate::parse_strict_value(json_bytes)
            .map_err(|_| reg("GRAPH_METHODS_INVALID", "method row strict JSON"))?;
        let row: GraphMethodRow = serde_json::from_value(value)
            .map_err(|_| reg("GRAPH_METHODS_INVALID", "method row shape"))?;
        validate_graph_row(&row)?;
        let id = hex32(&row.method_id)
            .ok_or_else(|| reg("GRAPH_METHOD_ID_INVALID", "method id is not lower-hex"))?;
        if !seen.insert(id) {
            return Err(reg(
                "GRAPH_METHOD_ID_DUPLICATE",
                "duplicate full-key method id in graph",
            ));
        }
        let digest = Sha256::digest(&line);
        let mut row_sha256 = [0_u8; 32];
        row_sha256.copy_from_slice(&digest);
        entries.push(IndexEntry {
            method_id: id,
            offset: row_start,
            length,
            row_sha256,
        });
        if entries.len() as u64 > expected_rows {
            return Err(reg(
                "GRAPH_METHODS_COUNT_MISMATCH",
                "more method rows than the accepted row count",
            ));
        }
    }
    let actual_sha256 = hex_digest(hasher.finalize());
    if offset != expected_bytes
        || actual_sha256 != expected_sha256
        || entries.len() as u64 != expected_rows
    {
        return Err(reg(
            "GRAPH_METHODS_HASH_MISMATCH",
            "methods payload size, hash, or row count differs from the accepted pin",
        ));
    }
    entries.sort_by_key(|entry| entry.method_id);
    let mut previous: Option<[u8; 32]> = None;
    for entry in &entries {
        if previous.is_some_and(|value| value >= entry.method_id) {
            return Err(reg(
                "INDEX_ENTRY_ORDER_INVALID",
                "index entries are not strictly sorted and unique",
            ));
        }
        previous = Some(entry.method_id);
        let end = entry
            .offset
            .checked_add(u64::from(entry.length))
            .ok_or_else(|| reg("INDEX_ENTRY_BOUNDS_INVALID", "entry bounds overflow"))?;
        if entry.length == 0 || entry.offset >= expected_bytes || end > expected_bytes {
            return Err(reg(
                "INDEX_ENTRY_BOUNDS_INVALID",
                "entry offset/length is outside the graph payload",
            ));
        }
    }
    serialize_index(&actual_sha256, offset, entries.len() as u64, &entries)
}

fn parse_index_bytes(
    bytes: &[u8],
    expected_index_sha256: &str,
    expected_graph_sha256: &str,
    expected_graph_bytes: u64,
    expected_rows: u64,
) -> RegistryResult<(IndexHeader, Vec<IndexEntry>)> {
    if crate::sha256_hex(bytes) != expected_index_sha256 {
        return Err(reg(
            "TRUST_INDEX_HASH_MISMATCH",
            "index bytes do not match the explicit index pin",
        ));
    }
    if bytes.len() < INDEX_HEADER_BYTES || &bytes[..8] != INDEX_MAGIC {
        return Err(reg("INDEX_INVALID", "index magic/header is invalid"));
    }
    let version =
        read_u32(bytes, 8).ok_or_else(|| reg("INDEX_INVALID", "index version is missing"))?;
    if version != INDEX_VERSION {
        return Err(reg("INDEX_INVALID", "index version is unsupported"));
    }
    let graph_sha256 = hex_digest(&bytes[12..44]);
    let graph_bytes =
        read_u64(bytes, 44).ok_or_else(|| reg("INDEX_INVALID", "index graph size is missing"))?;
    let row_count =
        read_u64(bytes, 52).ok_or_else(|| reg("INDEX_INVALID", "index row count is missing"))?;
    if graph_sha256 != expected_graph_sha256
        || graph_bytes != expected_graph_bytes
        || row_count != expected_rows
    {
        return Err(reg(
            "INDEX_GRAPH_BINDING_MISMATCH",
            "index header is not bound to the accepted graph payload",
        ));
    }
    if row_count > (MAX_INDEX_BYTES / INDEX_ENTRY_BYTES as u64) {
        return Err(reg("INDEX_INVALID", "index row count exceeds budget"));
    }
    let expected_len = INDEX_HEADER_BYTES as u64
        + row_count
            .checked_mul(INDEX_ENTRY_BYTES as u64)
            .ok_or_else(|| reg("INDEX_INVALID", "index size overflow"))?;
    if bytes.len() as u64 != expected_len {
        return Err(reg(
            "INDEX_TRUNCATED_OR_TRAILING",
            "index is not the exact fixed size for its header",
        ));
    }
    let mut entries = Vec::with_capacity(row_count as usize);
    let mut previous: Option<[u8; 32]> = None;
    let mut cursor = INDEX_HEADER_BYTES;
    for _ in 0..row_count {
        let mut method_id = [0_u8; 32];
        method_id.copy_from_slice(&bytes[cursor..cursor + 32]);
        let offset = read_u64(bytes, cursor + 32)
            .ok_or_else(|| reg("INDEX_INVALID", "index entry offset"))?;
        let length = read_u32(bytes, cursor + 40)
            .ok_or_else(|| reg("INDEX_INVALID", "index entry length"))?;
        let mut row_sha256 = [0_u8; 32];
        row_sha256.copy_from_slice(&bytes[cursor + 44..cursor + 76]);
        if previous.is_some_and(|value| value >= method_id) {
            return Err(reg(
                "INDEX_ENTRY_ORDER_INVALID",
                "index entries are not strictly sorted and unique",
            ));
        }
        previous = Some(method_id);
        let end = offset
            .checked_add(u64::from(length))
            .ok_or_else(|| reg("INDEX_ENTRY_BOUNDS_INVALID", "entry bounds overflow"))?;
        if length == 0 || offset >= graph_bytes || end > graph_bytes {
            return Err(reg(
                "INDEX_ENTRY_BOUNDS_INVALID",
                "index entry offset/length is outside the graph payload",
            ));
        }
        entries.push(IndexEntry {
            method_id,
            offset,
            length,
            row_sha256,
        });
        cursor += INDEX_ENTRY_BYTES;
    }
    Ok((
        IndexHeader {
            graph_methods_sha256: graph_sha256,
            graph_methods_bytes: graph_bytes,
            row_count,
        },
        entries,
    ))
}

fn resolve_index_path(curated: &Path, rel: &str) -> RegistryResult<PathBuf> {
    crate::validate_relative_path(rel).map_err(|_| reg("UNSAFE_RELATIVE_PATH", "index path"))?;
    match inspect_relative_entry(curated, Path::new(rel)) {
        Ok(EntryKind::File) => Ok(curated.join(rel)),
        Ok(EntryKind::Directory) => Err(reg("PATH_NOT_FILE", "index path is a directory")),
        Err("MISSING_PATH") => Err(reg("INDEX_MISSING", "derived index is missing")),
        Err("UNSAFE_REPARSE_OR_SYMLINK") => Err(reg(
            "UNSAFE_REPARSE_OR_SYMLINK",
            "index path crosses a reparse object",
        )),
        Err(code) => Err(reg(
            "PATH_INVALID",
            format!("index path rejected with {code}"),
        )),
    }
}

fn load_index_trusted(
    curated: &Path,
    rel: &str,
    trust: &IndexTrust,
) -> RegistryResult<LoadedIndex> {
    let expected = trust.expected_sha256.as_deref().ok_or_else(|| {
        reg(
            "TRUST_INDEX_PIN_UNAVAILABLE",
            "production source-locate requires the primary-authorized index hash pin",
        )
    })?;
    let path = resolve_index_path(curated, rel)?;
    inspect_existing_file(&path)?;
    let observation = observe_file(&path)?;
    if observation.size > MAX_INDEX_BYTES {
        return Err(reg(
            "INDEX_SIZE_LIMIT",
            "derived index exceeds the 25 MiB access-index budget",
        ));
    }
    let bytes = fs::read(&path).map_err(|_| reg("INDEX_READ_ERROR", "index read"))?;
    let post = observe_file(&path)?;
    inspect_existing_file(&path)?;
    if observation != post {
        return Err(reg(
            "INDEX_UNSTABLE",
            "derived index changed while it was read; stability scope is size+mtime, not file identity",
        ));
    }
    let (header, entries) = parse_index_bytes(
        &bytes,
        expected,
        ACCEPTED_METHODS_SHA256,
        ACCEPTED_METHODS_BYTES,
        ACCEPTED_METHOD_ROWS,
    )?;
    Ok(LoadedIndex {
        rel_path: rel.to_string(),
        sha256: crate::sha256_hex(&bytes),
        bytes: bytes.len() as u64,
        header,
        entries,
    })
}

fn find_index_entry<'a>(index: &'a LoadedIndex, target: &[u8; 32]) -> Option<&'a IndexEntry> {
    index
        .entries
        .binary_search_by_key(target, |entry| entry.method_id)
        .ok()
        .map(|position| &index.entries[position])
}

fn read_indexed_row(
    roots: &Roots,
    manifest: &SourceManifest,
    index: &LoadedIndex,
    target: &str,
) -> RegistryResult<Option<(GraphMethodRow, IndexEntry)>> {
    let Some(id) = hex32(target) else {
        return Err(reg(
            "METHOD_ID_INVALID",
            "target method id is not lower-hex",
        ));
    };
    let Some(entry) = find_index_entry(index, &id) else {
        return Ok(None);
    };
    let graph_path = roots
        .curated
        .join(&manifest.baseline.graph.path)
        .join("methods.jsonl");
    inspect_existing_file(&graph_path)?;
    let before = observe_file(&graph_path)?;
    if before.size != index.header.graph_methods_bytes {
        return Err(reg(
            "GRAPH_METHODS_SIZE_MISMATCH",
            "graph methods size differs from the accepted index header",
        ));
    }
    let file = File::open(&graph_path).map_err(|_| reg("GRAPH_METHODS_MISSING", "methods open"))?;
    let mut reader = std::io::BufReader::new(file);
    reader
        .seek(SeekFrom::Start(entry.offset))
        .map_err(|_| reg("GRAPH_METHODS_SEEK", "methods seek"))?;
    let mut line = vec![0_u8; entry.length as usize];
    reader
        .read_exact(&mut line)
        .map_err(|_| reg("GRAPH_METHODS_READ", "methods row read"))?;
    let digest = Sha256::digest(&line);
    let mut actual_row_sha = [0_u8; 32];
    actual_row_sha.copy_from_slice(&digest);
    if actual_row_sha != entry.row_sha256 {
        return Err(reg(
            "GRAPH_ROW_HASH_MISMATCH",
            "selected graph row bytes do not match the index row hash",
        ));
    }
    let json_bytes = line.strip_suffix(b"\n").unwrap_or(&line);
    let value = crate::parse_strict_value(json_bytes)
        .map_err(|_| reg("GRAPH_METHODS_INVALID", "selected row strict JSON"))?;
    let row: GraphMethodRow = serde_json::from_value(value)
        .map_err(|_| reg("GRAPH_METHODS_INVALID", "selected row shape"))?;
    validate_graph_row(&row)?;
    if !row.method_id.eq_ignore_ascii_case(target) {
        return Err(reg(
            "GRAPH_ROW_KEY_MISMATCH",
            "selected row full key differs from the requested method id",
        ));
    }
    let after = observe_file(&graph_path)?;
    inspect_existing_file(&graph_path)?;
    if before != after {
        return Err(reg(
            "GRAPH_METHODS_UNSTABLE",
            "graph methods changed during the selected-row read",
        ));
    }
    Ok(Some((row, entry.clone())))
}

fn token(value: Option<String>, label: &'static str, type_token: bool) -> RegistryResult<String> {
    let raw =
        value.ok_or_else(|| reg("LOCATE_ARGUMENT_MISSING", format!("{label} is required")))?;
    let upper = raw.trim().to_ascii_uppercase();
    let normalized = if let Some(rest) = upper.strip_prefix("0X") {
        format!("0x{rest}")
    } else {
        upper
    };
    let valid = if type_token {
        crate::validate_type_token(&normalized)
    } else {
        crate::validate_method_token(&normalized)
    };
    if !valid {
        return Err(reg(
            "LOCATE_ARGUMENT_INVALID",
            format!("{label} must be 0x02/0x06 plus eight hex characters"),
        ));
    }
    Ok(normalized)
}

#[derive(Debug, Clone, Default)]
struct PhysCache {
    map: BTreeMap<(String, String), (bool, Value)>,
    declared: BTreeMap<(String, String), (Option<u64>, String)>,
}

impl PhysCache {
    fn check(
        &mut self,
        roots: &Roots,
        root_id: &str,
        path: &str,
        declared_size: Option<u64>,
        declared_sha256: &str,
    ) -> RegistryResult<(bool, Value)> {
        let key = (root_id.to_string(), path.to_string());
        let merged_size = if let Some((known_size, known_sha)) = self.declared.get(&key) {
            if known_sha != declared_sha256
                || matches!(
                    (*known_size, declared_size),
                    (Some(left), Some(right)) if left != right
                )
            {
                return Err(reg(
                    "SOURCE_LOCATION_CONFLICT",
                    format!(
                        "same artifact path was declared with conflicting bytes: {root_id} {}",
                        path
                    ),
                ));
            }
            known_size.or(declared_size)
        } else {
            declared_size
        };
        self.declared
            .insert(key.clone(), (merged_size, declared_sha256.to_string()));
        if let Some((exists, value)) = self.map.get(&key) {
            return Ok((*exists, value.clone()));
        }
        let value =
            verify_current_artifact(roots, root_id, path, declared_size, Some(declared_sha256))?;
        self.map.insert(key, value.clone());
        Ok(value)
    }
}

#[derive(Debug, Clone)]
struct ViewDraft {
    kind: String,
    declared_coverage: Option<String>,
    evidence: Vec<String>,
    body_availability: String,
    provenance_status: Option<String>,
    limitations: Vec<String>,
    evidence_class: Option<String>,
    precision: Option<String>,
    body_sha256: Option<String>,
    body_hash_verified: Option<bool>,
    support_refs: Vec<Value>,
}

#[derive(Debug, Clone)]
struct LocationDraft {
    role: String,
    representation_class: String,
    declared_class: String,
    root_id: String,
    path: String,
    declared_size: Option<u64>,
    declared_sha256: String,
    range: Option<LineRange>,
    body_availability: String,
    evidence_level: String,
    provenance_status: String,
    scope: Option<String>,
    views: Vec<ViewDraft>,
    limitations: Vec<String>,
}

impl LocationDraft {
    fn key(&self) -> String {
        format!(
            "{}\0{}\0{}\0{}\0{}\0{}\0{}",
            self.role,
            self.representation_class,
            self.root_id,
            self.path,
            self.range
                .as_ref()
                .map(|range| range.start_line)
                .unwrap_or(0),
            self.range.as_ref().map(|range| range.end_line).unwrap_or(0),
            self.evidence_level
        )
    }

    fn source_id(&self) -> String {
        format!("SRC-{}", crate::sha256_hex(self.key().as_bytes()))
    }
}

#[derive(Default)]
struct Locator {
    locations: BTreeMap<String, LocationDraft>,
    conflicts: Vec<String>,
}

impl Locator {
    fn add(&mut self, draft: LocationDraft) {
        let key = draft.key();
        if let Some(existing) = self.locations.get_mut(&key) {
            if existing.declared_size != draft.declared_size
                || existing.declared_sha256 != draft.declared_sha256
            {
                self.conflicts.push(format!(
                    "conflicting declared bytes for {} {}",
                    draft.root_id, draft.path
                ));
            }
            for view in draft.views {
                if !existing.views.iter().any(|candidate| {
                    candidate.kind == view.kind
                        && candidate.declared_coverage == view.declared_coverage
                        && candidate.evidence == view.evidence
                        && candidate.precision == view.precision
                        && candidate.evidence_class == view.evidence_class
                }) {
                    existing.views.push(view);
                }
            }
            for limitation in draft.limitations {
                if !existing.limitations.contains(&limitation) {
                    existing.limitations.push(limitation);
                }
            }
            return;
        }
        self.locations.insert(key, draft);
    }

    fn has_graph_ref(&self, representation_id: &str, reference_role: &str) -> bool {
        self.locations.values().any(|location| {
            location.views.iter().any(|view| {
                view_graph_meta(view).is_some_and(|meta| {
                    meta.get("representation_id").and_then(Value::as_str) == Some(representation_id)
                        && meta.get("reference_role").and_then(Value::as_str)
                            == Some(reference_role)
                })
            })
        })
    }

    fn has_source_path(&self, root_id: &str, path: &str) -> bool {
        self.locations.values().any(|location| {
            location.root_id == root_id
                && location.path == path
                && location.views.iter().any(|view| {
                    view_graph_meta(view).is_some_and(|meta| {
                        matches!(
                            meta.get("reference_role").and_then(Value::as_str),
                            Some("source" | "semantic")
                        )
                    })
                })
        })
    }

    fn has_evidence_path(&self, root_id: &str, path: &str) -> bool {
        self.locations.values().any(|location| {
            location.root_id == root_id
                && location.path == path
                && location.views.iter().any(|view| {
                    view_graph_meta(view).is_some_and(|meta| {
                        matches!(
                            meta.get("reference_role").and_then(Value::as_str),
                            Some("raw" | "proof")
                        )
                    })
                })
        })
    }

    fn range_state(
        &self,
        draft: &LocationDraft,
        live: &Value,
    ) -> (
        Option<LineRange>,
        Option<LineRange>,
        Option<String>,
        Option<String>,
    ) {
        let Some(range) = &draft.range else {
            return (None, None, None, None);
        };
        let legacy = draft
            .views
            .iter()
            .any(|view| view_has_token(view, "RANGE:LEGACY_QUARANTINED_PATH"));
        let line_total = live.get("range_line_count").and_then(Value::as_u64);
        let (declared, validation) = match line_total {
            None => (
                Some(range.clone()),
                "SOURCE_UNREADABLE_FOR_RANGE".to_string(),
            ),
            Some(total)
                if range.start_line == 0
                    || range.end_line < range.start_line
                    || range.end_line > total =>
            {
                (Some(range.clone()), "OUT_OF_BOUNDS".to_string())
            }
            Some(_) if legacy => (Some(range.clone()), "LEGACY_QUARANTINED_PATH".to_string()),
            Some(_) if draft.role.starts_with("GRAPH_") => (
                Some(range.clone()),
                "DECLARED_HISTORICAL_NOT_VALIDATED".to_string(),
            ),
            Some(_) => (Some(range.clone()), "VALIDATED_CURRENT_RANGE".to_string()),
        };
        let navigation = if validation == "VALIDATED_CURRENT_RANGE" {
            declared.clone()
        } else {
            None
        };
        let basis = if draft.role.starts_with("GRAPH_") {
            Some("GRAPH_DECLARED_HISTORICAL".to_string())
        } else if validation == "VALIDATED_CURRENT_RANGE" {
            Some("CURRENT_TYPED_OR_SELECTOR_RANGE".to_string())
        } else {
            Some("DECLARED_ONLY".to_string())
        };
        (declared, navigation, Some(validation), basis)
    }

    fn finalize(
        &self,
        roots: &Roots,
        cache: &mut PhysCache,
    ) -> RegistryResult<(Vec<Value>, u64, u64)> {
        if let Some(conflict) = self.conflicts.first() {
            return Err(reg("SOURCE_LOCATION_CONFLICT", conflict.clone()));
        }
        let mut output = Vec::with_capacity(self.locations.len());
        let mut missing = 0_u64;
        let mut hash_verified = 0_u64;
        for draft in self.locations.values() {
            let (exists, live) = cache.check(
                roots,
                &draft.root_id,
                &draft.path,
                draft.declared_size,
                &draft.declared_sha256,
            )?;
            if !exists {
                missing += 1;
            }
            let (declared_range, navigation_range, range_validation, range_basis) =
                self.range_state(draft, &live);
            let declaration_role = draft.views.iter().find_map(|view| {
                view_graph_meta(view).and_then(|meta| {
                    meta.get("reference_role")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
            });
            let semantic_declaration = declaration_role.as_deref() == Some("semantic");
            let body_availability = if exists {
                draft.body_availability.clone()
            } else {
                "BODY_UNKNOWN".to_string()
            };
            if live.get("hash_checked").and_then(Value::as_bool) == Some(true)
                && live.get("hash_matches").and_then(Value::as_bool) == Some(true)
            {
                hash_verified += 1;
            }
            let mut views: Vec<Value> = draft
                .views
                .iter()
                .map(|view| {
                    let meta = view_graph_meta(view);
                    let body_presence = view
                        .evidence
                        .iter()
                        .find_map(|item| item.strip_prefix("BODY_PRESENCE:"))
                        .unwrap_or("NONE");
                    json!({
                        "kind": view.kind,
                        "representation_id": meta.as_ref().and_then(|value| value.get("representation_id")).cloned().unwrap_or(Value::Null),
                        "declared_kind": meta.as_ref().and_then(|value| value.get("representation_kind")).cloned().unwrap_or(Value::Null),
                        "reference_role": meta.as_ref().and_then(|value| value.get("reference_role")).cloned().unwrap_or(Value::Null),
                        "declared_coverage": view.declared_coverage,
                        "evidence": view.evidence,
                        "body_availability": view.body_availability,
                        "body_presence_evidence": body_presence,
                        "provenance_status": view.provenance_status,
                        "evidence_class": if meta.is_some() { Value::Null } else { json!(view.evidence_class) },
                        "precision": view.precision,
                        "body_sha256": view.body_sha256,
                        "declared_body_hash_verified": view.body_hash_verified,
                        "support_refs": view.support_refs,
                        "limitations": view.limitations,
                    })
                })
                .collect();
            views.sort_by(|left, right| {
                left["kind"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(right["kind"].as_str().unwrap_or(""))
            });
            output.push(json!({
                "source_id": draft.source_id(),
                "role": draft.role,
                "representation_class": if exists { draft.representation_class.clone() } else { "MISSING".to_string() },
                "declared_representation_class": draft.declared_class,
                "root_id": draft.root_id,
                "source_path": draft.path,
                "declared_size": draft.declared_size,
                "declared_sha256": draft.declared_sha256,
                "declared_source_range": if semantic_declaration { Value::Null } else { json!(declared_range.clone()) },
                "declared_semantic_range": if semantic_declaration { json!(declared_range.clone()) } else { Value::Null },
                "declared_range": declared_range,
                "navigation_range": navigation_range,
                "source_range": navigation_range,
                "range_validation": range_validation,
                "range_basis": range_basis,
                "body_availability": body_availability,
                "declared_body_availability": draft.body_availability,
                "evidence_level": draft.evidence_level,
                "provenance_status": draft.provenance_status,
                "scope": draft.scope,
                "live": live,
                "source_hash_verified": exists && live.get("hash_matches").and_then(Value::as_bool) == Some(true),
                "views": views,
                "limitations": if exists {
                    draft.limitations.clone()
                } else {
                    let mut limitations = draft.limitations.clone();
                    limitations.push(
                        "Declared artifact is missing; body availability is not claimed for live bytes."
                            .to_string(),
                    );
                    limitations
                },
            }));
        }
        output.sort_by(|left, right| {
            left["source_id"]
                .as_str()
                .unwrap_or("")
                .cmp(right["source_id"].as_str().unwrap_or(""))
        });
        Ok((output, missing, hash_verified))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssociationKind {
    Ordinary,
    GeneratedAssociation,
    AssociatedDeclaringType,
}

impl AssociationKind {
    fn label(self) -> &'static str {
        match self {
            Self::Ordinary => "ORDINARY",
            Self::GeneratedAssociation => "GENERATED_ASSOCIATION",
            Self::AssociatedDeclaringType => "ASSOCIATED_DECLARING_TYPE",
        }
    }
}

fn association_kind_for(
    record_id: &str,
    record_type_def: Option<&str>,
    identity: &MethodIdentity,
    coverage: &str,
    verification_basis: &str,
) -> AssociationKind {
    let coverage = coverage.to_ascii_uppercase();
    let generated = [
        "ASSOCIATION_ONLY",
        "GENERATED_ASSOCIATION",
        "ASYNC_MOVENEXT_ASSOCIATION",
        "ASSOCIATED_CLOSURE",
        "FRESH_FULL_CIL_CALLBACK",
        "GENERATED ASSOCIATION, NOT AN INDEPENDENT ORDINARY BODY",
        "GENERATED;",
    ]
    .iter()
    .any(|marker| coverage.contains(marker));
    if generated {
        return AssociationKind::GeneratedAssociation;
    }
    let _ = verification_basis;
    let declaring_mismatch = matches!(record_id, "PSCHED_GRAPH" | "PEXEC_SYSTEM" | "PEXEC_BASE")
        && record_type_def
            .map(|root| !root.eq_ignore_ascii_case(&identity.type_def))
            .unwrap_or(false);
    if declaring_mismatch {
        AssociationKind::AssociatedDeclaringType
    } else {
        AssociationKind::Ordinary
    }
}

fn association_kind(access: &CurrentMethodAccess) -> AssociationKind {
    association_kind_for(
        &access.record_id,
        access.record_type_def.as_deref(),
        &access.identity,
        &access.method_coverage,
        &access.method_verification_basis,
    )
}

fn association_basis(access: &CurrentMethodAccess) -> Value {
    let kind = association_kind(access);
    json!({
        "kind": kind.label(),
        "basis": match kind {
            AssociationKind::GeneratedAssociation => "ACCEPTED_METHOD_COVERAGE_GENERATED_ASSOCIATION",
            AssociationKind::AssociatedDeclaringType => "ACTUAL_TYPEDEF_DIFFERS_FROM_ACCEPTED_RECORD_TYPEDEF",
            AssociationKind::Ordinary => "ORDINARY_ACCEPTED_METHOD",
        },
        "record_id": access.record_id,
        "root_type_def": access.record_type_def,
        "actual_type_def": access.identity.type_def,
        "coverage": access.method_coverage,
        "independent_ordinary_body": kind == AssociationKind::Ordinary,
    })
}

fn body_status_for(evidence_status: &str, association: AssociationKind) -> &'static str {
    match (evidence_status, association) {
        ("METADATA_ONLY", _) => "BODY_ABSENT",
        ("DIRECT_CIL_FULL", AssociationKind::Ordinary) => "BODY_AVAILABLE",
        ("DIRECT_CIL_FULL", _) => "ASSOCIATED_OR_EMBEDDED",
        ("DIRECT_CIL_PARTIAL", AssociationKind::Ordinary) => "BODY_PARTIAL",
        ("DIRECT_CIL_PARTIAL", _) => "BODY_UNKNOWN",
        ("SOURCE_ONLY" | "UNKNOWN" | "BLOCKED", _) => "BODY_UNKNOWN",
        _ => "BODY_UNKNOWN",
    }
}

fn accepted_body_status(access: Option<&CurrentMethodAccess>) -> &'static str {
    access
        .map(|access| body_status_for(&access.method_evidence_status, association_kind(access)))
        .unwrap_or("BODY_UNKNOWN")
}

fn match_body_status(candidate: &CurrentMethodMatch) -> &'static str {
    body_status_for(
        &candidate.method_evidence_status,
        association_kind_for(
            &candidate.record_id,
            candidate.record_type_def.as_deref(),
            &candidate.identity,
            &candidate.method_coverage,
            &candidate.method_verification_basis,
        ),
    )
}

fn match_association_kind(candidate: &CurrentMethodMatch) -> AssociationKind {
    association_kind_for(
        &candidate.record_id,
        candidate.record_type_def.as_deref(),
        &candidate.identity,
        &candidate.method_coverage,
        &candidate.method_verification_basis,
    )
}

fn accepted_evidence_status(access: Option<&CurrentMethodAccess>) -> String {
    access
        .map(|access| access.method_evidence_status.clone())
        .unwrap_or_else(|| "UNRESOLVED".to_string())
}

fn provenance_status(access: Option<&CurrentMethodAccess>) -> String {
    access
        .and_then(|access| access.source_provenance.as_ref())
        .map(SourceProvenance::status_label)
        .unwrap_or("NO_CURRENT_PROVENANCE_RECORD")
        .to_string()
}

fn supplement_provenance_status(access: Option<&CurrentMethodAccess>) -> String {
    match access {
        Some(access) => provenance_status(Some(access)),
        None => "SOURCE_CLOSURE_ACCEPTED_SCOPED".to_string(),
    }
}

fn supplement_provenance_verification(access: Option<&CurrentMethodAccess>) -> &'static str {
    if access.is_some() {
        "CURRENT_EFFECTIVE_VIEW"
    } else {
        "NOT_RECHECKED_THIS_LOOKUP"
    }
}

fn source_provenance_value(access: Option<&CurrentMethodAccess>) -> Value {
    match access.and_then(|access| access.source_provenance.as_ref()) {
        Some(SourceProvenance::ProfileEntry {
            status,
            action,
            imported,
            evidence_anchors,
            legacy_original_references,
        }) => json!({
            "status": status.as_str(),
            "action": action.as_str(),
            "imported_anchor": imported,
            "evidence_anchors": evidence_anchors,
            "legacy_original_references": legacy_original_references,
            "legacy_anchor_reactivated": false
        }),
        Some(SourceProvenance::NotValidated) => json!({
            "status": "NOT_VALIDATED_BY_CURRENT_PROVENANCE_PROFILE"
        }),
        None => json!({"status": "NO_CURRENT_PROVENANCE_RECORD"}),
    }
}

fn current_authority_value(access: Option<&CurrentMethodAccess>) -> Value {
    let Some(access) = access else {
        return Value::Null;
    };
    json!({
        "record_id": access.record_id,
        "original_path": access.record_original_path,
        "original_size": access.record_original_size,
        "original_sha256": access.record_original_sha256,
        "proposed_path": access.record_proposed_path,
        "record_authority_id": access.record_authority_id,
        "record_authority_sha256": access.record_authority_sha256,
        "record_authority_mvid": access.record_authority_mvid,
        "record_type_def": access.record_type_def,
        "record_scope": access.record_scope,
        "record_completeness": access.record_completeness,
        "record_representation": access.record_representation,
        "record_curation_status": access.record_curation_status,
        "record_export_status": access.record_export_status,
        "record_evidence_status": access.record_evidence_status,
        "record_source_status": access.record_source_status,
        "record_supersedes": access.record_supersedes,
        "record_superseded_by": access.record_superseded_by,
        "method_coverage": access.method_coverage,
        "method_evidence_status": access.method_evidence_status,
        "method_source_status": access.method_source_status,
        "method_verification_basis": access.method_verification_basis,
        "method_rva": access.method_rva,
        "method_il_range": access.method_il_range,
        "association": association_basis(access),
        "body_availability": accepted_body_status(Some(access)),
        "current_match_count": access.current_matches.len(),
        "historical_match_count": access.exact_historical_matches.len(),
        "method_evidence_references": access.method_evidence_references.iter().map(|reference| json!({
            "path": reference.path,
            "locator": reference.locator,
            "artifact_sha256": reference.artifact_sha256,
        })).collect::<Vec<_>>(),
        "source_provenance": source_provenance_value(Some(access)),
        "imported_file": access.imported_file.as_ref().map(|file| json!({
            "id": file.id,
            "original_path": file.original_path,
            "imported": {
                "path": file.imported.path,
                "size": file.imported.size,
                "sha256": file.imported.sha256,
            },
            "method_count": file.methods.len(),
        })),
        "historical_relations": access.historical.iter().map(|method| json!({
            "symbol_key": method.symbol_key,
            "name": method.name,
            "coverage": method.coverage,
            "exact_fullkey_match": &method.symbol_key == &access.identity,
            "relation": "ACCEPTED_RECORD_HISTORICAL",
        })).collect::<Vec<_>>(),
        "historical_alternatives_fabricated": false,
    })
}

fn roles_for_path(locations: &[Value], root_id: &str, path: &str) -> Vec<String> {
    locations
        .iter()
        .filter(|location| location["root_id"] == root_id && location["source_path"] == path)
        .filter_map(|location| location["role"].as_str().map(str::to_string))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn exact_source_views_value(access: Option<&CurrentMethodAccess>, locations: &[Value]) -> Value {
    let Some(access) = access else {
        return json!({
            "current": [],
            "historical": [],
            "unrepresented": []
        });
    };
    let mut current = Vec::new();
    let mut unrepresented = Vec::new();
    for candidate in &access.current_matches {
        let class = if candidate.is_active_primary {
            "PRIMARY_CURATED"
        } else {
            alternative_class(access, candidate)
        };
        let mut source_paths = Vec::new();
        if let Some(file) = &candidate.imported_file {
            source_paths.push(json!({
                "root_id": "CURATED_ROOT",
                "path": file.imported.path,
                "size": file.imported.size,
                "sha256": file.imported.sha256,
                "class": class,
                "roles": roles_for_path(locations, "CURATED_ROOT", &file.imported.path),
            }));
        }
        source_paths.push(json!({
            "root_id": "RESEARCH_ROOT",
            "path": candidate.record_original_path,
            "size": candidate.record_original_size,
            "sha256": candidate.record_original_sha256,
            "class": if candidate.is_active_primary { "SUPPLEMENTARY" } else { class },
            "roles": roles_for_path(locations, "RESEARCH_ROOT", &candidate.record_original_path),
        }));
        let represented = source_paths.iter().any(|source| {
            source["roles"]
                .as_array()
                .map(|roles| !roles.is_empty())
                .unwrap_or(false)
        });
        let value = json!({
            "record_id": candidate.record_id,
            "method_id": method_id(
                &candidate.identity.full_authority_sha256,
                &candidate.identity.mvid,
                &candidate.identity.type_def,
                &candidate.identity.method_def,
            ),
            "symbol_key": candidate.identity,
            "is_active_primary": candidate.is_active_primary,
            "active_scope": candidate.active_scope,
            "class": class,
            "method_evidence_status": candidate.method_evidence_status,
            "accepted_current_class": if candidate.is_active_primary && candidate.active_scope {
                Value::String(candidate.method_evidence_status.clone())
            } else {
                Value::Null
            },
            "alternative_status": candidate.method_evidence_status,
            "association_kind": match_association_kind(candidate).label(),
            "body_availability": match_body_status(candidate),
            "record": {
                "proposed_path": candidate.record_proposed_path,
                "authority_id": candidate.record_authority_id,
                "authority_sha256": candidate.record_authority_sha256,
                "authority_mvid": candidate.record_authority_mvid,
                "type_def": candidate.record_type_def,
                "scope": candidate.record_scope,
                "completeness": candidate.record_completeness,
                "representation": candidate.record_representation,
                "curation_status": candidate.record_curation_status,
                "export_status": candidate.record_export_status,
                "evidence_status": candidate.record_evidence_status,
                "source_status": candidate.record_source_status,
                "supersedes": candidate.record_supersedes,
                "superseded_by": candidate.record_superseded_by,
            },
            "method": {
                "coverage": candidate.method_coverage,
                "source_status": candidate.method_source_status,
                "verification_basis": candidate.method_verification_basis,
                "rva": candidate.method_rva,
                "il_range": candidate.method_il_range,
                "evidence_references": candidate.method_evidence_references.iter().map(|reference| json!({
                    "path": reference.path,
                    "locator": reference.locator,
                    "artifact_sha256": reference.artifact_sha256,
                })).collect::<Vec<_>>(),
            },
            "source_paths": source_paths,
            "represented": represented,
        });
        if !represented {
            unrepresented.push(json!({
                "record_id": candidate.record_id,
                "reason": "NO_LOCATION_EMITTED",
            }));
        }
        current.push(value);
    }
    let mut historical = Vec::new();
    for candidate in &access.exact_historical_matches {
        let roles = roles_for_path(locations, "RESEARCH_ROOT", &candidate.record_original_path);
        let represented = candidate.has_current_exact_match || !roles.is_empty();
        let value = json!({
            "record_id": candidate.record_id,
            "method_id": method_id(
                &candidate.symbol_key.full_authority_sha256,
                &candidate.symbol_key.mvid,
                &candidate.symbol_key.type_def,
                &candidate.symbol_key.method_def,
            ),
            "symbol_key": candidate.symbol_key,
            "name": candidate.name,
            "coverage": candidate.coverage,
            "class": "HISTORICAL_ALTERNATIVE",
            "source_path": candidate.record_original_path,
            "source_size": candidate.record_original_size,
            "source_sha256": candidate.record_original_sha256,
            "record_curation_status": candidate.record_curation_status,
            "record": {
                "curation_status": candidate.record_curation_status,
                "export_status": candidate.record_export_status,
                "evidence_status": candidate.record_evidence_status,
                "source_status": candidate.record_source_status,
                "supersedes": candidate.record_supersedes,
                "superseded_by": candidate.record_superseded_by,
                "type_def": candidate.record_type_def,
            },
            "imported_file": candidate.imported_file.as_ref().map(|file| json!({
                "id": file.id,
                "imported_path": file.imported.path,
                "imported_size": file.imported.size,
                "imported_sha256": file.imported.sha256,
            })),
            "same_record_as_primary": candidate.same_record_as_primary,
            "has_current_exact_match": candidate.has_current_exact_match,
            "represented": represented,
            "represented_by": if candidate.has_current_exact_match {
                "CURRENT_MATCH_LOCATION"
            } else {
                "EXACT_HISTORICAL_FULLKEY_SOURCE"
            },
            "roles": roles,
        });
        if !represented {
            unrepresented.push(json!({
                "record_id": candidate.record_id,
                "method_id": method_id(
                    &candidate.symbol_key.full_authority_sha256,
                    &candidate.symbol_key.mvid,
                    &candidate.symbol_key.type_def,
                    &candidate.symbol_key.method_def,
                ),
                "reason": "HISTORICAL_SOURCE_LOCATION_NOT_EMITTED",
            }));
        }
        historical.push(value);
    }
    json!({
        "current": current,
        "historical": historical,
        "current_match_count": access.current_matches.len(),
        "historical_match_count": access.exact_historical_matches.len(),
        "represented_current": current.iter().filter(|row| row["represented"] == true).count(),
        "represented_historical": historical.iter().filter(|row| row["represented"] == true).count(),
        "unrepresented": unrepresented,
    })
}

fn add_active_locations(locator: &mut Locator, access: Option<&CurrentMethodAccess>) {
    let Some(access) = access else {
        return;
    };
    let body = accepted_body_status(Some(access));
    let association = association_kind(access);
    let evidence = access.method_evidence_status.clone();
    let provenance = provenance_status(Some(access));
    if let Some(file) = &access.imported_file {
        locator.add(LocationDraft {
            role: "PRIMARY_CURATED_IMPORT".to_string(),
            representation_class: "PRIMARY_CURATED".to_string(),
            declared_class: "PRIMARY_CURATED".to_string(),
            root_id: "CURATED_ROOT".to_string(),
            path: file.imported.path.clone(),
            declared_size: Some(file.imported.size),
            declared_sha256: file.imported.sha256.clone(),
            range: None,
            body_availability: body.to_string(),
            evidence_level: evidence.clone(),
            provenance_status: provenance.clone(),
            scope: Some("CURRENT_ACCEPTED_IMPORT".to_string()),
            views: vec![ViewDraft {
                kind: "ACCEPTED_CURRENT_IMPORT".to_string(),
                declared_coverage: Some(access.method_coverage.clone()),
                evidence: vec![access.method_evidence_status.clone()],
                body_availability: body.to_string(),
                provenance_status: Some(provenance.clone()),
                limitations: vec![
                    "Whole-file hash verifies bytes; method semantics are not checked.".to_string(),
                ],
                evidence_class: Some("ACCEPTED_CURRENT_METHOD".to_string()),
                precision: Some("FILE_METHOD_ASSOCIATION".to_string()),
                body_sha256: None,
                body_hash_verified: None,
                support_refs: Vec::new(),
            }],
            limitations: vec![
                "Accepted evidence status is authoritative only for this representation."
                    .to_string(),
                "No whole-class or unselected-member body is claimed by this method location."
                    .to_string(),
                format!(
                    "Accepted association class {}; evidence class remains {}.",
                    association.label(),
                    access.method_evidence_status
                ),
            ],
        });
    }
    locator.add(LocationDraft {
        role: "CURRENT_ACCEPTED_ORIGINAL".to_string(),
        representation_class: "SUPPLEMENTARY".to_string(),
        declared_class: "SUPPLEMENTARY".to_string(),
        root_id: "RESEARCH_ROOT".to_string(),
        path: access.record_original_path.clone(),
        declared_size: Some(access.record_original_size),
        declared_sha256: access.record_original_sha256.clone(),
        range: None,
        body_availability: body.to_string(),
        evidence_level: evidence.clone(),
        provenance_status: provenance.clone(),
        scope: Some("CURRENT_ACCEPTED_ORIGINAL_RESEARCH_PATH".to_string()),
        views: vec![ViewDraft {
            kind: "ACCEPTED_CURRENT_ORIGINAL".to_string(),
            declared_coverage: Some(access.method_coverage.clone()),
            evidence: vec![access.method_evidence_status.clone()],
            body_availability: body.to_string(),
            provenance_status: Some(provenance.clone()),
            limitations: vec![
                "Record-level original research path; it is not a second accepted import."
                    .to_string(),
            ],
            evidence_class: Some("ACCEPTED_CURRENT_ORIGINAL".to_string()),
            precision: Some("FILE_METHOD_ASSOCIATION".to_string()),
            body_sha256: None,
            body_hash_verified: None,
            support_refs: Vec::new(),
        }],
        limitations: vec![
            "Original-path bytes are checked only when the path is present.".to_string(),
        ],
    });
    if let Some(SourceProvenance::ProfileEntry {
        imported,
        evidence_anchors,
        ..
    }) = &access.source_provenance
    {
        let imported_root = imported.root.as_str();
        let imported_body = if imported.precision == Precision::ExactRange {
            body
        } else {
            "BODY_UNKNOWN"
        };
        locator.add(LocationDraft {
            role: "CURRENT_TYPED_IMPORTED_ANCHOR".to_string(),
            representation_class: if imported_root == "CURATED_ROOT" {
                "PRIMARY_CURATED".to_string()
            } else {
                "SUPPLEMENTARY".to_string()
            },
            declared_class: "PRIMARY_CURATED".to_string(),
            root_id: imported_root.to_string(),
            path: imported.path.clone(),
            declared_size: None,
            declared_sha256: imported.whole_file_sha256.clone(),
            range: imported
                .start_line
                .zip(imported.end_line)
                .map(|(start, end)| LineRange {
                    start_line: start,
                    end_line: end,
                }),
            body_availability: imported_body.to_string(),
            evidence_level: format!("TYPED_IMPORTED_ANCHOR_{}", imported.precision.as_str()),
            provenance_status: provenance.clone(),
            scope: Some(imported.scope.clone()),
            views: vec![ViewDraft {
                kind: "CURRENT_TYPED_IMPORTED_ANCHOR".to_string(),
                declared_coverage: None,
                evidence: {
                    let mut evidence = vec!["TYPED_CURRENT_PROVENANCE".to_string()];
                    if imported.start_line.is_some() && imported.end_line.is_some() {
                        evidence.push("RANGE:CURRENT_TYPED".to_string());
                    }
                    evidence
                },
                body_availability: imported_body.to_string(),
                provenance_status: Some(provenance.clone()),
                limitations: vec![
                    "Typed errata anchor; no numeric legacy anchor is reactivated.".to_string(),
                ],
                evidence_class: None,
                precision: Some(imported.precision.as_str().to_string()),
                body_sha256: None,
                body_hash_verified: None,
                support_refs: imported
                    .support_refs
                    .iter()
                    .filter_map(|support| serde_json::to_value(support).ok())
                    .collect(),
            }],
            limitations: vec![
                "Anchor precision is retained; it does not promote the accepted method class."
                    .to_string(),
            ],
        });
        for anchor in evidence_anchors {
            let evidence_class = anchor.evidence_class.as_str();
            let (class, body, scope) = match anchor.evidence_class {
                EvidenceClass::AssociatedSource => (
                    "SUPPLEMENTARY",
                    "ASSOCIATED_OR_EMBEDDED",
                    Some("ASSOCIATED_SOURCE"),
                ),
                EvidenceClass::PerformanceSource => (
                    "SUPPLEMENTARY",
                    "BODY_UNKNOWN",
                    Some("PERFORMANCE_EVIDENCE_SOURCE"),
                ),
                EvidenceClass::SemanticSource | EvidenceClass::CanonicalSource => (
                    "SUPPLEMENTARY",
                    "BODY_UNKNOWN",
                    Some("SEMANTIC_OR_CANONICAL_EVIDENCE_SOURCE"),
                ),
                _ => (
                    "EVIDENCE_ONLY",
                    "NOT_APPLICABLE",
                    Some("EVIDENCE_ONLY_ANCHOR"),
                ),
            };
            locator.add(LocationDraft {
                role: "CURRENT_TYPED_EVIDENCE_ANCHOR".to_string(),
                representation_class: class.to_string(),
                declared_class: class.to_string(),
                root_id: anchor.root.as_str().to_string(),
                path: anchor.path.clone(),
                declared_size: None,
                declared_sha256: anchor.artifact_sha256.clone(),
                range: anchor.start_line.zip(anchor.end_line).map(|(start, end)| LineRange {
                    start_line: start,
                    end_line: end,
                }),
                body_availability: body.to_string(),
                evidence_level: format!("TYPED_EVIDENCE_ANCHOR_{evidence_class}"),
                provenance_status: provenance.clone(),
                scope: scope.map(str::to_string),
                views: vec![ViewDraft {
                    kind: "CURRENT_TYPED_EVIDENCE_ANCHOR".to_string(),
                    declared_coverage: None,
                    evidence: vec![evidence_class.to_string()],
                    body_availability: body.to_string(),
                    provenance_status: Some(provenance.clone()),
                    limitations: vec![
                        "Typed evidence anchor; raw/proof/evidence bytes do not claim a source body."
                            .to_string(),
                    ],
                    evidence_class: Some(evidence_class.to_string()),
                    precision: Some(anchor.precision.as_str().to_string()),
                    body_sha256: None,
                    body_hash_verified: None,
                    support_refs: anchor
                        .support_refs
                        .iter()
                        .filter_map(|support| serde_json::to_value(support).ok())
                        .collect(),
                }],
                limitations: vec![
                    "Evidence anchors are source-vs-evidence roles, not merged method bodies."
                        .to_string(),
                ],
            });
        }
    }
    for candidate in &access.current_matches {
        if !candidate.is_active_primary {
            add_alternative_current_locations(locator, access, candidate);
        }
    }
    add_historical_only_locations(locator, access);
}

fn match_provenance_status(candidate: &CurrentMethodMatch) -> String {
    candidate
        .source_provenance
        .as_ref()
        .map(SourceProvenance::status_label)
        .unwrap_or("NO_CURRENT_PROVENANCE_RECORD")
        .to_string()
}

fn has_exact_historical(access: &CurrentMethodAccess, record_id: &str) -> bool {
    access
        .exact_historical_matches
        .iter()
        .any(|candidate| candidate.record_id == record_id)
}

fn alternative_class(access: &CurrentMethodAccess, candidate: &CurrentMethodMatch) -> &'static str {
    if candidate.record_curation_status == "HISTORICAL_ONLY"
        || !candidate.record_superseded_by.is_empty()
        || has_exact_historical(access, &candidate.record_id)
    {
        "HISTORICAL_ALTERNATIVE"
    } else {
        "SUPPLEMENTARY"
    }
}

fn add_alternative_current_locations(
    locator: &mut Locator,
    access: &CurrentMethodAccess,
    candidate: &CurrentMethodMatch,
) {
    let class = alternative_class(access, candidate);
    let body = match_body_status(candidate);
    let association = match_association_kind(candidate);
    let evidence = candidate.method_evidence_status.clone();
    let provenance = match_provenance_status(candidate);
    if let Some(file) = &candidate.imported_file {
        locator.add(LocationDraft {
            role: "ALTERNATIVE_CURRENT_IMPORT".to_string(),
            representation_class: class.to_string(),
            declared_class: class.to_string(),
            root_id: "CURATED_ROOT".to_string(),
            path: file.imported.path.clone(),
            declared_size: Some(file.imported.size),
            declared_sha256: file.imported.sha256.clone(),
            range: None,
            body_availability: body.to_string(),
            evidence_level: evidence.clone(),
            provenance_status: provenance.clone(),
            scope: Some("ALTERNATIVE_DECLARED_RECORD_NOT_ACTIVE_PRIMARY".to_string()),
            views: vec![ViewDraft {
                kind: "ALTERNATIVE_CURRENT_IMPORT".to_string(),
                declared_coverage: Some(candidate.method_coverage.clone()),
                evidence: vec![candidate.method_evidence_status.clone()],
                body_availability: body.to_string(),
                provenance_status: Some(provenance.clone()),
                limitations: vec![
                    "Declared alternative current record; it is not the uniquely selected active primary."
                        .to_string(),
                ],
                evidence_class: Some("ALTERNATIVE_CURRENT_METHOD".to_string()),
                precision: Some("FILE_METHOD_ASSOCIATION".to_string()),
                body_sha256: None,
                body_hash_verified: None,
                support_refs: Vec::new(),
            }],
            limitations: vec![
                format!(
                    "Alternative record {} keeps its own evidence class {} and association {}.",
                    candidate.record_id,
                    candidate.method_evidence_status,
                    association.label()
                ),
            ],
        });
    }
    locator.add(LocationDraft {
        role: "ALTERNATIVE_CURRENT_ORIGINAL".to_string(),
        representation_class: class.to_string(),
        declared_class: class.to_string(),
        root_id: "RESEARCH_ROOT".to_string(),
        path: candidate.record_original_path.clone(),
        declared_size: Some(candidate.record_original_size),
        declared_sha256: candidate.record_original_sha256.clone(),
        range: None,
        body_availability: body.to_string(),
        evidence_level: evidence.clone(),
        provenance_status: provenance.clone(),
        scope: Some("ALTERNATIVE_DECLARED_RECORD_SOURCE".to_string()),
        views: vec![ViewDraft {
            kind: "ALTERNATIVE_CURRENT_ORIGINAL".to_string(),
            declared_coverage: Some(candidate.method_coverage.clone()),
            evidence: vec![candidate.method_evidence_status.clone()],
            body_availability: body.to_string(),
            provenance_status: Some(provenance.clone()),
            limitations: vec![
                "Record-declared source path; no historical path is invented.".to_string(),
            ],
            evidence_class: Some("ALTERNATIVE_CURRENT_METHOD".to_string()),
            precision: Some("FILE_METHOD_ASSOCIATION".to_string()),
            body_sha256: None,
            body_hash_verified: None,
            support_refs: Vec::new(),
        }],
        limitations: vec![
            "Alternative source is accounted separately from the active primary record."
                .to_string(),
        ],
    });
    if let Some(SourceProvenance::ProfileEntry {
        imported,
        evidence_anchors,
        ..
    }) = &candidate.source_provenance
    {
        let imported_body = if imported.precision == Precision::ExactRange {
            body
        } else {
            "BODY_UNKNOWN"
        };
        locator.add(LocationDraft {
            role: "ALTERNATIVE_TYPED_IMPORTED_ANCHOR".to_string(),
            representation_class: class.to_string(),
            declared_class: class.to_string(),
            root_id: imported.root.as_str().to_string(),
            path: imported.path.clone(),
            declared_size: None,
            declared_sha256: imported.whole_file_sha256.clone(),
            range: imported
                .start_line
                .zip(imported.end_line)
                .map(|(start, end)| LineRange {
                    start_line: start,
                    end_line: end,
                }),
            body_availability: imported_body.to_string(),
            evidence_level: format!(
                "ALTERNATIVE_TYPED_IMPORTED_ANCHOR_{}",
                imported.precision.as_str()
            ),
            provenance_status: provenance.clone(),
            scope: Some(imported.scope.clone()),
            views: vec![ViewDraft {
                kind: "ALTERNATIVE_TYPED_IMPORTED_ANCHOR".to_string(),
                declared_coverage: None,
                evidence: {
                    let mut evidence = vec!["ALTERNATIVE_TYPED_CURRENT_PROVENANCE".to_string()];
                    if imported.start_line.is_some() && imported.end_line.is_some() {
                        evidence.push("RANGE:CURRENT_TYPED".to_string());
                    }
                    evidence
                },
                body_availability: imported_body.to_string(),
                provenance_status: Some(provenance.clone()),
                limitations: vec![
                    "Typed alternative anchor; legacy numeric anchors are not reactivated."
                        .to_string(),
                ],
                evidence_class: None,
                precision: Some(imported.precision.as_str().to_string()),
                body_sha256: None,
                body_hash_verified: None,
                support_refs: imported
                    .support_refs
                    .iter()
                    .filter_map(|support| serde_json::to_value(support).ok())
                    .collect(),
            }],
            limitations: vec![
                "Alternative anchor does not change the active primary classification.".to_string(),
            ],
        });
        for anchor in evidence_anchors {
            let evidence_class = anchor.evidence_class.as_str();
            let (anchor_body, scope) = match anchor.evidence_class {
                EvidenceClass::AssociatedSource => ("ASSOCIATED_OR_EMBEDDED", "ASSOCIATED_SOURCE"),
                EvidenceClass::PerformanceSource => ("BODY_UNKNOWN", "PERFORMANCE_EVIDENCE_SOURCE"),
                EvidenceClass::SemanticSource | EvidenceClass::CanonicalSource => {
                    ("BODY_UNKNOWN", "SEMANTIC_OR_CANONICAL_EVIDENCE_SOURCE")
                }
                _ => ("NOT_APPLICABLE", "EVIDENCE_ONLY_ANCHOR"),
            };
            let anchor_class = match anchor.evidence_class {
                EvidenceClass::AssociatedSource
                | EvidenceClass::PerformanceSource
                | EvidenceClass::SemanticSource
                | EvidenceClass::CanonicalSource => class,
                _ => "EVIDENCE_ONLY",
            };
            locator.add(LocationDraft {
                role: "ALTERNATIVE_TYPED_EVIDENCE_ANCHOR".to_string(),
                representation_class: anchor_class.to_string(),
                declared_class: anchor_class.to_string(),
                root_id: anchor.root.as_str().to_string(),
                path: anchor.path.clone(),
                declared_size: None,
                declared_sha256: anchor.artifact_sha256.clone(),
                range: anchor
                    .start_line
                    .zip(anchor.end_line)
                    .map(|(start, end)| LineRange {
                        start_line: start,
                        end_line: end,
                    }),
                body_availability: anchor_body.to_string(),
                evidence_level: format!("ALTERNATIVE_TYPED_EVIDENCE_ANCHOR_{evidence_class}"),
                provenance_status: provenance.clone(),
                scope: Some(scope.to_string()),
                views: vec![ViewDraft {
                    kind: "ALTERNATIVE_TYPED_EVIDENCE_ANCHOR".to_string(),
                    declared_coverage: None,
                    evidence: vec![evidence_class.to_string()],
                    body_availability: anchor_body.to_string(),
                    provenance_status: Some(provenance.clone()),
                    limitations: vec![
                        "Alternative evidence anchor; raw/proof bytes do not claim a source body."
                            .to_string(),
                    ],
                    evidence_class: Some(evidence_class.to_string()),
                    precision: Some(anchor.precision.as_str().to_string()),
                    body_sha256: None,
                    body_hash_verified: None,
                    support_refs: anchor
                        .support_refs
                        .iter()
                        .filter_map(|support| serde_json::to_value(support).ok())
                        .collect(),
                }],
                limitations: vec![
                    "Alternative source-vs-evidence roles remain separate.".to_string(),
                ],
            });
        }
    }
}

fn add_historical_only_locations(locator: &mut Locator, access: &CurrentMethodAccess) {
    for historical in &access.exact_historical_matches {
        if historical.has_current_exact_match {
            continue;
        }
        locator.add(LocationDraft {
            role: "EXACT_HISTORICAL_FULLKEY_SOURCE".to_string(),
            representation_class: "HISTORICAL_ALTERNATIVE".to_string(),
            declared_class: "HISTORICAL_ALTERNATIVE".to_string(),
            root_id: "RESEARCH_ROOT".to_string(),
            path: historical.record_original_path.clone(),
            declared_size: Some(historical.record_original_size),
            declared_sha256: historical.record_original_sha256.clone(),
            range: None,
            body_availability: "BODY_UNKNOWN".to_string(),
            evidence_level: format!("HISTORICAL_{}", historical.coverage),
            provenance_status: "HISTORICAL_RELATION_ONLY".to_string(),
            scope: Some("EXACT_HISTORICAL_FULLKEY_RECORD_SOURCE".to_string()),
            views: vec![ViewDraft {
                kind: "EXACT_HISTORICAL_FULLKEY_SOURCE".to_string(),
                declared_coverage: Some(historical.coverage.clone()),
                evidence: vec!["ACCEPTED_RECORD_HISTORICAL_FULLKEY".to_string()],
                body_availability: "BODY_UNKNOWN".to_string(),
                provenance_status: Some("HISTORICAL_RELATION_ONLY".to_string()),
                limitations: vec![
                    "Exact historical fullkey is bound to this record source; no historical path was invented."
                        .to_string(),
                ],
                evidence_class: Some("HISTORICAL_FULLKEY".to_string()),
                precision: Some("FILE_METHOD_ASSOCIATION".to_string()),
                body_sha256: None,
                body_hash_verified: None,
                support_refs: Vec::new(),
            }],
            limitations: vec![
                format!(
                    "Historical record {} is a relationship/source view, not an independent current body.",
                    historical.record_id
                ),
            ],
        });
    }
}

fn supplement_info<'a>(
    manifest: &'a SourceManifest,
    target: &str,
) -> Option<(&'a SourceImport, &'a MemberSelector)> {
    find_import_member(manifest, target)
}

fn supplement_value(manifest: &SourceManifest, target: &str) -> Option<Value> {
    let (import, member) = supplement_info(manifest, target)?;
    Some(json!({
        "import_id": import.id,
        "origin_id": import.origin_id,
        "mode": import.mode,
        "source": import.source,
        "destination": import.destination,
        "member": member,
        "function_views": import.function_views,
        "provenance_status": supplement_provenance_status(None),
        "provenance_verification": "NOT_RECHECKED_THIS_LOOKUP",
        "accepted_scoped_proof_selector": true,
        "associated_callback_has_independent_body": false
    }))
}

fn add_supplement_locations(
    locator: &mut Locator,
    manifest: &SourceManifest,
    target: &str,
    access: Option<&CurrentMethodAccess>,
) {
    let Some((import, member)) = supplement_info(manifest, target) else {
        return;
    };
    let associated = member.role == "ASSOCIATED";
    let body = if associated {
        "ASSOCIATED_OR_EMBEDDED"
    } else {
        "BODY_AVAILABLE"
    };
    let provenance = supplement_provenance_status(access);
    locator.add(LocationDraft {
        role: if associated {
            "SUPPLEMENT_ASSOCIATED_DESTINATION".to_string()
        } else {
            "SUPPLEMENT_MEMBER_DESTINATION".to_string()
        },
        representation_class: "PRIMARY_CURATED".to_string(),
        declared_class: "PRIMARY_CURATED".to_string(),
        root_id: import.destination.root_id.clone(),
        path: import.destination.path.clone(),
        declared_size: Some(import.destination.size),
        declared_sha256: import.destination.sha256.clone(),
        range: member.output_lines.clone(),
        body_availability: body.to_string(),
        evidence_level: if associated {
            "ASSOCIATED_OR_EMBEDDED".to_string()
        } else {
            "DIRECT_CIL_FULL".to_string()
        },
        provenance_status: provenance.clone(),
        scope: Some(format!("SUPPLEMENT_MEMBER_{}", member.role)),
        views: vec![ViewDraft {
            kind: "SUPPLEMENT_MEMBER_SELECTOR".to_string(),
            declared_coverage: Some(member.scope.clone()),
            evidence: {
                let mut evidence = vec![
                    "DETERMINISTIC_LINE_SELECTION".to_string(),
                    member.representation.clone(),
                ];
                if member.output_lines.is_some() {
                    evidence.push("RANGE:SUPPLEMENT_SELECTOR".to_string());
                }
                evidence
            },
            body_availability: body.to_string(),
            provenance_status: Some(provenance.clone()),
            limitations: if associated {
                vec![
                    "Associated callback is embedded in the selected destination; it has no independent body."
                        .to_string(),
                ]
            } else {
                vec![
                    "Selected output range; whole-file hash does not prove unselected bytes."
                        .to_string(),
                ]
            },
            evidence_class: Some(member.role.clone()),
            precision: Some("EXACT_RANGE".to_string()),
            body_sha256: None,
            body_hash_verified: None,
            support_refs: Vec::new(),
        }],
        limitations: vec![
            "Supplement member selectors are the accepted four-key finite set.".to_string(),
            format!(
                "Supplement provenance status {} ({}); accepted scoped proof selector, not old-533 errata.",
                provenance,
                supplement_provenance_verification(access)
            ),
        ],
    });
    locator.add(LocationDraft {
        role: "SUPPLEMENT_SOURCE_SELECTION".to_string(),
        representation_class: "SUPPLEMENTARY".to_string(),
        declared_class: "SUPPLEMENTARY".to_string(),
        root_id: import.source.root_id.clone(),
        path: import.source.path.clone(),
        declared_size: Some(import.source.size),
        declared_sha256: import.source.sha256.clone(),
        range: member.source_input_lines.clone(),
        body_availability: body.to_string(),
        evidence_level: "DETERMINISTIC_LINE_SELECTION".to_string(),
        provenance_status: provenance.clone(),
        scope: Some("SUPPLEMENT_SOURCE_INPUT_SELECTION".to_string()),
        views: vec![ViewDraft {
            kind: "SUPPLEMENT_SOURCE_SELECTION".to_string(),
            declared_coverage: Some(member.scope.clone()),
            evidence: {
                let mut evidence = vec!["DETERMINISTIC_LINE_SELECTION".to_string()];
                if member.source_input_lines.is_some() {
                    evidence.push("RANGE:SUPPLEMENT_SELECTOR".to_string());
                }
                evidence
            },
            body_availability: body.to_string(),
            provenance_status: Some(provenance_status(access)),
            limitations: if associated {
                vec![
                    "Source selector is associated with the embedded callback; no independent callback body is claimed."
                        .to_string(),
                ]
            } else {
                vec![
                    "Selected source range; direct-CIL equivalence is not re-proved by this lookup."
                        .to_string(),
                ]
            },
            evidence_class: Some(member.role.clone()),
            precision: Some("EXACT_RANGE".to_string()),
            body_sha256: None,
            body_hash_verified: None,
            support_refs: Vec::new(),
        }],
        limitations: vec![
            "Source selection is not a second accepted import.".to_string(),
            format!(
                "Supplement provenance status {} ({}); accepted scoped proof selector, not old-533 errata.",
                provenance,
                supplement_provenance_verification(access)
            ),
        ],
    });
}

fn graph_view_meta(rep: &GraphRepresentation, reference_role: &str) -> String {
    json!({
        "representation_id": rep.representation_id,
        "representation_kind": rep.representation_kind,
        "reference_role": reference_role,
    })
    .to_string()
}

fn view_graph_meta(view: &ViewDraft) -> Option<Value> {
    let value = view.evidence_class.as_deref()?;
    let parsed: Value = serde_json::from_str(value).ok()?;
    if parsed.get("representation_id").is_some()
        && parsed.get("representation_kind").is_some()
        && parsed.get("reference_role").is_some()
    {
        Some(parsed)
    } else {
        None
    }
}

fn view_has_token(view: &ViewDraft, token: &str) -> bool {
    view.evidence.iter().any(|value| value == token)
}

fn graph_body_presence(rep: &GraphRepresentation) -> &'static str {
    if rep.representation_kind == "P1_METHOD_PROVENANCE"
        && rep.coverage.as_deref() == Some("DECOMPILE_OK/RESOLVED")
        && rep.body_sha256.is_some()
        && rep.source_ref.is_some()
        && rep.proof_ref.is_some()
    {
        "DECLARED_DECOMPILED_BODY_PRESENT"
    } else if rep.body_sha256.is_some() {
        "DECLARED_BODY_HASH_ONLY"
    } else {
        "NONE"
    }
}

fn graph_fallback_class(rep: &GraphRepresentation) -> &'static str {
    match rep.representation_kind.as_str() {
        "CANONICAL_METHOD_PROVENANCE" | "P1_METHOD_PROVENANCE" => "CANONICAL_GENERATED",
        "CURATED_IMPORTED"
        | "CURATED_ORIGINAL"
        | "GAMEPLAY_SEMANTIC_OVERLAY"
        | "PERFORMANCE_METHOD_LINK" => "SUPPLEMENTARY",
        _ => "UNRESOLVED",
    }
}

fn graph_ref_draft(
    role: &str,
    class: &str,
    artifact: &GraphArtifactRef,
    body: &str,
    evidence_level: &str,
    provenance: &str,
    scope: Option<&str>,
    view_kind: &str,
    reference_role: &str,
    rep: &GraphRepresentation,
) -> LocationDraft {
    LocationDraft {
        role: role.to_string(),
        representation_class: class.to_string(),
        declared_class: class.to_string(),
        root_id: artifact.root_id.clone(),
        path: artifact.path.clone(),
        declared_size: Some(artifact.size),
        declared_sha256: artifact.sha256.clone(),
        range: None,
        body_availability: body.to_string(),
        evidence_level: evidence_level.to_string(),
        provenance_status: provenance.to_string(),
        scope: scope.map(str::to_string),
        views: vec![ViewDraft {
            kind: view_kind.to_string(),
            declared_coverage: rep.coverage.clone(),
            evidence: {
                let mut evidence = rep.evidence.clone();
                evidence.push(format!("BODY_PRESENCE:{}", graph_body_presence(rep)));
                evidence
            },
            body_availability: body.to_string(),
            provenance_status: Some(provenance.to_string()),
            limitations: Vec::new(),
            evidence_class: Some(graph_view_meta(rep, reference_role)),
            precision: None,
            body_sha256: rep.body_sha256.clone(),
            body_hash_verified: rep.body_hash_verified,
            support_refs: Vec::new(),
        }],
        limitations: Vec::new(),
    }
}

fn graph_body(_rep: &GraphRepresentation, _explicit_full_coverage: bool) -> &'static str {
    // A graph-declared body hash is not a fresh semantic-identity proof. Keep
    // body placement unknown and expose the declaration as separate evidence.
    "BODY_UNKNOWN"
}

fn graph_fallback_draft(
    rep: &GraphRepresentation,
    artifact: &GraphArtifactRef,
    reference_role: &str,
    legacy_path: bool,
) -> LocationDraft {
    let evidence_only = matches!(reference_role, "raw" | "proof");
    let role = match reference_role {
        "source" => "GRAPH_DECLARED_SOURCE_FALLBACK",
        "semantic" => "GRAPH_DECLARED_SEMANTIC_FALLBACK",
        "raw" => "GRAPH_DECLARED_RAW_FALLBACK",
        _ => "GRAPH_DECLARED_PROOF_FALLBACK",
    };
    let class = if evidence_only {
        "EVIDENCE_ONLY"
    } else {
        graph_fallback_class(rep)
    };
    let body = if evidence_only {
        "NOT_APPLICABLE"
    } else {
        "BODY_UNKNOWN"
    };
    let mut evidence = rep.evidence.clone();
    evidence.push(format!("BODY_PRESENCE:{}", graph_body_presence(rep)));
    evidence.push(format!("REFERENCE_FALLBACK:{reference_role}"));
    let mut draft = LocationDraft {
        role: role.to_string(),
        representation_class: class.to_string(),
        declared_class: class.to_string(),
        root_id: artifact.root_id.clone(),
        path: artifact.path.clone(),
        declared_size: Some(artifact.size),
        declared_sha256: artifact.sha256.clone(),
        range: None,
        body_availability: body.to_string(),
        evidence_level: "DECLARED_REFERENCE_FALLBACK".to_string(),
        provenance_status: "GRAPH_DECLARED_REFERENCE".to_string(),
        scope: Some("GRAPH_REFERENCE_FALLBACK".to_string()),
        views: vec![ViewDraft {
            kind: format!("DECLARED_{}_FALLBACK", reference_role.to_ascii_uppercase()),
            declared_coverage: rep.coverage.clone(),
            evidence,
            body_availability: body.to_string(),
            provenance_status: Some("GRAPH_DECLARED_REFERENCE".to_string()),
            limitations: vec![
                "Every declared representation reference is retained even when a branch did not emit it."
                    .to_string(),
            ],
            evidence_class: Some(graph_view_meta(rep, reference_role)),
            precision: None,
            body_sha256: rep.body_sha256.clone(),
            body_hash_verified: rep.body_hash_verified,
            support_refs: Vec::new(),
        }],
        limitations: Vec::new(),
    };
    if !evidence_only {
        let range = if reference_role == "semantic" {
            rep.semantic_range.clone()
        } else {
            rep.source_range.clone()
        };
        if let Some(range) = range {
            set_declared_graph_range(&mut draft, range, legacy_path);
        }
    }
    draft
}

fn set_declared_graph_range(draft: &mut LocationDraft, range: LineRange, legacy_path: bool) {
    draft.range = Some(range);
    draft
        .views
        .first_mut()
        .expect("graph draft has a view")
        .evidence
        .push("RANGE:DECLARED_HISTORICAL_NOT_VALIDATED".to_string());
    if legacy_path {
        draft.views[0]
            .evidence
            .push("RANGE:LEGACY_QUARANTINED_PATH".to_string());
        draft.views[0].limitations.push(
            "Path matches a current legacy/original incorrect anchor; the graph range is quarantined, not revived."
                .to_string(),
        );
    }
}

#[derive(Debug, Default)]
struct GraphRefAccounting {
    declared_representation_refs: u64,
    accounted_representation_refs: u64,
    unaccounted_representation_refs: Vec<Value>,
    row_source_refs: u64,
    row_source_accounted: u64,
    row_raw_refs: u64,
    row_raw_accounted: u64,
}

fn add_graph_locations(
    locator: &mut Locator,
    row: &GraphMethodRow,
    access: Option<&CurrentMethodAccess>,
    accepted_paths: &BTreeSet<String>,
) -> GraphRefAccounting {
    let accepted_body = accepted_body_status(access);
    let accepted_evidence = accepted_evidence_status(access);
    let current_import_path = access
        .and_then(|access| access.imported_file.as_ref())
        .map(|file| file.imported.path.as_str());
    let current_original_path = access.map(|access| access.record_original_path.as_str());
    let legacy_paths = access
        .and_then(|access| access.source_provenance.as_ref())
        .and_then(|provenance| match provenance {
            SourceProvenance::ProfileEntry {
                legacy_original_references,
                ..
            } => Some(
                legacy_original_references
                    .iter()
                    .map(|anchor| anchor.path.clone())
                    .collect::<BTreeSet<_>>(),
            ),
            SourceProvenance::NotValidated => None,
        })
        .unwrap_or_default();
    for rep in &row.representations {
        match rep.representation_kind.as_str() {
            "CANONICAL_METHOD_PROVENANCE" => {
                if let Some(artifact) = &rep.source_ref {
                    let mut draft = graph_ref_draft(
                        "GRAPH_CANONICAL_SOURCE",
                        "CANONICAL_GENERATED",
                        artifact,
                        "BODY_UNKNOWN",
                        "METADATA_RAW_PROVENANCE",
                        "GRAPH_CANONICAL_ASSOCIATION_ONLY",
                        None,
                        "CANONICAL_METHOD_PROVENANCE",
                        "source",
                        rep,
                    );
                    draft.views[0].limitations.push(
                        "Canonical raw/source association does not prove a method or constructor body."
                            .to_string(),
                    );
                    draft.limitations.push(
                        "Original representation kind retained separately from accepted current-import status."
                            .to_string(),
                    );
                    locator.add(draft);
                }
                if let Some(artifact) = &rep.raw_ref {
                    let mut draft = graph_ref_draft(
                        "GRAPH_CANONICAL_RAW",
                        "EVIDENCE_ONLY",
                        artifact,
                        "NOT_APPLICABLE",
                        "RAW_METHOD_REFERENCE",
                        "GRAPH_EVIDENCE_ONLY",
                        Some("RAW_OR_PROOF_NEVER_CLAIMS_SOURCE_BODY"),
                        "CANONICAL_RAW_REFERENCE",
                        "raw",
                        rep,
                    );
                    draft
                        .limitations
                        .push("Raw method evidence is not a physical source body.".to_string());
                    locator.add(draft);
                }
                if let Some(artifact) = &rep.proof_ref {
                    let mut draft = graph_ref_draft(
                        "GRAPH_CANONICAL_PROOF",
                        "EVIDENCE_ONLY",
                        artifact,
                        "NOT_APPLICABLE",
                        "METHOD_PROVENANCE_PROOF_REFERENCE",
                        "GRAPH_EVIDENCE_ONLY",
                        Some("RAW_OR_PROOF_NEVER_CLAIMS_SOURCE_BODY"),
                        "CANONICAL_PROOF_REFERENCE",
                        "proof",
                        rep,
                    );
                    draft
                        .limitations
                        .push("Proof evidence is not a physical source body.".to_string());
                    locator.add(draft);
                }
            }
            "P1_METHOD_PROVENANCE" => {
                let body = graph_body(
                    rep,
                    rep.coverage.as_deref() == Some("DECOMPILE_OK/RESOLVED"),
                );
                if let Some(artifact) = &rep.source_ref {
                    let mut draft = graph_ref_draft(
                        "GRAPH_P1_SOURCE",
                        "CANONICAL_GENERATED",
                        artifact,
                        body,
                        "P1_METHOD_PROVENANCE",
                        if body == "BODY_AVAILABLE" {
                            "P1_DECOMPILE_OK_BODY_HASH_VERIFIED"
                        } else {
                            "P1_DECLARED_PROVENANCE"
                        },
                        None,
                        "P1_METHOD_PROVENANCE",
                        "source",
                        rep,
                    );
                    draft.views[0].limitations.push(
                        "P1 whole-file source association; the body hash is declared evidence for one selected body, not the whole file or semantic identity."
                            .to_string(),
                    );
                    draft.views[0].limitations.push(
                        "BODY_PRESENCE:DECLARED_DECOMPILED_BODY_PRESENT is a declared snapshot fact; it is not a fresh body extraction."
                            .to_string(),
                    );
                    locator.add(draft);
                }
                if let Some(artifact) = &rep.proof_ref {
                    locator.add(graph_ref_draft(
                        "GRAPH_P1_PROOF",
                        "EVIDENCE_ONLY",
                        artifact,
                        "NOT_APPLICABLE",
                        "P1_PROOF_REFERENCE",
                        "GRAPH_EVIDENCE_ONLY",
                        Some("RAW_OR_PROOF_NEVER_CLAIMS_SOURCE_BODY"),
                        "P1_PROOF_REFERENCE",
                        "proof",
                        rep,
                    ));
                }
            }
            "GAMEPLAY_SEMANTIC_OVERLAY" | "PERFORMANCE_METHOD_LINK" => {
                let explicit_full = rep
                    .coverage
                    .as_deref()
                    .map(|coverage| coverage.contains("FULL_CIL"))
                    .unwrap_or(false);
                let body = graph_body(rep, explicit_full);
                if let Some(artifact) = &rep.source_ref {
                    let mut draft = graph_ref_draft(
                        "GRAPH_SEMANTIC_SOURCE",
                        "SUPPLEMENTARY",
                        artifact,
                        body,
                        if explicit_full {
                            "GRAPH_DECLARED_FULL_CIL"
                        } else {
                            "GRAPH_SEMANTIC_REFERENCE"
                        },
                        "GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT",
                        Some("GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT"),
                        "GRAPH_SEMANTIC_REFERENCE",
                        "source",
                        rep,
                    );
                    draft.views[0].limitations.push(
                        "Graph-declared body evidence is not an accepted current-import claim."
                            .to_string(),
                    );
                    if let Some(range) = &rep.source_range {
                        set_declared_graph_range(
                            &mut draft,
                            range.clone(),
                            legacy_paths.contains(&artifact.path),
                        );
                        draft.views[0].limitations.push(
                            "Range is graph-declared historical; it is not returned as current validated navigation."
                                .to_string(),
                        );
                    }
                    locator.add(draft);
                }
                if let Some(artifact) = &rep.semantic_ref {
                    let mut draft = graph_ref_draft(
                        "GRAPH_SEMANTIC_OVERLAY",
                        "SUPPLEMENTARY",
                        artifact,
                        body,
                        "GRAPH_SEMANTIC_OVERLAY",
                        "GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT",
                        Some("GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT"),
                        "GRAPH_SEMANTIC_OVERLAY",
                        "semantic",
                        rep,
                    );
                    if let Some(range) = &rep.semantic_range {
                        set_declared_graph_range(
                            &mut draft,
                            range.clone(),
                            legacy_paths.contains(&artifact.path),
                        );
                        draft.views[0].limitations.push(
                            "Semantic range is graph-declared historical, not current validated navigation."
                                .to_string(),
                        );
                    }
                    draft.views[0].limitations.push(
                        "Semantic overlay is a graph reference, not an accepted import body."
                            .to_string(),
                    );
                    locator.add(draft);
                }
                if let Some(artifact) = &rep.raw_ref {
                    locator.add(graph_ref_draft(
                        "GRAPH_SEMANTIC_RAW",
                        "EVIDENCE_ONLY",
                        artifact,
                        "NOT_APPLICABLE",
                        "RAW_METHOD_REFERENCE",
                        "GRAPH_EVIDENCE_ONLY",
                        Some("RAW_OR_PROOF_NEVER_CLAIMS_SOURCE_BODY"),
                        "GRAPH_SEMANTIC_RAW",
                        "raw",
                        rep,
                    ));
                }
                if let Some(artifact) = &rep.proof_ref {
                    locator.add(graph_ref_draft(
                        "GRAPH_SEMANTIC_PROOF",
                        "EVIDENCE_ONLY",
                        artifact,
                        "NOT_APPLICABLE",
                        "SEMANTIC_PROOF_REFERENCE",
                        "GRAPH_EVIDENCE_ONLY",
                        Some("RAW_OR_PROOF_NEVER_CLAIMS_SOURCE_BODY"),
                        "GRAPH_SEMANTIC_PROOF",
                        "proof",
                        rep,
                    ));
                }
            }
            "CURATED_IMPORTED" => {
                if let Some(artifact) = &rep.source_ref {
                    let is_primary = accepted_paths.contains(&artifact.path)
                        && current_import_path == Some(artifact.path.as_str());
                    let class = if is_primary {
                        "PRIMARY_CURATED"
                    } else {
                        "SUPPLEMENTARY"
                    };
                    let body = if is_primary {
                        accepted_body
                    } else {
                        "BODY_UNKNOWN"
                    };
                    let mut draft = graph_ref_draft(
                        "GRAPH_CURATED_IMPORT",
                        class,
                        artifact,
                        body,
                        if is_primary {
                            &accepted_evidence
                        } else {
                            "GRAPH_CURATED_REFERENCE"
                        },
                        if is_primary {
                            "GRAPH_CURATED_IMPORT_REFERENCE"
                        } else {
                            "GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT"
                        },
                        if is_primary {
                            Some("CURRENT_ACCEPTED_IMPORT")
                        } else {
                            Some("GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT")
                        },
                        "GRAPH_CURATED_IMPORT",
                        "source",
                        rep,
                    );
                    if !is_primary {
                        draft.views[0].limitations.push(
                            "Curated path is not the current accepted import for this method."
                                .to_string(),
                        );
                    }
                    locator.add(draft);
                }
                if let Some(artifact) = &rep.proof_ref {
                    locator.add(graph_ref_draft(
                        "GRAPH_CURATED_PROOF",
                        "EVIDENCE_ONLY",
                        artifact,
                        "NOT_APPLICABLE",
                        "CURATED_PROOF_REFERENCE",
                        "GRAPH_EVIDENCE_ONLY",
                        Some("RAW_OR_PROOF_NEVER_CLAIMS_SOURCE_BODY"),
                        "GRAPH_CURATED_PROOF",
                        "proof",
                        rep,
                    ));
                }
            }
            "CURATED_ORIGINAL" => {
                if let Some(artifact) = &rep.source_ref {
                    let is_current = current_original_path == Some(artifact.path.as_str());
                    let body = if is_current {
                        accepted_body
                    } else {
                        "BODY_UNKNOWN"
                    };
                    locator.add(graph_ref_draft(
                        "GRAPH_CURATED_ORIGINAL",
                        "SUPPLEMENTARY",
                        artifact,
                        body,
                        if is_current {
                            &accepted_evidence
                        } else {
                            "GRAPH_ORIGINAL_REFERENCE"
                        },
                        if is_current {
                            "ACCEPTED_CURRENT_ORIGINAL"
                        } else {
                            "GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT"
                        },
                        Some("ORIGINAL_RESEARCH_REFERENCE"),
                        "GRAPH_CURATED_ORIGINAL",
                        "source",
                        rep,
                    ));
                }
            }
            _ => {
                if let Some(artifact) = &rep.source_ref {
                    locator.add(graph_ref_draft(
                        "GRAPH_REPRESENTATION_SOURCE",
                        "SUPPLEMENTARY",
                        artifact,
                        "BODY_UNKNOWN",
                        "GRAPH_UNCLASSIFIED_REFERENCE",
                        "GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT",
                        Some("GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT"),
                        "GRAPH_REPRESENTATION_SOURCE",
                        "source",
                        rep,
                    ));
                }
                if let Some(artifact) = &rep.semantic_ref {
                    locator.add(graph_ref_draft(
                        "GRAPH_REPRESENTATION_SEMANTIC",
                        "SUPPLEMENTARY",
                        artifact,
                        "BODY_UNKNOWN",
                        "GRAPH_UNCLASSIFIED_REFERENCE",
                        "GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT",
                        Some("GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT"),
                        "GRAPH_REPRESENTATION_SEMANTIC",
                        "semantic",
                        rep,
                    ));
                }
            }
        }
        for (reference_role, artifact) in [
            ("source", rep.source_ref.as_ref()),
            ("semantic", rep.semantic_ref.as_ref()),
            ("raw", rep.raw_ref.as_ref()),
            ("proof", rep.proof_ref.as_ref()),
        ] {
            if let Some(artifact) = artifact
                && !locator.has_graph_ref(&rep.representation_id, reference_role)
            {
                locator.add(graph_fallback_draft(
                    rep,
                    artifact,
                    reference_role,
                    legacy_paths.contains(&artifact.path),
                ));
            }
        }
    }
    for artifact in &row.source_refs {
        if locator.has_source_path(&artifact.root_id, &artifact.path) {
            continue;
        }
        let class = if access
            .and_then(|access| access.imported_file.as_ref())
            .map(|file| file.imported.path.as_str())
            == Some(artifact.path.as_str())
            && accepted_paths.contains(&artifact.path)
        {
            "PRIMARY_CURATED"
        } else {
            "SUPPLEMENTARY"
        };
        let mut draft = LocationDraft {
            role: "GRAPH_SOURCE_REF_EXTRA".to_string(),
            representation_class: class.to_string(),
            declared_class: class.to_string(),
            root_id: artifact.root_id.clone(),
            path: artifact.path.clone(),
            declared_size: Some(artifact.size),
            declared_sha256: artifact.sha256.clone(),
            range: None,
            body_availability: "BODY_UNKNOWN".to_string(),
            evidence_level: "ROW_LEVEL_SOURCE_REFERENCE".to_string(),
            provenance_status: "GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT".to_string(),
            scope: Some("GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT".to_string()),
            views: vec![ViewDraft {
                kind: "ROW_LEVEL_SOURCE_REFERENCE".to_string(),
                declared_coverage: None,
                evidence: row.evidence_levels.keys().cloned().collect(),
                body_availability: "BODY_UNKNOWN".to_string(),
                provenance_status: Some("GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT".to_string()),
                limitations: vec![
                    "Row-level source reference is accounted for but has no representation body proof."
                        .to_string(),
                ],
                evidence_class: Some("ROW_SOURCE_REF".to_string()),
                precision: None,
                body_sha256: None,
                body_hash_verified: None,
                support_refs: Vec::new(),
            }],
            limitations: vec![
                "Not dropped: all graph source_refs remain represented even when unresolved."
                    .to_string(),
            ],
        };
        if !accepted_paths.contains(&artifact.path) {
            draft.views[0]
                .limitations
                .push("Path is outside the accepted 131 current curated refs.".to_string());
        }
        locator.add(draft);
    }
    for artifact in &row.raw_refs {
        if locator.has_evidence_path(&artifact.root_id, &artifact.path) {
            continue;
        }
        locator.add(LocationDraft {
            role: "GRAPH_RAW_REF".to_string(),
            representation_class: "EVIDENCE_ONLY".to_string(),
            declared_class: "EVIDENCE_ONLY".to_string(),
            root_id: artifact.root_id.clone(),
            path: artifact.path.clone(),
            declared_size: Some(artifact.size),
            declared_sha256: artifact.sha256.clone(),
            range: None,
            body_availability: "NOT_APPLICABLE".to_string(),
            evidence_level: "ROW_LEVEL_RAW_REFERENCE".to_string(),
            provenance_status: "GRAPH_EVIDENCE_ONLY".to_string(),
            scope: Some("RAW_OR_PROOF_NEVER_CLAIMS_SOURCE_BODY".to_string()),
            views: vec![ViewDraft {
                kind: "ROW_LEVEL_RAW_REFERENCE".to_string(),
                declared_coverage: None,
                evidence: row.evidence_levels.keys().cloned().collect(),
                body_availability: "NOT_APPLICABLE".to_string(),
                provenance_status: Some("GRAPH_EVIDENCE_ONLY".to_string()),
                limitations: vec!["Raw reference cannot claim a physical source body.".to_string()],
                evidence_class: Some("ROW_RAW_REF".to_string()),
                precision: None,
                body_sha256: None,
                body_hash_verified: None,
                support_refs: Vec::new(),
            }],
            limitations: Vec::new(),
        });
    }
    let mut accounting = GraphRefAccounting::default();
    for rep in &row.representations {
        for (reference_role, artifact) in [
            ("source", rep.source_ref.as_ref()),
            ("semantic", rep.semantic_ref.as_ref()),
            ("raw", rep.raw_ref.as_ref()),
            ("proof", rep.proof_ref.as_ref()),
        ] {
            if artifact.is_none() {
                continue;
            }
            accounting.declared_representation_refs += 1;
            if locator.has_graph_ref(&rep.representation_id, reference_role) {
                accounting.accounted_representation_refs += 1;
            } else {
                accounting.unaccounted_representation_refs.push(json!({
                    "representation_id": rep.representation_id,
                    "reference_role": reference_role,
                    "path": artifact.map(|value| value.path.clone()),
                }));
            }
        }
    }
    for artifact in &row.source_refs {
        accounting.row_source_refs += 1;
        if locator.has_source_path(&artifact.root_id, &artifact.path) {
            accounting.row_source_accounted += 1;
        }
    }
    for artifact in &row.raw_refs {
        accounting.row_raw_refs += 1;
        if locator.has_evidence_path(&artifact.root_id, &artifact.path) {
            accounting.row_raw_accounted += 1;
        }
    }
    accounting
}

fn accepted_curated_paths(
    roots: &Roots,
    manifest: &SourceManifest,
    successor: &multilang::Successor,
) -> RegistryResult<BTreeSet<String>> {
    let mut paths = BTreeSet::new();
    let active = crate::load_active_receipt(&roots.curated)
        .map_err(|error| reg("CURRENT_AUTHORITY_ERROR", error.code))?
        .ok_or_else(|| {
            reg(
                "CURRENT_AUTHORITY_MISSING",
                "source-locate requires the current active receipt",
            )
        })?;
    for file in active.files {
        paths.insert(file.imported.path);
    }
    for import in &manifest.imports {
        paths.insert(import.destination.path.clone());
    }
    paths.extend(successor.physical_paths());
    Ok(paths)
}

fn access_status(locations: &[Value], missing: u64) -> (u8, &'static str) {
    if missing > 0 {
        return (1, "PARTIAL_MISSING_BACKING");
    }
    let limited = locations.iter().any(|location| {
        let body = location["body_availability"].as_str().unwrap_or("");
        !matches!(body, "BODY_AVAILABLE" | "ASSOCIATED_OR_EMBEDDED")
            || location["representation_class"] == "UNRESOLVED"
    });
    if limited {
        (0, "FOUND_WITH_LIMITATIONS")
    } else {
        (0, "FOUND")
    }
}

fn index_summary_value(index: &LoadedIndex, entry: &IndexEntry) -> Value {
    json!({
        "path": index.rel_path,
        "bytes": index.bytes,
        "sha256": index.sha256,
        "header": {
            "graph_methods_sha256": index.header.graph_methods_sha256,
            "graph_methods_bytes": index.header.graph_methods_bytes,
            "row_count": index.header.row_count
        },
        "selected_entry": {
            "offset": entry.offset,
            "length": entry.length,
            "row_sha256": hex_digest(entry.row_sha256),
            "method_id": hex_digest(entry.method_id)
        }
    })
}

fn graph_row_value(row: &GraphMethodRow) -> Value {
    let mut representation_kinds: BTreeMap<String, u64> = BTreeMap::new();
    for rep in &row.representations {
        *representation_kinds
            .entry(rep.representation_kind.clone())
            .or_insert(0) += 1;
    }
    json!({
        "method_id": row.method_id,
        "assembly_sha256": row.assembly_sha256,
        "mvid": row.mvid,
        "type_token": row.type_token,
        "method_token": row.method_token,
        "root_owners": row.root_owners,
        "representation_kinds": representation_kinds,
        "evidence_levels": row.evidence_levels,
        "cohorts": row.cohorts,
        "limitations": row.limitations
    })
}

pub(super) fn run_locate(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    let (exit, value) = locate_with_trust(flags, &production_trust(), &production_index_trust())?;
    print_cli(value);
    Ok(exit)
}

fn locate_with_trust(
    flags: &BTreeMap<String, PathBuf>,
    manifest_trust: &Trust,
    index_trust: &IndexTrust,
) -> RegistryResult<(u8, Value)> {
    let path = manifest_path(flags);
    let (manifest, bytes) = load_manifest_trusted(&path, manifest_trust)?;
    let roots = roots_from_flags(flags, true)?;
    let successor = if manifest_trust.successor_required {
        Some(multilang::load_candidate(
            &roots,
            &manifest,
            &bytes,
            manifest_trust,
            true,
        )?)
    } else {
        None
    };
    let successor = successor.ok_or_else(|| {
        reg(
            "SUCCESSOR_REQUIRED",
            "source-locate requires the accepted successor view for current curated refs",
        )
    })?;

    let authority_arg = flags_string(flags, "--authority")
        .ok_or_else(|| reg("LOCATE_ARGUMENT_MISSING", "--authority is required"))?;
    let authority_sha = if authority_arg.eq_ignore_ascii_case(REQUIRED_MANAGED_AUTHORITY)
        || authority_arg.eq_ignore_ascii_case("MANAGED-CS-E3FC67839851")
    {
        REQUIRED_MANAGED_AUTHORITY
    } else {
        return Err(reg(
            "AUTHORITY_UNKNOWN",
            "authority is not the accepted managed authority id or exact assembly SHA",
        ));
    };
    let mvid = flags_string(flags, "--mvid").unwrap_or_else(|| REQUIRED_MANAGED_MVID.to_string());
    if !mvid.eq_ignore_ascii_case(REQUIRED_MANAGED_MVID) {
        return Err(reg(
            "AUTHORITY_MVID_MISMATCH",
            "supplied MVID does not match the accepted managed authority",
        ));
    }
    let type_def = token(flags_string(flags, "--type"), "--type", true)?;
    let method_def = token(flags_string(flags, "--method"), "--method", false)?;
    if !manifest.imports.iter().any(|import| {
        import.layer == "CS"
            && import
                .managed_authority_sha256
                .as_deref()
                .map(|value| value.eq_ignore_ascii_case(authority_sha))
                .unwrap_or(false)
            && import
                .managed_mvid
                .as_deref()
                .map(|value| value.eq_ignore_ascii_case(REQUIRED_MANAGED_MVID))
                .unwrap_or(false)
    }) {
        return Err(reg(
            "AUTHORITY_NOT_IN_ACCEPTED_MANIFEST",
            "managed authority/MVID are not bound by the accepted supplement manifest",
        ));
    }
    let identity = MethodIdentity {
        full_authority_sha256: authority_sha.to_string(),
        mvid: REQUIRED_MANAGED_MVID.to_string(),
        type_def: type_def.clone(),
        method_def: method_def.clone(),
    };
    let target = method_id(authority_sha, REQUIRED_MANAGED_MVID, &type_def, &method_def);
    let access = crate::current_imported_method_access(&roots.curated, &identity)
        .map_err(|code| reg("CURRENT_AUTHORITY_ERROR", code))?;
    let index_rel = flags_string(flags, "--index").unwrap_or_else(|| DEFAULT_INDEX_REL.to_string());
    let index = load_index_trusted(&roots.curated, &index_rel, index_trust)?;
    let Some((row, entry)) = read_indexed_row(&roots, &manifest, &index, &target)? else {
        let value = json!({
            "command": "source-locate",
            "schema": "wuwa-reconstruction-source-locate-v1",
            "ok": false,
            "exit": 1,
            "status": "NOT_INDEXED",
            "trust_status": trust_status(),
            "multilang_trust_status": multilang_trust_status(),
            "index_trust_status": index_trust_status(index_trust),
            "production_manifest_pin": PRODUCTION_MANIFEST_SHA256,
            "production_multilang_manifest_pin": PRODUCTION_MULTILANG_MANIFEST_SHA256,
            "index": {
                "path": index.rel_path,
                "bytes": index.bytes,
                "sha256": index.sha256,
                "header": {
                    "graph_methods_sha256": index.header.graph_methods_sha256,
                    "graph_methods_bytes": index.header.graph_methods_bytes,
                    "row_count": index.header.row_count
                }
            },
            "method": {
                "method_id": target,
                "authority_id": "MANAGED-CS-E3FC67839851",
                "assembly_sha256": authority_sha,
                "mvid": REQUIRED_MANAGED_MVID,
                "type_def": type_def,
                "method_def": method_def
            },
            "current_authority": current_authority_value(access.as_ref()),
            "alternative_current_record_ids": access
                .as_ref()
                .map(|access| access.alternative_record_ids.clone())
                .unwrap_or_default(),
            "exact_source_views": exact_source_views_value(access.as_ref(), &[]),
            "supplement": supplement_value(&manifest, &target),
            "locations": [],
            "accounting": {
                "indexed": false,
                "graph_row_present": false,
                "locations": 0,
                "all_graph_source_refs_accounted": true,
                "unselected_graph_rows_rechecked": false
            },
            "verification": {
                "index_hash_verified": true,
                "selected_row_verified": false,
                "method_semantics": "NOT_CHECKED"
            },
            "limits": [
                "Absence applies to the pinned historical graph snapshot only; the whole graph is not rehashed per lookup.",
                "No method semantics, buildability, ABI, or runtime behavior is established."
            ]
        });
        return Ok((1, value));
    };

    let accepted_paths = accepted_curated_paths(&roots, &manifest, &successor)?;
    let mut locator = Locator::default();
    add_active_locations(&mut locator, access.as_ref());
    add_supplement_locations(&mut locator, &manifest, &target, access.as_ref());
    let graph_ref_accounting =
        add_graph_locations(&mut locator, &row, access.as_ref(), &accepted_paths);
    if !graph_ref_accounting
        .unaccounted_representation_refs
        .is_empty()
    {
        return Err(reg(
            "GRAPH_REFERENCE_ACCOUNTING_LOST",
            format!(
                "{} declared representation references were not represented",
                graph_ref_accounting.unaccounted_representation_refs.len()
            ),
        ));
    }
    let mut cache = PhysCache::default();
    let (locations, missing, hash_verified) = locator.finalize(&roots, &mut cache)?;
    let exact_source_views = exact_source_views_value(access.as_ref(), &locations);
    if exact_source_views["unrepresented"]
        .as_array()
        .map(|rows| !rows.is_empty())
        .unwrap_or(false)
    {
        return Err(reg(
            "SOURCE_VIEW_ACCOUNTING_LOST",
            "one or more exact current/historical source views were not represented",
        ));
    }
    let source_refs_accounted =
        graph_ref_accounting.row_source_accounted == graph_ref_accounting.row_source_refs;
    let raw_refs_accounted =
        graph_ref_accounting.row_raw_accounted == graph_ref_accounting.row_raw_refs;
    if !source_refs_accounted || !raw_refs_accounted {
        return Err(reg(
            "GRAPH_REFERENCE_ACCOUNTING_LOST",
            "one or more graph source/raw references were not represented",
        ));
    }
    let (exit, status) = access_status(&locations, missing);
    let graph_source_refs = row.source_refs.len() as u64;
    let graph_raw_refs = row.raw_refs.len() as u64;
    let representations = row.representations.len() as u64;
    let value = json!({
        "command": "source-locate",
        "schema": "wuwa-reconstruction-source-locate-v1",
        "ok": exit == 0,
        "exit": exit,
        "status": status,
        "trust_status": trust_status(),
        "multilang_trust_status": multilang_trust_status(),
        "index_trust_status": index_trust_status(index_trust),
        "production_manifest_pin": PRODUCTION_MANIFEST_SHA256,
        "production_multilang_manifest_pin": PRODUCTION_MULTILANG_MANIFEST_SHA256,
        "method": {
            "method_id": target,
            "authority_id": "MANAGED-CS-E3FC67839851",
            "assembly_sha256": authority_sha,
            "mvid": REQUIRED_MANAGED_MVID,
            "type_def": type_def,
            "method_def": method_def
        },
        "index": index_summary_value(&index, &entry),
        "current_authority": current_authority_value(access.as_ref()),
        "alternative_current_record_ids": access
            .as_ref()
            .map(|access| access.alternative_record_ids.clone())
            .unwrap_or_default(),
        "supplement": supplement_value(&manifest, &target),
        "graph_row": graph_row_value(&row),
        "exact_source_views": exact_source_views,
        "locations": locations,
        "accounting": {
            "indexed": true,
            "graph_row_present": true,
            "representations": representations,
            "graph_source_refs": graph_source_refs,
            "graph_raw_refs": graph_raw_refs,
            "declared_representation_refs": graph_ref_accounting.declared_representation_refs,
            "accounted_representation_refs": graph_ref_accounting.accounted_representation_refs,
            "unaccounted_representation_refs": graph_ref_accounting.unaccounted_representation_refs.len() as u64,
            "row_source_refs": graph_ref_accounting.row_source_refs,
            "row_source_accounted": graph_ref_accounting.row_source_accounted,
            "row_raw_refs": graph_ref_accounting.row_raw_refs,
            "row_raw_accounted": graph_ref_accounting.row_raw_accounted,
            "locations": locations.len() as u64,
            "locations_missing": missing,
            "locations_hash_verified": hash_verified,
            "all_graph_source_refs_accounted": source_refs_accounted,
            "all_graph_raw_refs_accounted": raw_refs_accounted,
            "all_declared_representation_refs_accounted": graph_ref_accounting.declared_representation_refs == graph_ref_accounting.accounted_representation_refs,
            "unselected_graph_rows_rechecked": false
        },
        "verification": {
            "index_sha256_verified": true,
            "index_header_graph_binding_verified": true,
            "index_stability_scope": "SIZE_AND_MTIME_NOT_FILE_ID",
            "selected_row_sha256_verified": true,
            "selected_row_full_key_verified": true,
            "graph_file_size_verified": true,
            "graph_file_pre_post_stable": true,
            "graph_methods_stability_scope": "SIZE_AND_MTIME_NOT_FILE_ID",
            "unselected_graph_rows_rechecked": false,
            "source_hashes_verified": hash_verified,
            "method_semantics": "NOT_CHECKED",
            "body_semantics": "NOT_CHECKED",
            "buildability": "NOT_ESTABLISHED",
            "abi_equivalence": "NOT_ESTABLISHED"
        },
        "limits": [
            "The selected graph row was verified; the remaining 305855 graph rows were not rehashed by this lookup.",
            "The index is a derivative accelerator; the pinned graph and accepted manifests remain authority.",
            "Whole-file hashes verify bytes, not method semantics or body identity inside the file.",
            "Graph-declared coverage/evidence is retained per representation and is not an accepted current-import claim.",
            "Accepted status fields are authoritative only for their own representation.",
            "UNKNOWN and METADATA_ONLY accepted classes are not promoted by graph references.",
            "Raw/proof references are evidence-only and never claim a physical source body.",
            "Canonical source association does not prove a constructor or method body.",
            "Historical managed relations are record metadata; no separate historical managed artifact is fabricated.",
            "No buildability, ABI, or runtime behavior is established; no source or binary was executed."
        ]
    });
    Ok((exit, value))
}

fn prepare_output_parent(curated: &Path, rel: &str) -> RegistryResult<PathBuf> {
    crate::validate_relative_path(rel).map_err(|_| reg("UNSAFE_RELATIVE_PATH", "index output"))?;
    let relative = Path::new(rel);
    let mut current = curated.to_path_buf();
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            let Component::Normal(name) = component else {
                return Err(reg("UNSAFE_RELATIVE_PATH", "index output parent"));
            };
            current.push(name);
            match fs::symlink_metadata(&current) {
                Ok(metadata) => {
                    if super::is_reparse(&metadata) || !metadata.is_dir() {
                        return Err(reg(
                            "UNSAFE_REPARSE_OR_SYMLINK",
                            "index output parent is not a plain directory",
                        ));
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir(&current)
                        .map_err(|_| reg("INDEX_OUTPUT_DIRECTORY", "cannot create index parent"))?;
                    let metadata = fs::symlink_metadata(&current)
                        .map_err(|_| reg("INDEX_OUTPUT_DIRECTORY", "index parent metadata"))?;
                    if super::is_reparse(&metadata) || !metadata.is_dir() {
                        return Err(reg(
                            "UNSAFE_REPARSE_OR_SYMLINK",
                            "created index parent is not a plain directory",
                        ));
                    }
                }
                Err(_) => {
                    return Err(reg(
                        "INDEX_OUTPUT_DIRECTORY",
                        "index parent metadata failed",
                    ));
                }
            }
        }
    }
    Ok(curated.join(relative))
}

fn publish_index(path: &Path, bytes: &[u8]) -> RegistryResult<()> {
    let metadata = fs::symlink_metadata(path);
    match metadata {
        Ok(_) => {
            return Err(reg(
                "INDEX_OUTPUT_EXISTS",
                "refusing to overwrite an existing derived index",
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err(reg("INDEX_OUTPUT_EXISTS", "index output metadata failed")),
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            reg(
                "INDEX_OUTPUT_CREATE",
                format!("index create failed at {}: {error}", path.display()),
            )
        })?;
    if let Err(error) = file.write_all(bytes) {
        return Err(reg(
            "INDEX_OUTPUT_PARTIAL",
            format!(
                "partial index left at {}; no automatic deletion: {error}",
                path.display()
            ),
        ));
    }
    if let Err(error) = file.sync_all() {
        return Err(reg(
            "INDEX_OUTPUT_PARTIAL",
            format!(
                "partial index left at {}; sync failed: {error}",
                path.display()
            ),
        ));
    }
    drop(file);
    let written = fs::read(path).map_err(|error| {
        reg(
            "INDEX_READ_ERROR",
            format!("index read-back failed at {}: {error}", path.display()),
        )
    })?;
    if written != bytes {
        return Err(reg(
            "INDEX_READBACK_MISMATCH",
            format!(
                "published index differs from generated bytes; owned output left at {} for quarantine",
                path.display()
            ),
        ));
    }
    Ok(())
}

pub(super) fn run_access_index(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    let path = manifest_path(flags);
    let trust = production_trust();
    let (manifest, bytes) = load_manifest_trusted(&path, &trust)?;
    let roots = roots_from_flags(flags, false)?;
    let successor = multilang::load_candidate(&roots, &manifest, &bytes, &trust, true)?;
    let _summary = load_graph_summary(&roots, &manifest, false)?;
    let payload = manifest
        .baseline
        .graph
        .payloads
        .iter()
        .find(|payload| payload.name == "methods.jsonl")
        .ok_or_else(|| reg("GRAPH_PAYLOAD_HASH_MISMATCH", "methods payload missing"))?;
    if payload.bytes != ACCEPTED_METHODS_BYTES
        || payload.sha256 != ACCEPTED_METHODS_SHA256
        || manifest
            .baseline
            .graph
            .counts
            .get("known_methods_all5_incl_abstract")
            .copied()
            != Some(ACCEPTED_METHOD_ROWS)
    {
        return Err(reg(
            "INDEX_GRAPH_BINDING_MISMATCH",
            "accepted manifest does not bind the index graph constants",
        ));
    }
    let output_rel =
        flags_string(flags, "--output").unwrap_or_else(|| DEFAULT_INDEX_REL.to_string());
    let output_path = prepare_output_parent(&roots.curated, &output_rel)?;
    if fs::symlink_metadata(&output_path).is_ok() {
        return Err(reg(
            "INDEX_OUTPUT_EXISTS",
            "refusing to overwrite an existing derived index",
        ));
    }
    let graph_path = roots
        .curated
        .join(&manifest.baseline.graph.path)
        .join("methods.jsonl");
    inspect_existing_file(&graph_path)?;
    let before = observe_file(&graph_path)?;
    let file = File::open(&graph_path).map_err(|_| reg("GRAPH_METHODS_MISSING", "methods open"))?;
    let mut reader = std::io::BufReader::with_capacity(1024 * 1024, file);
    let index_bytes = build_index_bytes(
        &mut reader,
        ACCEPTED_METHODS_BYTES,
        ACCEPTED_METHODS_SHA256,
        ACCEPTED_METHOD_ROWS,
    )?;
    let after = observe_file(&graph_path)?;
    inspect_existing_file(&graph_path)?;
    if before != after {
        return Err(reg(
            "GRAPH_METHODS_UNSTABLE",
            "graph methods changed while the index was built",
        ));
    }
    publish_index(&output_path, &index_bytes)?;
    let index_sha = crate::sha256_hex(&index_bytes);
    print_cli(json!({
        "command": "source-access-index",
        "schema": "wuwa-reconstruction-source-access-index-v1",
        "ok": true,
        "exit": 0,
        "status": "INDEX_CREATED",
        "trust_status": trust_status(),
        "multilang_trust_status": multilang_trust_status(),
        "index_trust_status": index_trust_status(&production_index_trust()),
        "approved_for_production_lookup": false,
        "production_manifest_pin": PRODUCTION_MANIFEST_SHA256,
        "production_multilang_manifest_pin": PRODUCTION_MULTILANG_MANIFEST_SHA256,
        "output": {
            "path": output_rel,
            "bytes": index_bytes.len(),
            "sha256": index_sha
        },
        "header": {
            "magic": "WUWAIDX1",
            "version": INDEX_VERSION,
            "graph_methods_sha256": ACCEPTED_METHODS_SHA256,
            "graph_methods_bytes": ACCEPTED_METHODS_BYTES,
            "row_count": ACCEPTED_METHOD_ROWS,
            "entry_bytes": INDEX_ENTRY_BYTES,
            "header_bytes": INDEX_HEADER_BYTES
        },
        "input": {
            "graph_methods_size_stable": before == after,
            "rows_validated": ACCEPTED_METHOD_ROWS,
            "unique_method_ids": ACCEPTED_METHOD_ROWS,
            "strict_json_rows": ACCEPTED_METHOD_ROWS,
            "source_copies": false,
            "content_embedded": false,
            "provenance_embedded": false
        },
        "successor_manifest_sha256": successor.sha256(),
        "limits": [
            "Derived index only; it is not a new canonical database.",
            "The index header is bound to the accepted graph methods SHA, size, and row count.",
            "Production source-locate verifies the whole index against the compiled authorized SHA-256 pin and fails closed on any mismatch."
        ]
    }));
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn require_private_index(path: &Path) {
        assert!(
            path.exists(),
            "dependent private index is required to run corpus tests; missing {}",
            path.display()
        );
    }

    #[test]
    #[should_panic(expected = "dependent private index is required to run corpus tests")]
    fn missing_private_index_guard_names_the_dependency() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = std::env::current_dir()
            .expect("process current dir")
            .join("target/test-fixtures")
            .join(format!(
                "missing-private-index-{}-{unique}.idx",
                std::process::id()
            ));
        assert!(
            !path.exists(),
            "missing-index fixture exists: {}",
            path.display()
        );
        require_private_index(&path);
    }

    fn graph_row(method_def: &str, type_def: &str) -> String {
        let id = method_id(
            REQUIRED_MANAGED_AUTHORITY,
            REQUIRED_MANAGED_MVID,
            type_def,
            method_def,
        );
        json!({
            "method_id": id,
            "assembly_sha256": REQUIRED_MANAGED_AUTHORITY,
            "mvid": REQUIRED_MANAGED_MVID,
            "type_token": type_def,
            "method_token": method_def,
            "root_owners": [],
            "representations": [],
            "evidence_levels": {},
            "source_refs": [],
            "raw_refs": [],
            "cohorts": [],
            "limitations": []
        })
        .to_string()
            + "\n"
    }

    fn synthetic_graph() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(graph_row("0x06000002", "0x02000002").as_bytes());
        bytes.extend_from_slice(graph_row("0x06000001", "0x02000001").as_bytes());
        bytes.extend_from_slice(graph_row("0x06000003", "0x02000003").as_bytes());
        bytes
    }

    fn synthetic_index(graph: &[u8]) -> Vec<u8> {
        let mut reader = Cursor::new(graph);
        build_index_bytes(
            &mut reader,
            graph.len() as u64,
            &crate::sha256_hex(graph),
            3,
        )
        .unwrap()
    }

    fn write_fixture_graph(bytes: &[u8]) -> (PathBuf, String) {
        let root = std::env::current_dir()
            .unwrap()
            .join("target/corpus-access01-fixtures");
        fs::create_dir_all(&root).unwrap();
        let name = format!("graph-{}-{}", std::process::id(), crate::sha256_hex(bytes));
        let path = root.join(&name);
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("methods.jsonl"), bytes).unwrap();
        (path.join("methods.jsonl"), name)
    }

    #[test]
    fn index_builder_is_deterministic_and_rejects_input_hash() {
        let graph = synthetic_graph();
        let first = synthetic_index(&graph);
        let second = synthetic_index(&graph);
        assert_eq!(first, second);
        assert_eq!(first.len(), INDEX_HEADER_BYTES + 3 * INDEX_ENTRY_BYTES);
        let index_sha = crate::sha256_hex(&first);
        let (header, entries) = parse_index_bytes(
            &first,
            &index_sha,
            &crate::sha256_hex(&graph),
            graph.len() as u64,
            3,
        )
        .unwrap();
        assert_eq!(header.row_count, 3);
        assert_eq!(entries.len(), 3);
        assert!(
            entries
                .windows(2)
                .all(|pair| pair[0].method_id < pair[1].method_id)
        );
        let mut reader = Cursor::new(graph.as_slice());
        assert!(build_index_bytes(&mut reader, graph.len() as u64, &"0".repeat(64), 3).is_err());
        let mut reader = Cursor::new(graph.as_slice());
        assert!(
            build_index_bytes(
                &mut reader,
                graph.len() as u64 + 1,
                &crate::sha256_hex(&graph),
                3
            )
            .is_err()
        );
        let mut reader = Cursor::new(graph.as_slice());
        assert!(
            build_index_bytes(
                &mut reader,
                graph.len() as u64,
                &crate::sha256_hex(&graph),
                2
            )
            .is_err()
        );
    }

    #[test]
    fn index_loader_rejects_truncation_hash_duplicate_unsorted_bounds_and_pin() {
        let graph = synthetic_graph();
        let index = synthetic_index(&graph);
        let index_sha = crate::sha256_hex(&index);
        let graph_sha = crate::sha256_hex(&graph);
        let mut truncated = index.clone();
        truncated.pop();
        assert!(
            parse_index_bytes(&truncated, &index_sha, &graph_sha, graph.len() as u64, 3).is_err()
        );
        assert!(
            parse_index_bytes(&index, &"0".repeat(64), &graph_sha, graph.len() as u64, 3).is_err()
        );

        let mut duplicate = index.clone();
        let mut second_id = [0_u8; 32];
        second_id.copy_from_slice(&duplicate[INDEX_HEADER_BYTES + 76..INDEX_HEADER_BYTES + 108]);
        duplicate[INDEX_HEADER_BYTES..INDEX_HEADER_BYTES + 32].copy_from_slice(&second_id);
        assert!(
            parse_index_bytes(
                &duplicate,
                &crate::sha256_hex(&duplicate),
                &graph_sha,
                graph.len() as u64,
                3
            )
            .is_err()
        );

        let mut out_of_bounds = index.clone();
        out_of_bounds[INDEX_HEADER_BYTES + 32..INDEX_HEADER_BYTES + 40]
            .copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(
            parse_index_bytes(
                &out_of_bounds,
                &crate::sha256_hex(&out_of_bounds),
                &graph_sha,
                graph.len() as u64,
                3
            )
            .is_err()
        );
        let missing = std::env::current_dir()
            .unwrap()
            .join("target/corpus-access01-fixtures/missing.idx");
        assert_eq!(
            load_index_trusted(
                &std::env::current_dir().unwrap(),
                "target/corpus-access01-fixtures/missing.idx",
                &missing_index_trust()
            )
            .unwrap_err()
            .code,
            "TRUST_INDEX_PIN_UNAVAILABLE"
        );
        assert!(missing.to_string_lossy().contains("missing.idx"));
    }

    #[test]
    fn selected_row_rejects_rowhash_and_key_swap() {
        let graph = synthetic_graph();
        let index = synthetic_index(&graph);
        let graph_sha = crate::sha256_hex(&graph);
        let (header, entries) = parse_index_bytes(
            &index,
            &crate::sha256_hex(&index),
            &graph_sha,
            graph.len() as u64,
            3,
        )
        .unwrap();
        let loaded = LoadedIndex {
            rel_path: "fixture.idx".to_string(),
            sha256: crate::sha256_hex(&index),
            bytes: index.len() as u64,
            header,
            entries,
        };
        let (graph_path, graph_rel) = write_fixture_graph(&graph);
        let root = std::env::current_dir()
            .unwrap()
            .join("target/corpus-access01-fixtures");
        let roots = Roots {
            research: root.clone(),
            curated: root.clone(),
        };
        let mut manifest = manifest_value_for_access();
        manifest.baseline.graph.path = graph_rel;
        let target = method_id(
            REQUIRED_MANAGED_AUTHORITY,
            REQUIRED_MANAGED_MVID,
            "0x02000001",
            "0x06000001",
        );
        let (row, _) = read_indexed_row(&roots, &manifest, &loaded, &target)
            .unwrap()
            .unwrap();
        assert_eq!(row.method_id, target);

        let mut swapped = loaded.clone();
        swapped.entries[0].row_sha256 = swapped.entries[1].row_sha256;
        let swapped_target = hex_digest(swapped.entries[0].method_id);
        assert!(read_indexed_row(&roots, &manifest, &swapped, &swapped_target).is_err());
        let _ = fs::remove_file(graph_path);
    }

    fn manifest_value_for_access() -> SourceManifest {
        serde_json::from_value(json!({
            "schema": MANIFEST_SCHEMA,
            "manifest_version": 2,
            "campaign": "RE-CORPUS-ACCESS01",
            "status": "CHECKPOINT_CANDIDATE_NOT_FINAL",
            "not_final": true,
            "publication": "DISABLED_INTERNAL_ONLY",
            "baseline": {
                "snapshot": {
                    "root_id":"CURATED_ROOT","path":"snap",
                    "snapshot_manifest":{"root_id":"CURATED_ROOT","path":"snap/snapshot.json","size":1,"sha256":"b".repeat(64)},
                    "artifacts":{"root_id":"CURATED_ROOT","path":"snap/artifacts.jsonl","size":18720457,"sha256":"d805365e60cf1b1c156dceeb1b496779de082744e6d0c82dd126b1b5f452fcf6"},
                    "snapshot_id":"d805365e60cf1b1c156dceeb1b496779de082744e6d0c82dd126b1b5f452fcf6",
                    "artifact_entries":41456,"artifacts_bytes":18720457
                },
                "graph": {
                    "root_id":"CURATED_ROOT","path":"graph","graph_id":"c".repeat(64),
                    "counts":{"known_methods_all5_incl_abstract":305856},
                    "payloads":[
                        {"name":"methods.jsonl","bytes":1,"sha256":"c".repeat(64)},
                        {"name":"artifacts.links.jsonl","bytes":1,"sha256":"c".repeat(64)},
                        {"name":"relationships.jsonl","bytes":1,"sha256":"c".repeat(64)},
                        {"name":"cohorts.jsonl","bytes":1,"sha256":"c".repeat(64)},
                        {"name":"missing.jsonl","bytes":1,"sha256":"c".repeat(64)}
                    ]
                },
                "accepted_composition":{
                    "old_c_file_count":104,"old_qualified_keys":533,"new_c_file_count":1,
                    "supplemented_c_file_count":105,"supplemented_qualified_keys":537,
                    "new_method_ids":[method_id("e3fc", "mvid", "0x02000001", "0x06000001")]
                }
            },
            "non_managed_artifacts":[],
            "imports":[],
            "backlog":[],
            "family_rules":[{
                "family_id":"F","origin_class":"O","disposition_class":"D","match_kind":"OTHERWISE",
                "path_rule":"","extensions":[],"snapshot_kind":null,"snapshot_count":41456,
                "evidence_scope":"E","notes":"N"
            }],
            "limits":[]
        }))
        .unwrap()
    }

    #[test]
    fn unsafe_paths_and_unknown_roots_are_rejected() {
        let root = std::env::current_dir()
            .unwrap()
            .join("target/corpus-access01-unsafe");
        fs::create_dir_all(&root).unwrap();
        let roots = Roots {
            research: root.clone(),
            curated: root.clone(),
        };
        let mut cache = PhysCache::default();
        for path in ["../escape.cs", "C:/Windows/win.ini", "/abs/path"] {
            assert!(
                cache
                    .check(&roots, "CURATED_ROOT", path, None, &"a".repeat(64))
                    .is_err(),
                "path must be rejected: {path}"
            );
        }
        assert!(
            cache
                .check(&roots, "UNKNOWN_ROOT", "src/a.cs", None, &"a".repeat(64))
                .is_err()
        );
        let missing = cache
            .check(
                &roots,
                "CURATED_ROOT",
                "src/missing.cs",
                None,
                &"a".repeat(64),
            )
            .unwrap();
        assert!(!missing.0);
        assert_eq!(missing.1["status"], "MISSING");
        fs::write(root.join("good.cs"), b"class A {}\n").unwrap();
        let good = cache
            .check(
                &roots,
                "CURATED_ROOT",
                "good.cs",
                Some(11),
                &crate::sha256_hex(b"class A {}\n"),
            )
            .unwrap();
        assert!(good.0);
        assert_eq!(
            cache
                .check(&roots, "CURATED_ROOT", "good.cs", Some(11), &"b".repeat(64),)
                .unwrap_err()
                .code,
            "SOURCE_LOCATION_CONFLICT"
        );
    }

    #[cfg(windows)]
    #[test]
    fn real_windows_junction_ancestry_is_rejected() {
        let root = std::env::current_dir()
            .unwrap()
            .join("target/corpus-access01-junction");
        let _ = fs::remove_dir_all(&root);
        let real = root.join("real");
        fs::create_dir_all(&real).unwrap();
        fs::write(real.join("file.cs"), b"class A {}\n").unwrap();
        let junction = root.join("junction");
        let command = format!(
            "mklink /J \"{}\" \"{}\"",
            junction.display(),
            real.display()
        );
        use std::os::windows::process::CommandExt;
        let output = std::process::Command::new("cmd")
            .arg("/C")
            .raw_arg(command)
            .output()
            .expect("mkdir junction command");
        assert!(
            output.status.success(),
            "real Windows junction creation must succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let metadata = fs::symlink_metadata(&junction).unwrap();
        assert!(is_reparse(&metadata));
        let roots = Roots {
            research: root.clone(),
            curated: root.clone(),
        };
        let mut cache = PhysCache::default();
        assert!(
            cache
                .check(
                    &roots,
                    "CURATED_ROOT",
                    "junction/file.cs",
                    Some(11),
                    &crate::sha256_hex(b"class A {}\n")
                )
                .is_err()
        );
        assert!(inspect_existing_directory(&junction).is_err());
        assert_eq!(
            resolve_index_path(&root, "junction/index.idx")
                .unwrap_err()
                .code,
            "UNSAFE_REPARSE_OR_SYMLINK"
        );
        let _ = fs::remove_dir_all(&root);
    }

    fn actual_paths_for_access() -> (PathBuf, PathBuf, PathBuf) {
        let cwd = std::env::current_dir().unwrap();
        let curated = if cwd.join("metadata/source-closure.v1.json").exists() {
            cwd
        } else if cwd
            .join("github_src/metadata/source-closure.v1.json")
            .exists()
        {
            cwd.join("github_src")
        } else {
            let mut cursor = cwd.as_path();
            loop {
                if cursor.join("metadata/source-closure.v1.json").exists() {
                    break cursor.to_path_buf();
                }
                cursor = cursor.parent().expect("curated root not found");
            }
        };
        let manifest = curated.join("metadata/source-closure.v1.json");
        let research = curated.parent().unwrap().to_path_buf();
        (curated, research, manifest)
    }

    fn actual_trust(curated: &Path) -> Trust {
        let parent = fs::read(curated.join("metadata/source-closure.v1.json")).unwrap();
        let successor =
            fs::read(curated.join("metadata/source-closure.multilang.v1.json")).unwrap();
        fixture_trust_with_multilang(crate::sha256_hex(&parent), crate::sha256_hex(&successor))
    }

    fn canary(
        curated: &Path,
        research: &Path,
        manifest: &Path,
        index_rel: &str,
        index_sha: &str,
        type_def: &str,
        method_def: &str,
    ) -> (u8, Value) {
        let mut flags = BTreeMap::new();
        flags.insert("--manifest".to_string(), manifest.to_path_buf());
        flags.insert("--curated-root".to_string(), curated.to_path_buf());
        flags.insert("--research-root".to_string(), research.to_path_buf());
        flags.insert("--index".to_string(), PathBuf::from(index_rel));
        flags.insert(
            "--authority".to_string(),
            PathBuf::from("MANAGED-CS-E3FC67839851"),
        );
        flags.insert("--mvid".to_string(), PathBuf::from(REQUIRED_MANAGED_MVID));
        flags.insert("--type".to_string(), PathBuf::from(type_def));
        flags.insert("--method".to_string(), PathBuf::from(method_def));
        locate_with_trust(
            &flags,
            &actual_trust(curated),
            &fixture_index_trust(index_sha),
        )
        .unwrap()
    }

    #[test]
    fn actual_canaries_resolve_all_views_with_dev_index() {
        let (curated, research, manifest) = actual_paths_for_access();
        let index_rel = DEFAULT_INDEX_REL;
        let index_path = curated.join(index_rel);
        require_private_index(&index_path);
        let index_sha = crate::sha256_hex(&fs::read(&index_path).unwrap());
        let cases = [
            ("0x020000A3", "0x0600042E", "Event"),
            ("0x02002EB2", "0x06018868", "CalculationUnknown"),
            ("0x02002EB2", "0x06018867", "CalculationFull"),
            ("0x02000BEC", "0x06003267", "Timer"),
            ("0x020034D4", "0x0601C917", "TaskGraph"),
            ("0x020034D8", "0x0601C933", "TaskSystem"),
            ("0x02002E71", "0x06018708", "Gameplay"),
            ("0x020048E7", "0x06030B87", "Supplement"),
            ("0x020048E7", "0x06030BA8", "Callback"),
            ("0x020034D3", "0x0601C913", "Abstract"),
            ("0x02000022", "0x0600009A", "Canonical"),
            ("0x02000039", "0x060000F4", "Multiple"),
        ];
        for (type_def, method_def, label) in cases {
            let (exit, value) = canary(
                &curated, &research, &manifest, index_rel, &index_sha, type_def, method_def,
            );
            assert_eq!(
                exit,
                0,
                "{label}: {}",
                serde_json::to_string_pretty(&value).unwrap()
            );
            assert_eq!(value["method"]["method_id"].as_str().unwrap().len(), 64);
            assert_eq!(
                value["graph_row"]["method_id"],
                value["method"]["method_id"]
            );
            assert_eq!(value["verification"]["selected_row_sha256_verified"], true);
            assert_eq!(
                value["verification"]["selected_row_full_key_verified"],
                true
            );
            let locations = value["locations"].as_array().unwrap();
            assert!(!locations.is_empty(), "{label} has no locations");
            let mut ids = BTreeSet::new();
            for location in locations {
                assert!(ids.insert(location["source_id"].as_str().unwrap().to_string()));
                if location["live"]["exists"].as_bool() == Some(true) {
                    assert_eq!(
                        location["source_hash_verified"], true,
                        "{label}: {location}"
                    );
                } else {
                    assert_eq!(location["representation_class"], "MISSING");
                }
            }
            match label {
                "Event" => {
                    assert_eq!(
                        value["current_authority"]["method_evidence_status"],
                        "DIRECT_CIL_FULL"
                    );
                    assert!(locations.iter().any(|location| {
                        location["role"] == "PRIMARY_CURATED_IMPORT"
                            && location["body_availability"] == "BODY_AVAILABLE"
                    }));
                }
                "CalculationUnknown" => {
                    assert_eq!(
                        value["current_authority"]["method_evidence_status"],
                        "UNKNOWN"
                    );
                    assert!(locations.iter().any(|location| {
                        location["role"] == "PRIMARY_CURATED_IMPORT"
                            && location["body_availability"] == "BODY_UNKNOWN"
                    }));
                    assert!(locations.iter().any(|location| {
                        location["scope"] == "GRAPH_REFERENCE_NOT_ACCEPTED_IMPORT"
                    }));
                }
                "CalculationFull" => {
                    assert_eq!(
                        value["current_authority"]["method_evidence_status"],
                        "DIRECT_CIL_FULL"
                    );
                }
                "Supplement" => {
                    assert!(value["supplement"]["member"]["role"] == "ORDINARY");
                    assert_eq!(
                        value["supplement"]["provenance_status"],
                        "SOURCE_CLOSURE_ACCEPTED_SCOPED"
                    );
                    assert_eq!(
                        value["supplement"]["provenance_verification"],
                        "NOT_RECHECKED_THIS_LOOKUP"
                    );
                    assert!(locations.iter().any(|location| {
                        location["role"] == "SUPPLEMENT_MEMBER_DESTINATION"
                            && location["body_availability"] == "BODY_AVAILABLE"
                    }));
                }
                "Callback" => {
                    assert!(value["supplement"]["member"]["role"] == "ASSOCIATED");
                    assert!(locations.iter().any(|location| {
                        location["role"] == "SUPPLEMENT_ASSOCIATED_DESTINATION"
                            && location["body_availability"] == "ASSOCIATED_OR_EMBEDDED"
                    }));
                }
                "Abstract" => {
                    assert_eq!(
                        value["current_authority"]["method_evidence_status"],
                        "METADATA_ONLY"
                    );
                    assert!(locations.iter().any(|location| {
                        location["role"] == "PRIMARY_CURATED_IMPORT"
                            && location["body_availability"] == "BODY_ABSENT"
                    }));
                }
                "Canonical" => {
                    assert!(locations.iter().any(|location| {
                        location["role"] == "GRAPH_CANONICAL_SOURCE"
                            && location["representation_class"] == "CANONICAL_GENERATED"
                            && location["body_availability"] == "BODY_UNKNOWN"
                    }));
                }
                "Multiple" => {
                    assert!(locations.len() >= 2);
                }
                _ => {}
            }
        }
        let (exit, not_indexed) = canary(
            &curated,
            &research,
            &manifest,
            index_rel,
            &index_sha,
            "0x020FFFFF",
            "0x060FFFFF",
        );
        assert_eq!(exit, 1);
        assert_eq!(not_indexed["status"], "NOT_INDEXED");
        assert_eq!(not_indexed["accounting"]["indexed"], false);
        let (wrong_exit, wrong_typedef) = canary(
            &curated,
            &research,
            &manifest,
            index_rel,
            &index_sha,
            "0x02000039",
            "0x0600042E",
        );
        assert_eq!(wrong_exit, 1);
        assert_eq!(wrong_typedef["status"], "NOT_INDEXED");
    }

    #[test]
    fn checkpoint_actual_canary_report() {
        let (curated, research, manifest) = actual_paths_for_access();
        let index_rel = DEFAULT_INDEX_REL;
        let index_path = curated.join(index_rel);
        require_private_index(&index_path);
        let index_sha = crate::sha256_hex(&fs::read(&index_path).unwrap());
        let cases = [
            ("Event", "0x020000A3", "0x0600042E"),
            ("CalculationUnknown", "0x02002EB2", "0x06018868"),
            ("CalculationFull", "0x02002EB2", "0x06018867"),
            ("Timer", "0x02000BEC", "0x06003267"),
            ("TaskGraph", "0x020034D4", "0x0601C917"),
            ("TaskSystem", "0x020034D8", "0x0601C933"),
            ("Gameplay", "0x02002E71", "0x06018708"),
            ("Supplement", "0x020048E7", "0x06030B87"),
            ("Callback", "0x020048E7", "0x06030BA8"),
            ("Abstract", "0x020034D3", "0x0601C913"),
            ("Canonical", "0x02000022", "0x0600009A"),
            ("Multiple", "0x02000039", "0x060000F4"),
            ("AssocBaseDamage", "0x02002E65", "0x06018666"),
            ("AssocAsyncTask", "0x02009689", "0x0604A4E5"),
            ("EventE17", "0x020000A3", "0x06000434"),
            ("NotIndexed", "0x020FFFFF", "0x060FFFFF"),
        ];
        let mut rows = Vec::new();
        let mut event_value = None;
        for (label, type_def, method_def) in cases {
            let (exit, value) = canary(
                &curated, &research, &manifest, index_rel, &index_sha, type_def, method_def,
            );
            if label == "Event" {
                event_value = Some(value.clone());
            }
            let locations = value["locations"].as_array().cloned().unwrap_or_default();
            let mut classes: BTreeMap<String, u64> = BTreeMap::new();
            let mut bodies: BTreeMap<String, u64> = BTreeMap::new();
            let mut roles: BTreeMap<String, u64> = BTreeMap::new();
            let mut live_present = 0_u64;
            let mut live_missing = 0_u64;
            let mut hash_verified = 0_u64;
            for location in &locations {
                *classes
                    .entry(
                        location["representation_class"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                    )
                    .or_insert(0) += 1;
                *bodies
                    .entry(
                        location["body_availability"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                    )
                    .or_insert(0) += 1;
                *roles
                    .entry(location["role"].as_str().unwrap_or("").to_string())
                    .or_insert(0) += 1;
                if location["live"]["exists"].as_bool() == Some(true) {
                    live_present += 1;
                } else {
                    live_missing += 1;
                }
                if location["source_hash_verified"] == true {
                    hash_verified += 1;
                }
            }
            rows.push(json!({
                "label": label,
                "type_def": type_def,
                "method_def": method_def,
                "exit": exit,
                "status": value["status"],
                "method_id": value["method"]["method_id"],
                "graph_row_present": value["accounting"]["graph_row_present"],
                "current_evidence_status": value["current_authority"]["method_evidence_status"],
                "current_record_id": value["current_authority"]["record_id"],
                "supplement_member_role": value["supplement"]["member"]["role"],
                "alternative_current_record_ids": value["alternative_current_record_ids"],
                "current_match_count": value["exact_source_views"]["current_match_count"],
                "historical_match_count": value["exact_source_views"]["historical_match_count"],
                "association_kind": value["current_authority"]["association"]["kind"],
                "current_body_availability": value["current_authority"]["body_availability"],
                "declared_representation_refs": value["accounting"]["declared_representation_refs"],
                "accounted_representation_refs": value["accounting"]["accounted_representation_refs"],
                "unaccounted_representation_refs": value["accounting"]["unaccounted_representation_refs"],
                "locations": locations.len(),
                "classes": classes,
                "bodies": bodies,
                "roles": roles,
                "live_present": live_present,
                "live_missing": live_missing,
                "hash_verified": hash_verified,
                "index_selected_entry": value["index"]["selected_entry"],
                "graph_representation_kinds": value["graph_row"]["representation_kinds"],
            }));
        }
        let (manifest_value, manifest_bytes) = {
            let trust = actual_trust(&curated);
            load_manifest_trusted(&manifest, &trust).unwrap()
        };
        let roots = Roots {
            research: research.clone(),
            curated: curated.clone(),
        };
        let successor = multilang::load_candidate(
            &roots,
            &manifest_value,
            &manifest_bytes,
            &actual_trust(&curated),
            true,
        )
        .unwrap();
        let active = crate::load_active_receipt(&curated).unwrap().unwrap();
        let accepted = accepted_curated_paths(&roots, &manifest_value, &successor).unwrap();
        let history_proof = event_value
            .as_ref()
            .map(|value| {
                let h_path = "src/reconstructed/performance/managed/entity/Event.RangeReceiver.cs";
                let locations = value["locations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|location| location["source_path"] == h_path)
                    .map(|location| {
                        json!({
                            "role": location["role"],
                            "class": location["representation_class"],
                            "body_availability": location["body_availability"],
                            "source_hash_verified": location["source_hash_verified"],
                            "declared_sha256": location["declared_sha256"],
                        })
                    })
                    .collect::<Vec<_>>();
                json!({
                    "event_exact_current_matches": value["exact_source_views"]["current"],
                    "event_exact_historical_matches": value["exact_source_views"]["historical"],
                    "h_source_path": h_path,
                    "h_locations": locations,
                })
            })
            .unwrap_or(Value::Null);
        let report = json!({
            "command": "corpus-access01-checkpoint-canaries",
            "history_proof": history_proof,
            "index": {
                "path": index_rel,
                "bytes": fs::metadata(&index_path).unwrap().len(),
                "sha256": index_sha,
                "header": {
                    "graph_methods_sha256": ACCEPTED_METHODS_SHA256,
                    "graph_methods_bytes": ACCEPTED_METHODS_BYTES,
                    "row_count": ACCEPTED_METHOD_ROWS
                }
            },
            "canaries": rows,
            "accounting": {
                "active_curated_files": active.files.len(),
                "parent_import_destinations": manifest_value.imports.len(),
                "successor_physical_paths": successor.physical_paths().len(),
                "accepted_curated_total": accepted.len()
            }
        });
        let out = curated
            .join("tools/re/local-recovery/corpus-access01/reports/d02/ACTUAL_CANARIES_D02.json");
        fs::write(out, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }

    #[test]
    fn actual_index_sample_offsets_and_row_hashes_match_graph() {
        let (curated, research, manifest_path) = actual_paths_for_access();
        let index_path = curated.join(DEFAULT_INDEX_REL);
        require_private_index(&index_path);
        let index_bytes = fs::read(&index_path).unwrap();
        let index_sha = crate::sha256_hex(&index_bytes);
        let (header, entries) = parse_index_bytes(
            &index_bytes,
            &index_sha,
            ACCEPTED_METHODS_SHA256,
            ACCEPTED_METHODS_BYTES,
            ACCEPTED_METHOD_ROWS,
        )
        .unwrap();
        assert_eq!(header.row_count, ACCEPTED_METHOD_ROWS);
        assert_eq!(entries.len() as u64, ACCEPTED_METHOD_ROWS);
        assert_eq!(
            index_bytes.len() as u64,
            INDEX_HEADER_BYTES as u64 + ACCEPTED_METHOD_ROWS * INDEX_ENTRY_BYTES as u64
        );
        let loaded = LoadedIndex {
            rel_path: DEFAULT_INDEX_REL.to_string(),
            sha256: index_sha,
            bytes: index_bytes.len() as u64,
            header,
            entries,
        };
        let trust = actual_trust(&curated);
        let (manifest, manifest_bytes) = load_manifest_trusted(&manifest_path, &trust).unwrap();
        let roots = Roots { research, curated };
        let _ = manifest_bytes;
        let mut samples = BTreeSet::new();
        for position in [
            0,
            1,
            loaded.entries.len() / 4,
            loaded.entries.len() / 2,
            loaded.entries.len() * 3 / 4,
            loaded.entries.len() - 1,
        ] {
            samples.insert(hex_digest(loaded.entries[position].method_id));
        }
        for (type_def, method_def) in [
            ("0x020000A3", "0x0600042E"),
            ("0x02002EB2", "0x06018868"),
            ("0x020048E7", "0x06030B87"),
            ("0x02000022", "0x0600009A"),
        ] {
            samples.insert(method_id(
                REQUIRED_MANAGED_AUTHORITY,
                REQUIRED_MANAGED_MVID,
                type_def,
                method_def,
            ));
        }
        for target in samples {
            let (row, entry) = read_indexed_row(&roots, &manifest, &loaded, &target)
                .unwrap()
                .expect("sample key must be indexed");
            assert_eq!(row.method_id, target);
            assert_eq!(hex_digest(entry.method_id), target);
            assert!(entry.length > 0);
            assert!(entry.offset + u64::from(entry.length) <= ACCEPTED_METHODS_BYTES);
        }
    }

    #[test]
    fn actual_event_alternatives_and_exact_historical_sources_are_accounted() {
        let (curated, research, manifest) = actual_paths_for_access();
        let index_rel = DEFAULT_INDEX_REL;
        let index_path = curated.join(index_rel);
        require_private_index(&index_path);
        let index_sha = crate::sha256_hex(&fs::read(curated.join(index_rel)).unwrap());
        for (type_def, method_def) in [("0x020000A3", "0x0600042E"), ("0x020000A3", "0x06000434")] {
            let (exit, value) = canary(
                &curated, &research, &manifest, index_rel, &index_sha, type_def, method_def,
            );
            assert_eq!(exit, 0, "{}", serde_json::to_string_pretty(&value).unwrap());
            let current = value["exact_source_views"]["current"].as_array().unwrap();
            assert_eq!(current.len(), 2, "Event must retain E18/E17 plus H");
            let historical = &value["exact_source_views"]["historical"];
            assert_eq!(historical.as_array().unwrap().len(), 2);
            let h_current = current
                .iter()
                .find(|row| row["record_id"] == "H")
                .expect("H current exact match");
            assert_eq!(h_current["is_active_primary"], false);
            assert_eq!(h_current["class"], "HISTORICAL_ALTERNATIVE");
            assert_eq!(h_current["represented"], true);
            let h_historical = historical
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["record_id"] == "H")
                .expect("H exact historical match");
            assert_eq!(h_historical["represented"], true);
            assert_eq!(
                h_historical["source_path"],
                "src/reconstructed/performance/managed/entity/Event.RangeReceiver.cs"
            );
            let locations = value["locations"].as_array().unwrap();
            let h_location = locations
                .iter()
                .find(|location| {
                    location["role"] == "ALTERNATIVE_CURRENT_ORIGINAL"
                        && location["source_path"]
                            == "src/reconstructed/performance/managed/entity/Event.RangeReceiver.cs"
                })
                .expect("H physical source view");
            assert_eq!(h_location["representation_class"], "HISTORICAL_ALTERNATIVE");
            assert_eq!(h_location["body_availability"], "BODY_UNKNOWN");
            assert_eq!(h_location["source_hash_verified"], true);
            assert!(value["accounting"]["all_declared_representation_refs_accounted"] == true);
        }
    }

    #[test]
    fn actual_associated_bodies_are_not_independent_full_bodies() {
        let (curated, research, manifest) = actual_paths_for_access();
        let index_rel = DEFAULT_INDEX_REL;
        let index_path = curated.join(index_rel);
        require_private_index(&index_path);
        let index_sha = crate::sha256_hex(&fs::read(curated.join(index_rel)).unwrap());
        let cases = [
            (
                "0x020000A3",
                "0x0600042E",
                "ORDINARY",
                "DIRECT_CIL_FULL",
                "BODY_AVAILABLE",
            ),
            (
                "0x02002E65",
                "0x06018666",
                "GENERATED_ASSOCIATION",
                "DIRECT_CIL_FULL",
                "ASSOCIATED_OR_EMBEDDED",
            ),
            (
                "0x02009689",
                "0x0604A4E5",
                "GENERATED_ASSOCIATION",
                "SOURCE_ONLY",
                "BODY_UNKNOWN",
            ),
            (
                "0x0200968F",
                "0x0604A4F4",
                "ASSOCIATED_DECLARING_TYPE",
                "DIRECT_CIL_FULL",
                "ASSOCIATED_OR_EMBEDDED",
            ),
        ];
        for (type_def, method_def, association, evidence, body) in cases {
            let (exit, value) = canary(
                &curated, &research, &manifest, index_rel, &index_sha, type_def, method_def,
            );
            assert_eq!(exit, 0, "{}", serde_json::to_string_pretty(&value).unwrap());
            assert_eq!(
                value["current_authority"]["association"]["kind"],
                association
            );
            assert_eq!(
                value["current_authority"]["method_evidence_status"],
                evidence
            );
            assert_eq!(value["current_authority"]["body_availability"], body);
            let primary = value["locations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|location| location["role"] == "PRIMARY_CURATED_IMPORT")
                .expect("primary import location");
            assert_eq!(primary["body_availability"], body);
            assert_eq!(primary["evidence_level"], evidence);
            if association != "ORDINARY" {
                assert_ne!(primary["body_availability"], "BODY_AVAILABLE");
            }
        }
    }

    #[test]
    fn actual_graph_reference_accounting_and_range_policy() {
        let (curated, research, manifest) = actual_paths_for_access();
        let index_rel = DEFAULT_INDEX_REL;
        let index_path = curated.join(index_rel);
        require_private_index(&index_path);
        let index_sha = crate::sha256_hex(&fs::read(curated.join(index_rel)).unwrap());
        let (exit, value) = canary(
            &curated,
            &research,
            &manifest,
            index_rel,
            &index_sha,
            "0x02002EB2",
            "0x06018867",
        );
        assert_eq!(exit, 0);
        let accounting = &value["accounting"];
        assert_eq!(
            accounting["all_declared_representation_refs_accounted"],
            true
        );
        assert_eq!(accounting["unaccounted_representation_refs"], 0);
        assert_eq!(accounting["all_graph_source_refs_accounted"], true);
        assert_eq!(accounting["all_graph_raw_refs_accounted"], true);
        let semantic = value["locations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|location| location["role"] == "GRAPH_SEMANTIC_SOURCE")
            .expect("semantic source");
        assert_eq!(semantic["navigation_range"], Value::Null);
        assert_ne!(semantic["range_validation"], "VALIDATED_CURRENT_RANGE");
        if semantic["declared_source_range"].is_object() {
            assert!(
                semantic["declared_source_range"]["start_line"]
                    .as_u64()
                    .unwrap_or(0)
                    > 0
            );
        }
        if let Some(overlay) = value["locations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|location| location["role"] == "GRAPH_SEMANTIC_OVERLAY")
        {
            assert_eq!(overlay["declared_source_range"], Value::Null);
            assert!(overlay["declared_semantic_range"].is_object());
            assert_eq!(overlay["navigation_range"], Value::Null);
        }
        let view = semantic["views"]
            .as_array()
            .unwrap()
            .first()
            .expect("semantic view");
        assert_eq!(view["body_availability"], "BODY_UNKNOWN");
        assert_eq!(view["declared_body_hash_verified"], true);
        assert_eq!(view["body_presence_evidence"], "DECLARED_BODY_HASH_ONLY");
        assert!(view["representation_id"].as_str().is_some());

        let (exit, p1) = canary(
            &curated,
            &research,
            &manifest,
            index_rel,
            &index_sha,
            "0x02000013",
            "0x06000025",
        );
        assert_eq!(exit, 0);
        let p1_view = p1["locations"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|location| location["views"].as_array().unwrap())
            .find(|view| {
                view["declared_kind"] == "P1_METHOD_PROVENANCE"
                    && view["reference_role"] == "source"
            })
            .expect("P1 source view");
        assert_eq!(p1_view["body_availability"], "BODY_UNKNOWN");
        assert_eq!(
            p1_view["body_presence_evidence"],
            "DECLARED_DECOMPILED_BODY_PRESENT"
        );
    }

    #[test]
    fn non_ascii_token_input_does_not_panic() {
        for value in ["0\u{1F600}00000", "0x0\u{1F600}000"] {
            assert!(!valid_token_ci(value, "02"));
            assert!(token(Some(value.to_string()), "--type", true).is_err());
        }
        assert!(!valid_uuid_ci(
            "2d6d37cb-76e5-4075-8220-3eeca41471\u{1F600}"
        ));
    }

    #[test]
    fn publish_index_never_overwrites_or_deletes_existing_output() {
        let root = std::env::current_dir()
            .unwrap()
            .join("target/corpus-access01-publish");
        fs::create_dir_all(&root).unwrap();
        let existing = root.join("existing.idx");
        fs::write(&existing, b"existing-user-bytes").unwrap();
        let error = publish_index(&existing, b"new-index-bytes").unwrap_err();
        assert_eq!(error.code, "INDEX_OUTPUT_EXISTS");
        assert_eq!(fs::read(&existing).unwrap(), b"existing-user-bytes");
        let fresh = root.join("fresh.idx");
        let _ = fs::remove_file(&fresh);
        publish_index(&fresh, b"fresh-index-bytes").unwrap();
        assert_eq!(fs::read(&fresh).unwrap(), b"fresh-index-bytes");
        let blocked_parent = root.join("blocked-parent");
        fs::write(&blocked_parent, b"user-file").unwrap();
        let blocked = blocked_parent.join("child.idx");
        assert!(publish_index(&blocked, b"x").is_err());
        assert_eq!(fs::read(&blocked_parent).unwrap(), b"user-file");
    }

    #[test]
    fn accepted_artifact_hotpath_does_not_require_snapshot_or_links() {
        let (curated, research, manifest_path) = actual_paths_for_access();
        let trust = actual_trust(&curated);
        let (mut manifest, manifest_bytes) = load_manifest_trusted(&manifest_path, &trust).unwrap();
        let roots = Roots { research, curated };
        let successor =
            multilang::load_candidate(&roots, &manifest, &manifest_bytes, &trust, true).unwrap();
        manifest.baseline.snapshot.artifacts.path = "missing/snapshot/artifacts.jsonl".to_string();
        manifest.baseline.snapshot.snapshot_manifest.path =
            "missing/snapshot/snapshot.json".to_string();
        let (exit, value) = artifact_lookup_with_successor(
            &roots,
            &manifest,
            Some(&successor),
            "CURATED_ROOT",
            "src/scripts/typescript/declarations/config/SkillTree.ts",
            true,
        )
        .unwrap();
        assert_eq!(exit, 0);
        assert_eq!(value["graph_link"], Value::Null);
        assert_eq!(
            value["historical_registry_join"],
            "NOT_REQUESTED_EXACT_ACCEPTED_IMPORT"
        );
        assert_eq!(value["current"]["hash_matches"], true);
        assert!(
            artifact_lookup_with_successor(
                &roots,
                &manifest,
                Some(&successor),
                "CURATED_ROOT",
                "src/managed/cs-e3fc67839851/not/a/real.cs",
                true,
            )
            .is_err()
        );
    }

    #[test]
    fn actual_association_role_census_is_37_generated_plus_11_declaring() {
        let (curated, _, _) = actual_paths_for_access();
        let context = crate::load_context_at(&curated).unwrap();
        let mut generated = 0_u64;
        let mut declaring = 0_u64;
        let mut ordinary = 0_u64;
        let mut total = 0_u64;
        for record in &context.manifest.records {
            for method in &record.methods.current {
                total += 1;
                match association_kind_for(
                    &record.id,
                    record.type_def.as_deref(),
                    &method.symbol_key,
                    &method.coverage,
                    &method.verification_basis,
                ) {
                    AssociationKind::GeneratedAssociation => generated += 1,
                    AssociationKind::AssociatedDeclaringType => declaring += 1,
                    AssociationKind::Ordinary => ordinary += 1,
                }
            }
        }
        assert_eq!(generated, 37);
        assert_eq!(declaring, 11);
        assert_eq!(generated + declaring, 48);
        assert_eq!(total, 537);
        assert_eq!(ordinary, total - 48);
        for record in &context.manifest.records {
            for method in &record.methods.current {
                let kind = association_kind_for(
                    &record.id,
                    record.type_def.as_deref(),
                    &method.symbol_key,
                    &method.coverage,
                    &method.verification_basis,
                );
                let body = body_status_for(method.evidence_status.as_str(), kind);
                if kind != AssociationKind::Ordinary
                    && method.evidence_status.as_str() == "DIRECT_CIL_FULL"
                {
                    assert_eq!(body, "ASSOCIATED_OR_EMBEDDED");
                }
                if kind != AssociationKind::Ordinary
                    && method.evidence_status.as_str() != "DIRECT_CIL_FULL"
                {
                    assert_ne!(body, "BODY_AVAILABLE");
                }
            }
        }
    }

    #[test]
    fn actual_typed_provenance_anchor_counts_are_preserved() {
        let (curated, _, _) = actual_paths_for_access();
        let context = crate::load_context_at(&curated).unwrap();
        let mut exact = 0_u64;
        let mut file = 0_u64;
        let mut unresolved = 0_u64;
        let mut not_validated = 0_u64;
        let mut evidence_anchors = 0_u64;
        for record in &context.manifest.records {
            for method in &record.methods.current {
                match &method.source_provenance {
                    Some(SourceProvenance::ProfileEntry {
                        imported,
                        evidence_anchors: anchors,
                        ..
                    }) => {
                        match imported.precision {
                            Precision::ExactRange => exact += 1,
                            Precision::FileMethodAssociation => file += 1,
                            Precision::Unresolved => unresolved += 1,
                        }
                        evidence_anchors += anchors.len() as u64;
                    }
                    Some(SourceProvenance::NotValidated) => not_validated += 1,
                    None => unresolved += 1,
                }
            }
        }
        assert_eq!(exact, 231);
        assert_eq!(file, 302);
        assert_eq!(unresolved, 0);
        assert_eq!(not_validated, 4);
        assert_eq!(exact + file, 533);
        assert_eq!(exact + file + not_validated, 537);
        assert!(evidence_anchors > 0);
    }

    #[test]
    fn actual_locate_is_deterministic_and_has_no_timestamps() {
        let (curated, research, manifest) = actual_paths_for_access();
        let index_rel = DEFAULT_INDEX_REL;
        let index_path = curated.join(index_rel);
        require_private_index(&index_path);
        let index_sha = crate::sha256_hex(&fs::read(curated.join(index_rel)).unwrap());
        let first = canary(
            &curated,
            &research,
            &manifest,
            index_rel,
            &index_sha,
            "0x020000A3",
            "0x0600042E",
        );
        let second = canary(
            &curated,
            &research,
            &manifest,
            index_rel,
            &index_sha,
            "0x020000A3",
            "0x0600042E",
        );
        assert_eq!(first, second);
        let text = serde_json::to_string(&first.1).unwrap();
        for forbidden in ["timestamp", "elapsed_ms", "generated_at", "checked_at"] {
            assert!(
                !text.contains(forbidden),
                "semantic output contains {forbidden}"
            );
        }
    }

    #[test]
    fn actual_accepted_curated_refs_are_exactly_131() {
        let (curated, research, manifest_path) = actual_paths_for_access();
        let trust = actual_trust(&curated);
        let (manifest, manifest_bytes) = load_manifest_trusted(&manifest_path, &trust).unwrap();
        let roots = Roots {
            research: research.clone(),
            curated: curated.clone(),
        };
        let successor =
            multilang::load_candidate(&roots, &manifest, &manifest_bytes, &trust, true).unwrap();
        let active = crate::load_active_receipt(&curated).unwrap().unwrap();
        let active_paths = active
            .files
            .iter()
            .map(|file| file.imported.path.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(active_paths.len(), 104);
        let parent_paths = manifest
            .imports
            .iter()
            .map(|import| import.destination.path.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(parent_paths.len(), 8);
        let successor_paths = successor.physical_paths();
        assert_eq!(successor_paths.len(), 26);
        let all = accepted_curated_paths(&roots, &manifest, &successor).unwrap();
        assert_eq!(all.len(), 131);
        let mut exact = 0_u64;
        let mut existing = 0_u64;
        for path in &successor_paths {
            if successor.physical_lookup_value(path).unwrap()["physical_import"]["mode"]
                == "EXACT_BYTE_COPY"
            {
                exact += 1;
            } else {
                existing += 1;
            }
        }
        assert_eq!(exact, 19);
        assert_eq!(existing, 7);
        assert_eq!(active_paths.intersection(&parent_paths).count(), 0);
        assert_eq!(parent_paths.intersection(&successor_paths).count(), 7);
        assert_eq!(active_paths.intersection(&successor_paths).count(), 0);
    }

    #[test]
    fn actual_post_snapshot_import_lookup_resolves_131_refs() {
        let (curated, research, manifest_path) = actual_paths_for_access();
        let trust = actual_trust(&curated);
        let (manifest, manifest_bytes) = load_manifest_trusted(&manifest_path, &trust).unwrap();
        let roots = Roots {
            research: research.clone(),
            curated: curated.clone(),
        };
        let successor =
            multilang::load_candidate(&roots, &manifest, &manifest_bytes, &trust, true).unwrap();
        let (exit, value) = artifact_lookup_with_successor(
            &roots,
            &manifest,
            Some(&successor),
            "CURATED_ROOT",
            "src/scripts/typescript/declarations/config/SkillTree.ts",
            true,
        )
        .unwrap();
        assert_eq!(exit, 0);
        assert_eq!(value["baseline_scope"], "POST_SNAPSHOT_ACCEPTED_IMPORT");
        assert_eq!(value["post_snapshot_accepted"], true);
        assert_eq!(value["current"]["status"], "PRESENT");
        assert_eq!(value["current"]["hash_matches"], true);
        assert!(
            value["accepted_imports"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["kind"] == "SUCCESSOR_PHYSICAL_IMPORT")
        );

        let (exit, value) = artifact_lookup_with_successor(
            &roots,
            &manifest,
            Some(&successor),
            "CURATED_ROOT",
            "src/managed/cs-e3fc67839851/entity/RangeComponent.PendingEventDrain.cs",
            true,
        )
        .unwrap();
        assert_eq!(exit, 0);
        assert_eq!(value["current"]["hash_matches"], true);
        assert!(
            value["accepted_imports"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["kind"] == "PREDECESSOR_SUPPLEMENT_SELECTION")
        );
        assert_eq!(
            value["accepted_imports"].as_array().unwrap().len(),
            1,
            "the pending-drain import is the predecessor supplement, not an active receipt row"
        );

        assert!(
            artifact_lookup_with_successor(
                &roots,
                &manifest,
                Some(&successor),
                "CURATED_ROOT",
                "src/managed/cs-e3fc67839851/not/a/real.cs",
                true,
            )
            .is_err()
        );
    }

    #[test]
    fn production_index_trust_stays_unavailable() {
        let (curated, _, _) = actual_paths_for_access();
        // Explicit fixture missing-pin trust, not a phase assumption about the
        // now-authorized production constant.
        let error =
            load_index_trusted(&curated, DEFAULT_INDEX_REL, &missing_index_trust()).unwrap_err();
        assert_eq!(error.code, "TRUST_INDEX_PIN_UNAVAILABLE");
    }

    fn line_range(start_line: u64, end_line: u64) -> LineRange {
        LineRange {
            start_line,
            end_line,
        }
    }

    fn range_test_location(path: &str, size: u64, sha256: &str, range: LineRange) -> LocationDraft {
        LocationDraft {
            role: "CURRENT_TYPED_IMPORTED_ANCHOR".to_string(),
            representation_class: "PRIMARY_CURATED".to_string(),
            declared_class: "PRIMARY_CURATED".to_string(),
            root_id: "CURATED_ROOT".to_string(),
            path: path.to_string(),
            declared_size: Some(size),
            declared_sha256: sha256.to_string(),
            range: Some(range),
            body_availability: "BODY_AVAILABLE".to_string(),
            evidence_level: "TYPED_TEST_RANGE".to_string(),
            provenance_status: "TEST".to_string(),
            scope: Some("RANGE_TEST".to_string()),
            views: vec![ViewDraft {
                kind: "RANGE_TEST".to_string(),
                declared_coverage: None,
                evidence: vec!["RANGE:CURRENT_TYPED".to_string()],
                body_availability: "BODY_AVAILABLE".to_string(),
                provenance_status: Some("TEST".to_string()),
                limitations: Vec::new(),
                evidence_class: None,
                precision: Some("EXACT_RANGE".to_string()),
                body_sha256: None,
                body_hash_verified: None,
                support_refs: Vec::new(),
            }],
            limitations: Vec::new(),
        }
    }

    #[cfg(windows)]
    #[test]
    fn range_bearing_location_through_junction_is_rejected() {
        let root = std::env::current_dir()
            .unwrap()
            .join("target/corpus-access01-d02-junction");
        let _ = fs::remove_dir_all(&root);
        let real = root.join("real");
        fs::create_dir_all(&real).unwrap();
        let bytes = b"one\ntwo\nthree\n";
        fs::write(real.join("file.cs"), bytes).unwrap();
        let junction = root.join("link");
        let command = format!(
            "mklink /J \"{}\" \"{}\"",
            junction.display(),
            real.display()
        );
        use std::os::windows::process::CommandExt;
        let output = std::process::Command::new("cmd")
            .arg("/C")
            .raw_arg(command)
            .output()
            .expect("mkdir junction command");
        assert!(
            output.status.success(),
            "real Windows junction creation must succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let roots = Roots {
            research: root.clone(),
            curated: root.clone(),
        };
        let mut locator = Locator::default();
        locator.add(range_test_location(
            "link/file.cs",
            bytes.len() as u64,
            &crate::sha256_hex(bytes),
            line_range(1, 2),
        ));
        let mut cache = PhysCache::default();
        let error = locator.finalize(&roots, &mut cache).unwrap_err();
        assert_eq!(error.code, "UNSAFE_REPARSE_OR_SYMLINK");
    }

    #[test]
    fn missing_and_wrong_hash_ranges_do_not_claim_navigation() {
        let root = std::env::current_dir()
            .unwrap()
            .join("target/corpus-access01-d02-range");
        fs::create_dir_all(&root).unwrap();
        let roots = Roots {
            research: root.clone(),
            curated: root.clone(),
        };
        let mut locator = Locator::default();
        locator.add(range_test_location(
            "missing.cs",
            10,
            &"a".repeat(64),
            line_range(1, 2),
        ));
        let mut cache = PhysCache::default();
        let (locations, missing, _) = locator.finalize(&roots, &mut cache).unwrap();
        assert_eq!(missing, 1);
        assert_eq!(locations[0]["navigation_range"], Value::Null);
        assert!(locations[0]["declared_source_range"].is_object());
        assert_eq!(
            locations[0]["range_validation"],
            "SOURCE_UNREADABLE_FOR_RANGE"
        );
        assert_eq!(locations[0]["representation_class"], "MISSING");

        let wrong_path = root.join("wrong.cs");
        fs::write(&wrong_path, b"a\nb\n").unwrap();
        let mut locator = Locator::default();
        locator.add(range_test_location(
            "wrong.cs",
            4,
            &"b".repeat(64),
            line_range(1, 1),
        ));
        let mut cache = PhysCache::default();
        let error = locator.finalize(&roots, &mut cache).unwrap_err();
        assert_eq!(error.code, "CURRENT_ARTIFACT_HASH_MISMATCH");
    }

    #[test]
    fn d02_historical_only_keys_census() {
        let (curated, _, _) = actual_paths_for_access();
        let context = crate::load_context_at(&curated).unwrap();
        fn key4(identity: &MethodIdentity) -> (String, String, String, String) {
            (
                identity.full_authority_sha256.to_ascii_lowercase(),
                identity.mvid.to_ascii_lowercase(),
                identity.type_def.to_ascii_uppercase(),
                identity.method_def.to_ascii_uppercase(),
            )
        }
        let mut current = BTreeSet::new();
        let mut historical = Vec::new();
        for record in &context.manifest.records {
            for method in &record.methods.current {
                current.insert(key4(&method.symbol_key));
            }
            for method in &record.methods.historical {
                historical.push((record, method));
            }
        }
        let historical_only = historical
            .iter()
            .filter(|(_, method)| !current.contains(&key4(&method.symbol_key)))
            .map(|(record, method)| {
                json!({
                    "record_id": record.id,
                    "symbol_key": method.symbol_key,
                    "name": method.name,
                    "coverage": method.coverage,
                    "source_path": record.original_path,
                    "source_sha256": record.original_sha256,
                })
            })
            .collect::<Vec<_>>();
        let report = json!({
            "schema": "wuwa-reconstruction-corpus-access01-d02-historical-only-v1",
            "current_exact_keys": current.len(),
            "historical_entries": historical.len(),
            "historical_only_count": historical_only.len(),
            "historical_only_keys": historical_only,
            "scope": "small accepted metadata context; no graph/index scan",
        });
        let out = curated.join(
            "tools/re/local-recovery/corpus-access01/reports/d02/HISTORICAL_ONLY_KEYS_D02.json",
        );
        fs::create_dir_all(out.parent().unwrap()).unwrap();
        fs::write(out, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
        assert_eq!(historical_only.len(), 0);
    }

    #[test]
    fn partial_missing_keeps_good_views_but_is_nonzero() {
        let root = std::env::current_dir()
            .unwrap()
            .join("target/corpus-access01-partial");
        fs::create_dir_all(&root).unwrap();
        let bytes = b"class A {}\n";
        fs::write(root.join("good.cs"), bytes).unwrap();
        let roots = Roots {
            research: root.clone(),
            curated: root.clone(),
        };
        let mut locator = Locator::default();
        locator.add(LocationDraft {
            role: "GOOD".to_string(),
            representation_class: "PRIMARY_CURATED".to_string(),
            declared_class: "PRIMARY_CURATED".to_string(),
            root_id: "CURATED_ROOT".to_string(),
            path: "good.cs".to_string(),
            declared_size: Some(bytes.len() as u64),
            declared_sha256: crate::sha256_hex(bytes),
            range: None,
            body_availability: "BODY_AVAILABLE".to_string(),
            evidence_level: "DIRECT_CIL_FULL".to_string(),
            provenance_status: "TEST".to_string(),
            scope: None,
            views: Vec::new(),
            limitations: Vec::new(),
        });
        locator.add(LocationDraft {
            role: "MISSING".to_string(),
            representation_class: "SUPPLEMENTARY".to_string(),
            declared_class: "SUPPLEMENTARY".to_string(),
            root_id: "CURATED_ROOT".to_string(),
            path: "missing.cs".to_string(),
            declared_size: Some(1),
            declared_sha256: "a".repeat(64),
            range: None,
            body_availability: "BODY_UNKNOWN".to_string(),
            evidence_level: "TEST".to_string(),
            provenance_status: "TEST".to_string(),
            scope: None,
            views: Vec::new(),
            limitations: Vec::new(),
        });
        let mut cache = PhysCache::default();
        let (locations, missing, _) = locator.finalize(&roots, &mut cache).unwrap();
        assert_eq!(missing, 1);
        assert_eq!(locations.len(), 2);
        assert_eq!(
            access_status(&locations, missing),
            (1, "PARTIAL_MISSING_BACKING")
        );
        assert!(
            locations
                .iter()
                .any(|location| location["source_hash_verified"] == true)
        );
        assert!(
            locations
                .iter()
                .any(|location| location["representation_class"] == "MISSING")
        );
    }
}
