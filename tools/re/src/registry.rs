use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

mod graph;
mod source_closure;

pub const CONFIG_SCHEMA: &str = "wuwa-reconstruction-corpus-registry-config-v1";
pub const SNAPSHOT_SCHEMA: &str = "wuwa-reconstruction-corpus-registry-snapshot-v1";
const ROOT_IDS: [&str; 2] = ["RESEARCH_ROOT", "CURATED_ROOT"];
const TASK_ROOT_REASON: &str = "TASK_ROOT_RECURSION_GUARD";

#[derive(Debug)]
struct RegistryError {
    code: &'static str,
    detail: String,
}

impl RegistryError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

type RegistryResult<T> = Result<T, RegistryError>;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    schema: String,
    roots: Vec<RootEntry>,
    #[serde(default)]
    exclude_paths: Vec<ExcludeEntry>,
    #[serde(default)]
    exclude_directory_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RootEntry {
    id: String,
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExcludeEntry {
    root_id: String,
    path: String,
    reason: String,
}

#[derive(Debug)]
struct Config {
    config_dir: PathBuf,
    roots: Vec<Root>,
    excludes: Vec<ExcludeRule>,
    exclude_dir_names: Vec<String>,
}

#[derive(Debug)]
struct Root {
    id: String,
    path: PathBuf,
    skip_rel: BTreeSet<String>,
}

#[derive(Debug)]
struct ExcludeRule {
    root_id: String,
    rel: String,
    reason: String,
    measure: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Observation {
    size: u64,
    mtime: u64,
    identity: Option<(u32, u32, u32)>,
}

#[derive(Debug)]
struct HashedFile {
    sha256: String,
    size: u64,
    mtime: u64,
    identity_checked: bool,
    reason: Option<&'static str>,
}

#[derive(Debug)]
struct ScanOutcome {
    artifacts: Vec<ArtifactRow>,
    errors: Vec<ErrorRow>,
    warnings: Vec<String>,
    exclusions: Vec<ExclusionActual>,
    root_stats: BTreeMap<String, RootStat>,
    kind_counts: BTreeMap<String, u64>,
    identity_checked: u64,
    identity_stable: u64,
    unstable: u64,
    membership: BTreeMap<String, (u64, u64)>,
    closure: ClosureSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactRow {
    artifact_id: String,
    root_id: String,
    path: String,
    size: u64,
    sha256: Option<String>,
    status: String,
    kind: String,
    backup_coverage: String,
    error_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ErrorRow {
    root_id: String,
    path: String,
    code: String,
    detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExclusionActual {
    root_id: String,
    path: String,
    reason: String,
    measured: bool,
    files: Option<u64>,
    bytes: Option<u64>,
    reparse_rejected: u64,
    measure_errors: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RootStat {
    root_id: String,
    entries: u64,
    hashed_ok: u64,
    bytes: u64,
    unstable: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClosureSummary {
    first_entries: u64,
    second_entries: u64,
    added: Vec<PathRef>,
    removed: Vec<PathRef>,
    metadata_changed: Vec<PathRef>,
    second_errors: u64,
    status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PathRef {
    root_id: String,
    path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RootView {
    root_id: String,
    relative_marker: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RootRelationship {
    ancestor_root_id: String,
    descendant_root_id: String,
    relative_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StabilitySummary {
    method: String,
    identity_checked: u64,
    identity_stable: u64,
    files_unstable: u64,
    hash_method: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObservedSummary {
    entries: u64,
    hashed_ok: u64,
    unstable: u64,
    errors: u64,
    bytes: u64,
    root_membership: Vec<RootStat>,
    kinds: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExclusionTotals {
    target_files: u64,
    pytest_files: u64,
    ruff_files: u64,
    pycache_dirs: u64,
    pycache_files: u64,
    task_root_files: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackupCoverage {
    unknown: u64,
    note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotManifest {
    schema: String,
    task: String,
    phase: String,
    mode: String,
    snapshot_id: String,
    snapshot_id_basis: String,
    artifact_entries: u64,
    artifacts_bytes: u64,
    status: String,
    roots: Vec<RootView>,
    root_relationship: Vec<RootRelationship>,
    observed: ObservedSummary,
    exclusions: Vec<ExclusionActual>,
    exclusion_totals: ExclusionTotals,
    stability: StabilitySummary,
    closure: ClosureSummary,
    errors: Vec<ErrorRow>,
    warnings: Vec<String>,
    content_inspection: String,
    serialization: Value,
    backup_coverage: BackupCoverage,
    self_validation: String,
}

#[derive(Debug, Serialize)]
struct ExecutionReceipt {
    schema: String,
    command: String,
    started_unix_ms: u64,
    finished_unix_ms: u64,
    elapsed_ms: u64,
    status: String,
    exit: u8,
}

struct OutputDir {
    path: PathBuf,
}

impl OutputDir {
    fn create(config: &Config, output: &Path) -> RegistryResult<Self> {
        if !output.is_absolute() {
            return Err(RegistryError::new(
                "OUTPUT_NOT_ABSOLUTE",
                "output must be an absolute path",
            ));
        }
        reject_parent_components(output, "OUTPUT_PATH_UNSAFE")?;
        if !path_is_descendant_ci(output, &config.config_dir)
            || path_eq_ci(output, &config.config_dir)
        {
            return Err(RegistryError::new(
                "OUTPUT_OUTSIDE_CONFIG_DIRECTORY",
                "output must be a descendant of the private config directory",
            ));
        }
        for root in &config.roots {
            if path_is_descendant_ci(root.path.as_path(), output)
                || path_eq_ci(root.path.as_path(), output)
            {
                return Err(RegistryError::new(
                    "OUTPUT_CONTAINS_SCANNED_ROOT",
                    "output must not be an ancestor of a scanned root",
                ));
            }
        }
        match fs::symlink_metadata(output) {
            Ok(_) => {
                return Err(RegistryError::new(
                    "OUTPUT_ALREADY_EXISTS",
                    "output directory already exists",
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => {
                return Err(RegistryError::new(
                    "OUTPUT_STAT_ERROR",
                    "output path could not be inspected",
                ));
            }
        }
        let parent = output.parent().ok_or_else(|| {
            RegistryError::new("OUTPUT_PARENT_MISSING", "output parent is not available")
        })?;
        inspect_existing_directory(parent)?;
        fs::create_dir(output).map_err(|_| {
            RegistryError::new(
                "OUTPUT_CREATE_ERROR",
                "output directory could not be created",
            )
        })?;
        inspect_existing_directory(output)?;
        Ok(Self {
            path: output.to_path_buf(),
        })
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn validate_absolute_normal_path(path: &Path) -> RegistryResult<()> {
    if !path.is_absolute() {
        return Err(RegistryError::new(
            "ROOT_PATH_NOT_ABSOLUTE",
            "root path must be absolute",
        ));
    }
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {}
            Component::CurDir | Component::ParentDir => {
                return Err(RegistryError::new(
                    "ROOT_PATH_UNSAFE",
                    "root path must not contain . or .. components",
                ));
            }
        }
    }
    Ok(())
}

fn reject_parent_components(path: &Path, code: &'static str) -> RegistryResult<()> {
    for component in path.components() {
        if matches!(component, Component::CurDir | Component::ParentDir) {
            return Err(RegistryError::new(
                code,
                "path must not contain . or .. components",
            ));
        }
    }
    Ok(())
}

fn components_ci(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
        .collect()
}

fn path_eq_ci(left: &Path, right: &Path) -> bool {
    components_ci(left) == components_ci(right)
}

fn path_is_descendant_ci(path: &Path, ancestor: &Path) -> bool {
    let path_components = components_ci(path);
    let ancestor_components = components_ci(ancestor);
    path_components.len() > ancestor_components.len()
        && path_components
            .iter()
            .zip(ancestor_components.iter())
            .all(|(left, right)| left == right)
}

fn relative_components(path: &Path, ancestor: &Path) -> Option<Vec<String>> {
    let path_components = components_ci(path);
    let ancestor_components = components_ci(ancestor);
    if path_components.len() <= ancestor_components.len() {
        return None;
    }
    if !path_components
        .iter()
        .zip(ancestor_components.iter())
        .all(|(left, right)| left == right)
    {
        return None;
    }
    Some(path_components[ancestor_components.len()..].to_vec())
}

fn is_reparse(metadata: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        crate::reparse_attributes_rejected(metadata.file_attributes())
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn metadata_mtime(metadata: &Metadata) -> u64 {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.last_write_time()
    }
    #[cfg(not(windows))]
    {
        use std::time::UNIX_EPOCH;
        metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }
}

fn inspect_existing_directory(path: &Path) -> RegistryResult<()> {
    let mut ancestors: Vec<&Path> = path.ancestors().collect();
    ancestors.reverse();
    for ancestor in ancestors {
        let metadata = fs::symlink_metadata(ancestor).map_err(|_| {
            RegistryError::new("PATH_METADATA_ERROR", "path metadata could not be read")
        })?;
        if is_reparse(&metadata) {
            return Err(RegistryError::new(
                "UNSAFE_REPARSE_OR_SYMLINK",
                "reparse point or symlink is not allowed",
            ));
        }
        if ancestor != path && !metadata.is_dir() {
            return Err(RegistryError::new(
                "PATH_PARENT_NOT_DIRECTORY",
                "an ancestor is not a directory",
            ));
        }
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        RegistryError::new("PATH_METADATA_ERROR", "path metadata could not be read")
    })?;
    if is_reparse(&metadata) {
        return Err(RegistryError::new(
            "UNSAFE_REPARSE_OR_SYMLINK",
            "reparse point or symlink is not allowed",
        ));
    }
    if !metadata.is_dir() {
        return Err(RegistryError::new(
            "PATH_NOT_DIRECTORY",
            "path is not a directory",
        ));
    }
    Ok(())
}

fn inspect_existing_file(path: &Path) -> RegistryResult<()> {
    let mut ancestors: Vec<&Path> = path.ancestors().collect();
    ancestors.reverse();
    for ancestor in ancestors {
        let metadata = fs::symlink_metadata(ancestor).map_err(|_| {
            RegistryError::new("PATH_METADATA_ERROR", "path metadata could not be read")
        })?;
        if is_reparse(&metadata) {
            return Err(RegistryError::new(
                "UNSAFE_REPARSE_OR_SYMLINK",
                "reparse point or symlink is not allowed",
            ));
        }
        if ancestor != path && !metadata.is_dir() {
            return Err(RegistryError::new(
                "PATH_PARENT_NOT_DIRECTORY",
                "an ancestor is not a directory",
            ));
        }
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        RegistryError::new("PATH_METADATA_ERROR", "path metadata could not be read")
    })?;
    if is_reparse(&metadata) {
        return Err(RegistryError::new(
            "UNSAFE_REPARSE_OR_SYMLINK",
            "reparse point or symlink is not allowed",
        ));
    }
    if !metadata.is_file() {
        return Err(RegistryError::new(
            "PATH_NOT_FILE",
            "path is not a regular file",
        ));
    }
    Ok(())
}

fn inspect_relative_entry(root: &Path, relative: &Path) -> Result<EntryKind, &'static str> {
    if let Some(text) = relative.to_str() {
        if text.is_ascii() {
            let text = text.replace('\\', "/");
            return match crate::inspect_relative(root, &text) {
                Ok(crate::SafePathState::File) => Ok(EntryKind::File),
                Ok(crate::SafePathState::Directory) => Ok(EntryKind::Directory),
                Ok(crate::SafePathState::Missing) => Err("MISSING_PATH"),
                Err(error) => Err(match error.code.as_str() {
                    "UNSAFE_REPARSE_OR_SYMLINK" => "UNSAFE_REPARSE_OR_SYMLINK",
                    "PATH_PARENT_NOT_DIRECTORY" => "PATH_PARENT_NOT_DIRECTORY",
                    "PATH_NOT_REGULAR" => "PATH_NOT_REGULAR",
                    "MISSING_PATH" => "MISSING_PATH",
                    "LSTAT_ERROR" => "LSTAT_ERROR",
                    _ => "UNSAFE_RELATIVE_PATH",
                }),
            };
        }
    }
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err("UNSAFE_RELATIVE_PATH");
        };
        let text = name.to_string_lossy();
        if text.is_ascii() {
            if crate::validate_relative_path(text.as_ref()).is_err() {
                return Err("UNSAFE_RELATIVE_PATH");
            }
        } else if text.is_empty()
            || text == "."
            || text == ".."
            || text.ends_with('.')
            || text.ends_with(' ')
            || text
                .chars()
                .any(|character| character.is_control() || "<>\"|?*".contains(character))
        {
            return Err("UNSAFE_RELATIVE_PATH");
        }
        current.push(name);
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                "MISSING_PATH"
            } else {
                "LSTAT_ERROR"
            }
        })?;
        if is_reparse(&metadata) {
            return Err("UNSAFE_REPARSE_OR_SYMLINK");
        }
        if !metadata.is_dir() && current != root.join(relative) {
            return Err("PATH_PARENT_NOT_DIRECTORY");
        }
    }
    let metadata = fs::symlink_metadata(root.join(relative)).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            "MISSING_PATH"
        } else {
            "LSTAT_ERROR"
        }
    })?;
    if metadata.is_dir() {
        Ok(EntryKind::Directory)
    } else if metadata.is_file() {
        Ok(EntryKind::File)
    } else {
        Err("PATH_NOT_REGULAR")
    }
}

fn load_config(path: &Path) -> RegistryResult<Config> {
    if !path.is_absolute() {
        return Err(RegistryError::new(
            "CONFIG_NOT_ABSOLUTE",
            "config path must be absolute",
        ));
    }
    reject_parent_components(path, "CONFIG_PATH_UNSAFE")?;
    inspect_existing_file(path)?;
    let bytes = fs::read(path)
        .map_err(|_| RegistryError::new("CONFIG_READ_ERROR", "config could not be read"))?;
    let value = crate::parse_strict_value(&bytes).map_err(|_| {
        RegistryError::new(
            "CONFIG_JSON_INVALID",
            "config JSON is invalid or contains duplicate keys",
        )
    })?;
    let file: ConfigFile = serde_json::from_value(value).map_err(|_| {
        RegistryError::new("CONFIG_JSON_INVALID", "config JSON could not be parsed")
    })?;
    if file.schema != CONFIG_SCHEMA {
        return Err(RegistryError::new(
            "CONFIG_SCHEMA_INVALID",
            "config schema is not the recognized version",
        ));
    }
    if file.roots.is_empty() {
        return Err(RegistryError::new(
            "CONFIG_ROOTS_EMPTY",
            "at least one root is required",
        ));
    }
    let mut roots = Vec::with_capacity(file.roots.len());
    let mut seen_ids = BTreeSet::new();
    for entry in &file.roots {
        if !ROOT_IDS.contains(&entry.id.as_str()) {
            return Err(RegistryError::new(
                "CONFIG_ROOT_ID_UNKNOWN",
                "root id is not a recognized portable id",
            ));
        }
        if !seen_ids.insert(entry.id.clone()) {
            return Err(RegistryError::new(
                "CONFIG_ROOT_ID_DUPLICATE",
                "root id is duplicated",
            ));
        }
        let root_path = PathBuf::from(&entry.path);
        validate_absolute_normal_path(&root_path)?;
        inspect_existing_directory(&root_path)?;
        roots.push(Root {
            id: entry.id.clone(),
            path: root_path,
            skip_rel: BTreeSet::new(),
        });
    }
    let research = roots.iter().find(|root| root.id == "RESEARCH_ROOT");
    let curated = roots.iter().find(|root| root.id == "CURATED_ROOT");
    if let (Some(research), Some(curated)) = (research, curated) {
        if !path_is_descendant_ci(&curated.path, &research.path) {
            return Err(RegistryError::new(
                "CONFIG_CURATED_NOT_UNDER_RESEARCH",
                "CURATED_ROOT must be a descendant of RESEARCH_ROOT when both are configured",
            ));
        }
    }
    for (index, root) in roots.iter().enumerate() {
        for (other_index, other) in roots.iter().enumerate() {
            if index == other_index {
                continue;
            }
            if path_eq_ci(&root.path, &other.path) {
                return Err(RegistryError::new(
                    "CONFIG_ROOT_PATH_DUPLICATE",
                    "root paths must be distinct even when ids differ",
                ));
            }
            if path_is_descendant_ci(&other.path, &root.path) {
                let allowed = root.id == "RESEARCH_ROOT" && other.id == "CURATED_ROOT";
                if !allowed {
                    return Err(RegistryError::new(
                        "CONFIG_ROOT_OVERLAP_NOT_ALLOWED",
                        "only the intended RESEARCH_ROOT -> CURATED_ROOT split may overlap",
                    ));
                }
            }
        }
    }
    let root_paths: Vec<(String, PathBuf)> = roots
        .iter()
        .map(|root| (root.id.clone(), root.path.clone()))
        .collect();
    for root in &mut roots {
        for (other_id, other_path) in &root_paths {
            if &root.id == other_id {
                continue;
            }
            if path_is_descendant_ci(other_path, &root.path) {
                if let Some(parts) = relative_components(other_path, &root.path) {
                    if !parts.is_empty() {
                        root.skip_rel.insert(parts.join("/"));
                    }
                }
            }
        }
    }
    let mut excludes = Vec::new();
    let mut seen_excludes = BTreeSet::new();
    for entry in &file.exclude_paths {
        if !roots.iter().any(|root| root.id == entry.root_id) {
            return Err(RegistryError::new(
                "CONFIG_EXCLUDE_ROOT_UNKNOWN",
                "exclude_path root_id is not configured",
            ));
        }
        crate::validate_relative_path(&entry.path).map_err(|_| {
            RegistryError::new(
                "CONFIG_EXCLUDE_PATH_UNSAFE",
                "exclude_path is not a safe relative path",
            )
        })?;
        if entry.reason.is_empty()
            || !entry
                .reason
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(RegistryError::new(
                "CONFIG_EXCLUDE_REASON_INVALID",
                "exclude reason must be uppercase ASCII with digits or underscores",
            ));
        }
        let key = format!("{}\0{}", entry.root_id, entry.path.to_lowercase());
        if !seen_excludes.insert(key) {
            return Err(RegistryError::new(
                "CONFIG_EXCLUDE_DUPLICATE",
                "exclude_path is duplicated",
            ));
        }
        excludes.push(ExcludeRule {
            root_id: entry.root_id.clone(),
            rel: entry.path.replace('\\', "/"),
            reason: entry.reason.clone(),
            measure: entry.reason != TASK_ROOT_REASON,
        });
    }
    let mut exclude_dir_names = Vec::new();
    let mut seen_names = BTreeSet::new();
    for name in &file.exclude_directory_names {
        crate::validate_relative_path(name).map_err(|_| {
            RegistryError::new(
                "CONFIG_EXCLUDE_DIRECTORY_NAME_UNSAFE",
                "exclude directory name is not a safe relative component",
            )
        })?;
        if name.contains('/') {
            return Err(RegistryError::new(
                "CONFIG_EXCLUDE_DIRECTORY_NAME_UNSAFE",
                "exclude directory name must be a single component",
            ));
        }
        let lower = name.to_lowercase();
        if !seen_names.insert(lower.clone()) {
            return Err(RegistryError::new(
                "CONFIG_EXCLUDE_DIRECTORY_NAME_DUPLICATE",
                "exclude directory name is duplicated",
            ));
        }
        exclude_dir_names.push(lower);
    }
    let config_dir = path
        .parent()
        .ok_or_else(|| RegistryError::new("CONFIG_DIRECTORY_MISSING", "config has no parent"))?
        .to_path_buf();
    inspect_existing_directory(&config_dir)?;
    Ok(Config {
        config_dir,
        roots,
        excludes,
        exclude_dir_names,
    })
}

fn classify(rel: &str) -> &'static str {
    let lower = rel.to_lowercase();
    let extension = Path::new(&lower)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if extension == "bak" {
        return "backup";
    }
    if lower.starts_with("tools/") {
        return "tooling";
    }
    if lower.starts_with("evidence/")
        || lower.starts_with("metadata/")
        || lower.starts_with("catalog/")
    {
        return "evidence";
    }
    if lower.starts_with("protocol/") {
        return "protocol";
    }
    const GOVERNANCE: [&str; 11] = [
        "agents.md",
        "license",
        "provenance.md",
        "program_state.md",
        "methodology.md",
        "build_matrix.md",
        "readme.md",
        "third_party_notices.md",
        "contributing.md",
        ".gitignore",
        "opencode.json",
    ];
    if GOVERNANCE.contains(&lower.as_str()) {
        return "governance";
    }
    if matches!(
        extension,
        "rs" | "cs"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "py"
            | "csx"
            | "c"
            | "h"
            | "cpp"
            | "hpp"
            | "cc"
    ) {
        return "source";
    }
    if matches!(extension, "o" | "d" | "rmeta" | "rlib" | "timestamp") {
        return "generated";
    }
    "opaque"
}

fn artifact_id(root_id: &str, rel: &str) -> String {
    crate::sha256_hex(format!("{root_id}\0{rel}").as_bytes())
}

fn hex_digest(digest: impl AsRef<[u8]>) -> String {
    let mut output = String::with_capacity(64);
    for byte in digest.as_ref() {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn hash_reader<R: Read>(reader: &mut R) -> io::Result<(String, u64)> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut total = 0u64;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        total += read as u64;
    }
    Ok((hex_digest(hasher.finalize()), total))
}

#[cfg(windows)]
fn file_identity(file: &File) -> io::Result<(u32, u32, u32)> {
    use std::ffi::c_void;
    use std::os::windows::io::AsRawHandle;

    #[repr(C)]
    #[allow(dead_code)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    #[allow(dead_code)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    unsafe extern "system" {
        fn GetFileInformationByHandle(
            handle: *mut c_void,
            information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    let mut information: ByHandleFileInformation = unsafe { std::mem::zeroed() };
    let result = unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut information) };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((
        information.volume_serial_number,
        information.file_index_high,
        information.file_index_low,
    ))
}

#[cfg(not(windows))]
fn file_identity(_file: &File) -> io::Result<(u32, u32, u32)> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "native file identity is not available on this platform",
    ))
}

fn observe(file: &File) -> io::Result<Observation> {
    let metadata = file.metadata()?;
    Ok(Observation {
        size: metadata.len(),
        mtime: metadata_mtime(&metadata),
        identity: file_identity(file).ok(),
    })
}

fn metadata_observation(metadata: &Metadata) -> Observation {
    Observation {
        size: metadata.len(),
        mtime: metadata_mtime(metadata),
        identity: None,
    }
}

fn stability_reason(
    before: &Observation,
    after: &Observation,
    read_len: u64,
    post: &Observation,
) -> Option<&'static str> {
    if before.size != read_len {
        return Some("SIZE_MISMATCH_READ");
    }
    if before.size != after.size || before.mtime != after.mtime {
        return Some("METADATA_CHANGED_DURING_READ");
    }
    if let (Some(left), Some(right)) = (before.identity, after.identity) {
        if left != right {
            return Some("FILE_IDENTITY_CHANGED");
        }
    }
    if post.size != after.size || post.mtime != after.mtime {
        return Some("METADATA_CHANGED_AFTER_CLOSE");
    }
    None
}

fn hash_file(path: &Path) -> io::Result<HashedFile> {
    let mut file = File::open(path)?;
    let before = observe(&file)?;
    let (sha256, read_len) = hash_reader(&mut file)?;
    let after = observe(&file)?;
    let identity_checked = before.identity.is_some() && after.identity.is_some();
    drop(file);
    let post = fs::symlink_metadata(path)?;
    if is_reparse(&post) || !post.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file became a reparse point or non-regular file",
        ));
    }
    let post_observation = metadata_observation(&post);
    let reason = stability_reason(&before, &after, read_len, &post_observation);
    Ok(HashedFile {
        sha256,
        size: post_observation.size,
        mtime: post_observation.mtime,
        identity_checked,
        reason,
    })
}

fn measure_tree(root_id: &str, path: &Path, rel: &str, kind: EntryKind) -> ExclusionActual {
    let mut files = 0u64;
    let mut bytes = 0u64;
    let mut reparse_rejected = 0u64;
    let mut measure_errors = 0u64;
    let mut stack = vec![(path.to_path_buf(), rel.to_string())];
    if kind == EntryKind::File {
        if let Ok(metadata) = fs::symlink_metadata(path) {
            files = 1;
            bytes = metadata.len();
        } else {
            measure_errors += 1;
        }
        return ExclusionActual {
            root_id: root_id.to_string(),
            path: rel.to_string(),
            reason: String::new(),
            measured: true,
            files: Some(files),
            bytes: Some(bytes),
            reparse_rejected,
            measure_errors,
        };
    }
    while let Some((directory, directory_rel)) = stack.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => {
                measure_errors += 1;
                continue;
            }
        };
        for entry in entries {
            let Ok(entry) = entry else {
                measure_errors += 1;
                continue;
            };
            let name = entry.file_name().to_string_lossy().to_string();
            let child_rel = if directory_rel.is_empty() {
                name
            } else {
                format!("{directory_rel}/{name}")
            };
            let metadata = match fs::symlink_metadata(entry.path()) {
                Ok(metadata) => metadata,
                Err(_) => {
                    measure_errors += 1;
                    continue;
                }
            };
            if is_reparse(&metadata) {
                reparse_rejected += 1;
                continue;
            }
            if metadata.is_dir() {
                stack.push((entry.path(), child_rel));
            } else {
                files += 1;
                bytes += metadata.len();
            }
        }
    }
    ExclusionActual {
        root_id: root_id.to_string(),
        path: rel.to_string(),
        reason: String::new(),
        measured: true,
        files: Some(files),
        bytes: Some(bytes),
        reparse_rejected,
        measure_errors,
    }
}

fn exclusion_for(
    config: &Config,
    root_id: &str,
    rel: &str,
    name: &str,
    kind: EntryKind,
) -> Option<ExcludeRule> {
    for rule in &config.excludes {
        if rule.root_id == root_id && rule.rel.eq_ignore_ascii_case(rel) {
            return Some(ExcludeRule {
                root_id: rule.root_id.clone(),
                rel: rule.rel.clone(),
                reason: rule.reason.clone(),
                measure: rule.measure,
            });
        }
    }
    if kind == EntryKind::Directory {
        let lower = name.to_lowercase();
        if config.exclude_dir_names.contains(&lower) {
            return Some(ExcludeRule {
                root_id: root_id.to_string(),
                rel: rel.to_string(),
                reason: "PYCACHE_DIRECTORY".to_string(),
                measure: true,
            });
        }
    }
    None
}

fn read_directory_sorted(path: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        entries.push(entry?);
    }
    entries.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
    Ok(entries)
}

fn scan_once(
    config: &Config,
    output_dir: Option<&Path>,
    hash_files: bool,
) -> RegistryResult<ScanOutcome> {
    let mut artifacts = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut exclusions = Vec::new();
    let mut root_stats = BTreeMap::new();
    let mut kind_counts = BTreeMap::new();
    let mut identity_checked = 0u64;
    let mut identity_stable = 0u64;
    let mut unstable = 0u64;
    let mut membership = BTreeMap::new();

    for root in &config.roots {
        inspect_existing_directory(&root.path)?;
        let mut stat = RootStat {
            root_id: root.id.clone(),
            entries: 0,
            hashed_ok: 0,
            bytes: 0,
            unstable: 0,
        };
        let mut stack = vec![(root.path.clone(), String::new())];
        while let Some((directory, directory_rel)) = stack.pop() {
            let entries = match read_directory_sorted(&directory) {
                Ok(entries) => entries,
                Err(_) => {
                    errors.push(ErrorRow {
                        root_id: root.id.clone(),
                        path: if directory_rel.is_empty() {
                            ".".to_string()
                        } else {
                            directory_rel.clone()
                        },
                        code: "DIR_ENUM_ERROR".to_string(),
                        detail: "directory could not be enumerated".to_string(),
                    });
                    continue;
                }
            };
            for entry in entries {
                let name = entry.file_name().to_string_lossy().to_string();
                let rel = if directory_rel.is_empty() {
                    name.clone()
                } else {
                    format!("{directory_rel}/{name}")
                };
                if directory_rel.is_empty() && root.skip_rel.contains(&rel.to_lowercase()) {
                    continue;
                }
                let metadata = match fs::symlink_metadata(entry.path()) {
                    Ok(metadata) => metadata,
                    Err(_) => {
                        errors.push(ErrorRow {
                            root_id: root.id.clone(),
                            path: rel,
                            code: "ENTRY_STAT_ERROR".to_string(),
                            detail: "entry metadata could not be read".to_string(),
                        });
                        continue;
                    }
                };
                if is_reparse(&metadata) {
                    errors.push(ErrorRow {
                        root_id: root.id.clone(),
                        path: rel,
                        code: "REPARSE_OBJECT_REJECTED".to_string(),
                        detail: "not traversed, not read, not hashed".to_string(),
                    });
                    continue;
                }
                let kind = if metadata.is_dir() {
                    EntryKind::Directory
                } else if metadata.is_file() {
                    EntryKind::File
                } else {
                    errors.push(ErrorRow {
                        root_id: root.id.clone(),
                        path: rel,
                        code: "ENTRY_NOT_REGULAR".to_string(),
                        detail: "entry is not a regular file or directory".to_string(),
                    });
                    continue;
                };
                if let Some(output) = output_dir {
                    if path_eq_ci(entry.path().as_path(), output)
                        || path_is_descendant_ci(entry.path().as_path(), output)
                    {
                        exclusions.push(ExclusionActual {
                            root_id: root.id.clone(),
                            path: rel,
                            reason: "SCAN_OUTPUT_RECURSION_GUARD".to_string(),
                            measured: true,
                            files: Some(0),
                            bytes: Some(0),
                            reparse_rejected: 0,
                            measure_errors: 0,
                        });
                        continue;
                    }
                }
                if let Some(rule) = exclusion_for(config, &root.id, &rel, &name, kind) {
                    let mut actual = if rule.measure {
                        measure_tree(&root.id, &entry.path(), &rel, kind)
                    } else {
                        ExclusionActual {
                            root_id: root.id.clone(),
                            path: rel.clone(),
                            reason: rule.reason.clone(),
                            measured: false,
                            files: None,
                            bytes: None,
                            reparse_rejected: 0,
                            measure_errors: 0,
                        }
                    };
                    actual.reason = rule.reason;
                    if actual.measure_errors > 0 {
                        warnings.push(format!(
                            "excluded measurement had {} errors at {}",
                            actual.measure_errors, actual.path
                        ));
                    }
                    exclusions.push(actual);
                    continue;
                }
                if kind == EntryKind::Directory {
                    stack.push((entry.path(), rel));
                    continue;
                }
                let relative_path = Path::new(&rel);
                if let Err(code) = inspect_relative_entry(&root.path, relative_path) {
                    errors.push(ErrorRow {
                        root_id: root.id.clone(),
                        path: rel,
                        code: code.to_string(),
                        detail: "safe-path inspection rejected the entry".to_string(),
                    });
                    continue;
                }
                let key = format!("{}\0{}", root.id, rel);
                if membership.contains_key(&key) {
                    errors.push(ErrorRow {
                        root_id: root.id.clone(),
                        path: rel,
                        code: "DUPLICATE_ENUMERATION_KEY".to_string(),
                        detail: "same root and path enumerated twice".to_string(),
                    });
                    continue;
                }
                let mut size = metadata.len();
                let mut mtime = metadata_mtime(&metadata);
                let mut status = "OK".to_string();
                let mut sha256 = None;
                let mut error_code = None;
                if hash_files {
                    match hash_file(&entry.path()) {
                        Ok(hashed) => {
                            size = hashed.size;
                            mtime = hashed.mtime;
                            sha256 = Some(hashed.sha256);
                            if hashed.identity_checked {
                                identity_checked += 1;
                            }
                            match hashed.reason {
                                None => {
                                    identity_stable += 1;
                                    stat.hashed_ok += 1;
                                }
                                Some(reason) => {
                                    status = "UNSTABLE".to_string();
                                    error_code = Some(reason.to_string());
                                    unstable += 1;
                                    stat.unstable += 1;
                                    errors.push(ErrorRow {
                                        root_id: root.id.clone(),
                                        path: rel.clone(),
                                        code: "UNSTABLE_FILE".to_string(),
                                        detail: reason.to_string(),
                                    });
                                }
                            }
                        }
                        Err(_) => {
                            status = "ERROR".to_string();
                            error_code = Some("READ_OR_HASH_ERROR".to_string());
                            errors.push(ErrorRow {
                                root_id: root.id.clone(),
                                path: rel.clone(),
                                code: "READ_OR_HASH_ERROR".to_string(),
                                detail: "file could not be read or hashed".to_string(),
                            });
                        }
                    }
                }
                membership.insert(key, (size, mtime));
                stat.entries += 1;
                stat.bytes += size;
                *kind_counts.entry(classify(&rel).to_string()).or_insert(0) += 1;
                let kind_name = classify(&rel).to_string();
                if hash_files {
                    artifacts.push(ArtifactRow {
                        artifact_id: artifact_id(&root.id, &rel),
                        root_id: root.id.clone(),
                        path: rel,
                        size,
                        sha256,
                        status,
                        kind: kind_name,
                        backup_coverage: "UNKNOWN".to_string(),
                        error_code,
                    });
                }
            }
        }
        root_stats.insert(root.id.clone(), stat);
    }

    if hash_files {
        let mut casefold = BTreeMap::<String, Vec<(String, String)>>::new();
        for artifact in &artifacts {
            let key = format!(
                "{}\0{}",
                artifact.root_id.to_lowercase(),
                artifact.path.to_lowercase()
            );
            casefold
                .entry(key)
                .or_default()
                .push((artifact.root_id.clone(), artifact.path.clone()));
        }
        for (_, paths) in casefold {
            if paths.len() > 1 {
                let first = &paths[0];
                errors.push(ErrorRow {
                    root_id: first.0.clone(),
                    path: first.1.clone(),
                    code: "CASE_FOLD_COLLISION".to_string(),
                    detail: format!("collides with {} other artifact path(s)", paths.len() - 1),
                });
            }
        }
    }

    Ok(ScanOutcome {
        artifacts,
        errors,
        warnings,
        exclusions,
        root_stats,
        kind_counts,
        identity_checked,
        identity_stable,
        unstable,
        membership,
        closure: ClosureSummary {
            first_entries: 0,
            second_entries: 0,
            added: Vec::new(),
            removed: Vec::new(),
            metadata_changed: Vec::new(),
            second_errors: 0,
            status: "PENDING".to_string(),
        },
    })
}

fn compute_closure(
    first: &mut ScanOutcome,
    config: &Config,
    output_dir: Option<&Path>,
) -> RegistryResult<()> {
    let second = scan_once(config, output_dir, false)?;
    let first_keys: BTreeSet<String> = first.membership.keys().cloned().collect();
    let second_keys: BTreeSet<String> = second.membership.keys().cloned().collect();
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut metadata_changed = Vec::new();
    for key in &second_keys {
        if !first_keys.contains(key) {
            if let Some((root_id, path)) = key.split_once('\0') {
                added.push(PathRef {
                    root_id: root_id.to_string(),
                    path: path.to_string(),
                });
            }
        }
    }
    for key in &first_keys {
        if !second_keys.contains(key) {
            if let Some((root_id, path)) = key.split_once('\0') {
                removed.push(PathRef {
                    root_id: root_id.to_string(),
                    path: path.to_string(),
                });
            }
        } else if first.membership.get(key) != second.membership.get(key) {
            if let Some((root_id, path)) = key.split_once('\0') {
                metadata_changed.push(PathRef {
                    root_id: root_id.to_string(),
                    path: path.to_string(),
                });
            }
        }
    }
    let status = if added.is_empty()
        && removed.is_empty()
        && metadata_changed.is_empty()
        && second.errors.is_empty()
    {
        "STABLE"
    } else {
        "DRIFT"
    };
    first.closure = ClosureSummary {
        first_entries: first_keys.len() as u64,
        second_entries: second_keys.len() as u64,
        added,
        removed,
        metadata_changed,
        second_errors: second.errors.len() as u64,
        status: status.to_string(),
    };
    if first.closure.status != "STABLE" {
        first.errors.push(ErrorRow {
            root_id: config
                .roots
                .first()
                .map(|root| root.id.clone())
                .unwrap_or_else(|| "SYSTEM".to_string()),
            path: ".".to_string(),
            code: "CLOSURE_DRIFT".to_string(),
            detail: "membership or metadata changed between first pass and closure".to_string(),
        });
    }
    Ok(())
}

fn root_views(config: &Config) -> Vec<RootView> {
    let mut views = Vec::new();
    for root in &config.roots {
        let marker = config
            .roots
            .iter()
            .filter(|other| other.id != root.id)
            .filter_map(|other| relative_components(&root.path, &other.path))
            .map(|parts| parts.join("/"))
            .find(|value| !value.is_empty())
            .unwrap_or_else(|| ".".to_string());
        views.push(RootView {
            root_id: root.id.clone(),
            relative_marker: marker,
        });
    }
    views.sort_by(|left, right| left.root_id.cmp(&right.root_id));
    views
}

fn root_relationships(config: &Config) -> Vec<RootRelationship> {
    let mut relationships = Vec::new();
    for root in &config.roots {
        for other in &config.roots {
            if root.id == other.id {
                continue;
            }
            if let Some(parts) = relative_components(&other.path, &root.path) {
                if !parts.is_empty() {
                    relationships.push(RootRelationship {
                        ancestor_root_id: root.id.clone(),
                        descendant_root_id: other.id.clone(),
                        relative_path: parts.join("/"),
                    });
                }
            }
        }
    }
    relationships.sort_by(|left, right| {
        left.ancestor_root_id
            .cmp(&right.ancestor_root_id)
            .then_with(|| left.descendant_root_id.cmp(&right.descendant_root_id))
    });
    relationships
}

fn exclusion_totals(exclusions: &[ExclusionActual]) -> ExclusionTotals {
    let sum_files = |reason: &str| -> u64 {
        exclusions
            .iter()
            .filter(|entry| entry.reason == reason)
            .filter_map(|entry| entry.files)
            .sum()
    };
    let pycache = exclusions
        .iter()
        .filter(|entry| entry.reason == "PYCACHE_DIRECTORY")
        .collect::<Vec<_>>();
    ExclusionTotals {
        target_files: sum_files("BUILD_OUTPUT_TARGET"),
        pytest_files: sum_files("PYTEST_CACHE"),
        ruff_files: sum_files("RUFF_CACHE"),
        pycache_dirs: pycache.len() as u64,
        pycache_files: pycache.iter().filter_map(|entry| entry.files).sum(),
        task_root_files: exclusions
            .iter()
            .find(|entry| entry.reason == TASK_ROOT_REASON)
            .and_then(|entry| entry.files),
    }
}

fn build_manifest(
    config: &Config,
    artifacts: &[ArtifactRow],
    artifacts_bytes: u64,
    snapshot_id: &str,
    outcome: &ScanOutcome,
) -> SnapshotManifest {
    let mut root_membership: Vec<RootStat> = outcome.root_stats.values().cloned().collect();
    root_membership.sort_by(|left, right| left.root_id.cmp(&right.root_id));
    let observed_bytes = outcome.membership.values().map(|(size, _)| *size).sum();
    let hashed_ok = outcome.root_stats.values().map(|stat| stat.hashed_ok).sum();
    let status = if outcome.errors.is_empty() && outcome.closure.status == "STABLE" {
        "PASS"
    } else {
        "INCOMPLETE"
    };
    SnapshotManifest {
        schema: SNAPSHOT_SCHEMA.to_string(),
        task: "RE-CORPUS-PRESERVE01".to_string(),
        phase: "A".to_string(),
        mode: "FULL_STREAM_HASH".to_string(),
        snapshot_id: snapshot_id.to_string(),
        snapshot_id_basis: "SHA256 of the exact artifacts.jsonl bytes; candidate registry identity, not a self-validating proof".to_string(),
        artifact_entries: artifacts.len() as u64,
        artifacts_bytes,
        status: status.to_string(),
        roots: root_views(config),
        root_relationship: root_relationships(config),
        observed: ObservedSummary {
            entries: artifacts.len() as u64,
            hashed_ok,
            unstable: outcome.unstable,
            errors: outcome.errors.len() as u64,
            bytes: observed_bytes,
            root_membership,
            kinds: outcome.kind_counts.clone(),
        },
        exclusions: outcome.exclusions.clone(),
        exclusion_totals: exclusion_totals(&outcome.exclusions),
        stability: StabilitySummary {
            method: "FileStream-like std::fs::File read + pre/post metadata; GetFileInformationByHandle identity on Windows; post-close size/mtime check".to_string(),
            identity_checked: outcome.identity_checked,
            identity_stable: outcome.identity_stable,
            files_unstable: outcome.unstable,
            hash_method: "streamed SHA-256, 1 MiB read buffer".to_string(),
        },
        closure: outcome.closure.clone(),
        errors: outcome.errors.clone(),
        warnings: outcome.warnings.clone(),
        content_inspection: "NONE; archive contents were not read, unpacked, or analyzed; .bin/.dll/.zip were hashed as opaque bytes when included.".to_string(),
        serialization: json!({
            "artifact_order": "root_id ordinal then slash-relative path ordinal (UTF-8 bytes)",
            "artifact_json": "compact JSON object per line; stable struct field order; UTF-8; LF; no absolute paths",
            "portable": "root_id + slash-relative path only; no host absolute path, user profile, agent id, or volume/file-index values in portable rows"
        }),
        backup_coverage: BackupCoverage {
            unknown: artifacts.len() as u64,
            note: "Registry hashes are not backups. No independent backup is claimed.".to_string(),
        },
        self_validation: "snapshot_id is a candidate SHA256 of artifacts.jsonl; verify recomputes it from bytes. The manifest does not prove itself.".to_string(),
    }
}

fn write_exclusive(path: &Path, bytes: &[u8]) -> RegistryResult<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| {
            RegistryError::new("SNAPSHOT_WRITE_ERROR", "snapshot file could not be created")
        })?;
    file.write_all(bytes).map_err(|_| {
        RegistryError::new("SNAPSHOT_WRITE_ERROR", "snapshot file could not be written")
    })?;
    file.sync_all().map_err(|_| {
        RegistryError::new("SNAPSHOT_WRITE_ERROR", "snapshot file could not be synced")
    })?;
    Ok(())
}

fn write_artifacts(path: &Path, artifacts: &[ArtifactRow]) -> RegistryResult<(u64, String)> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| {
            RegistryError::new(
                "SNAPSHOT_WRITE_ERROR",
                "artifacts file could not be created",
            )
        })?;
    let mut writer = BufWriter::new(file);
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    for artifact in artifacts {
        let line = serde_json::to_string(artifact).map_err(|_| {
            RegistryError::new(
                "SNAPSHOT_SERIALIZE_ERROR",
                "artifact row could not be serialized",
            )
        })?;
        writer.write_all(line.as_bytes()).map_err(|_| {
            RegistryError::new("SNAPSHOT_WRITE_ERROR", "artifact row could not be written")
        })?;
        writer.write_all(b"\n").map_err(|_| {
            RegistryError::new(
                "SNAPSHOT_WRITE_ERROR",
                "artifact row terminator could not be written",
            )
        })?;
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
        bytes += line.len() as u64 + 1;
    }
    writer.flush().map_err(|_| {
        RegistryError::new(
            "SNAPSHOT_WRITE_ERROR",
            "artifacts file could not be flushed",
        )
    })?;
    let file = writer.into_inner().map_err(|_| {
        RegistryError::new(
            "SNAPSHOT_WRITE_ERROR",
            "artifacts file could not be finalized",
        )
    })?;
    file.sync_all().map_err(|_| {
        RegistryError::new("SNAPSHOT_WRITE_ERROR", "artifacts file could not be synced")
    })?;
    Ok((bytes, hex_digest(hasher.finalize())))
}

fn run_scan(config_path: &Path, output: &Path) -> RegistryResult<u8> {
    let started = now_unix_ms();
    let config = load_config(config_path)?;
    let output_dir = OutputDir::create(&config, output)?;
    let mut outcome = scan_once(&config, Some(&output_dir.path), true)?;
    compute_closure(&mut outcome, &config, Some(&output_dir.path))?;
    outcome.errors.sort_by(|left, right| {
        left.root_id
            .cmp(&right.root_id)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.code.cmp(&right.code))
    });
    outcome.artifacts.sort_by(|left, right| {
        left.root_id
            .cmp(&right.root_id)
            .then_with(|| left.path.cmp(&right.path))
    });
    let artifacts_path = output_dir.path.join("artifacts.jsonl");
    let (artifacts_bytes, snapshot_id) = write_artifacts(&artifacts_path, &outcome.artifacts)?;
    let manifest = build_manifest(
        &config,
        &outcome.artifacts,
        artifacts_bytes,
        &snapshot_id,
        &outcome,
    );
    let manifest_bytes = serde_json::to_vec(&manifest).map_err(|_| {
        RegistryError::new(
            "SNAPSHOT_SERIALIZE_ERROR",
            "snapshot manifest could not be serialized",
        )
    })?;
    write_exclusive(&output_dir.path.join("snapshot.json"), &manifest_bytes)?;
    let errors_bytes = serde_json::to_vec(&outcome.errors).map_err(|_| {
        RegistryError::new("SNAPSHOT_SERIALIZE_ERROR", "errors could not be serialized")
    })?;
    write_exclusive(&output_dir.path.join("errors.json"), &errors_bytes)?;
    let exit = if manifest.status == "PASS" { 0 } else { 1 };
    let finished = now_unix_ms();
    let execution = ExecutionReceipt {
        schema: "wuwa-reconstruction-corpus-registry-execution-v1".to_string(),
        command: "scan".to_string(),
        started_unix_ms: started,
        finished_unix_ms: finished,
        elapsed_ms: finished.saturating_sub(started),
        status: manifest.status.clone(),
        exit,
    };
    let execution_bytes = serde_json::to_vec(&execution).map_err(|_| {
        RegistryError::new(
            "SNAPSHOT_SERIALIZE_ERROR",
            "execution receipt could not be serialized",
        )
    })?;
    write_exclusive(&output_dir.path.join("execution.json"), &execution_bytes)?;
    let cli_status = manifest.status.clone();
    let cli_snapshot_id = manifest.snapshot_id.clone();
    let cli_counts = json!({
        "artifacts": manifest.artifact_entries,
        "hashed_ok": manifest.observed.hashed_ok,
        "errors": manifest.observed.errors,
        "bytes": manifest.observed.bytes,
        "unstable": manifest.observed.unstable
    });
    let cli_findings: Vec<Value> = manifest
        .errors
        .iter()
        .map(|error| {
            json!({
                "code": error.code.clone(),
                "root_id": error.root_id.clone(),
                "path": error.path.clone()
            })
        })
        .collect();
    print_cli(json!({
        "command": "scan",
        "ok": exit == 0,
        "exit": exit,
        "status": cli_status,
        "snapshot_id": cli_snapshot_id,
        "counts": cli_counts,
        "findings": cli_findings
    }));
    Ok(exit)
}

fn validate_snapshot_path(path: &str) -> bool {
    if path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path.contains(':')
        || !path
            .split('/')
            .all(|component| !component.is_empty() && component != "." && component != "..")
    {
        return false;
    }
    if path.is_ascii() && crate::validate_relative_path(path).is_err() {
        return false;
    }
    true
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_artifact_row(row: &ArtifactRow) -> RegistryResult<()> {
    if !ROOT_IDS.contains(&row.root_id.as_str()) {
        return Err(RegistryError::new(
            "SNAPSHOT_ROW_INVALID",
            "artifact root_id is not recognized",
        ));
    }
    if !validate_snapshot_path(&row.path) {
        return Err(RegistryError::new(
            "SNAPSHOT_ROW_INVALID",
            "artifact path is not a safe portable relative path",
        ));
    }
    if !is_lower_hex_64(&row.artifact_id) || artifact_id(&row.root_id, &row.path) != row.artifact_id
    {
        return Err(RegistryError::new(
            "SNAPSHOT_ROW_INVALID",
            "artifact_id is not the canonical lower-hex identity",
        ));
    }
    if !matches!(row.status.as_str(), "OK" | "UNSTABLE" | "ERROR") {
        return Err(RegistryError::new(
            "SNAPSHOT_ROW_INVALID",
            "artifact status is not recognized",
        ));
    }
    const KINDS: [&str; 8] = [
        "source",
        "evidence",
        "protocol",
        "governance",
        "generated",
        "tooling",
        "backup",
        "opaque",
    ];
    if !KINDS.contains(&row.kind.as_str()) {
        return Err(RegistryError::new(
            "SNAPSHOT_ROW_INVALID",
            "artifact kind is not recognized",
        ));
    }
    if row.backup_coverage != "UNKNOWN" {
        return Err(RegistryError::new(
            "SNAPSHOT_ROW_INVALID",
            "artifact backup_coverage must be UNKNOWN",
        ));
    }
    match row.status.as_str() {
        "OK" => {
            if row.error_code.is_some()
                || !row.sha256.as_deref().map(is_lower_hex_64).unwrap_or(false)
            {
                return Err(RegistryError::new(
                    "SNAPSHOT_ROW_INVALID",
                    "OK row must have a lower-hex sha256 and no error_code",
                ));
            }
        }
        "UNSTABLE" => {
            if row.error_code.as_deref().map(str::is_empty).unwrap_or(true)
                || !row.sha256.as_deref().map(is_lower_hex_64).unwrap_or(false)
            {
                return Err(RegistryError::new(
                    "SNAPSHOT_ROW_INVALID",
                    "UNSTABLE row must have sha256 and a non-empty error_code",
                ));
            }
        }
        "ERROR" => {
            if row.sha256.is_some() || row.error_code.as_deref().map(str::is_empty).unwrap_or(true)
            {
                return Err(RegistryError::new(
                    "SNAPSHOT_ROW_INVALID",
                    "ERROR row must have no sha256 and a non-empty error_code",
                ));
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn load_snapshot(
    snapshot: &Path,
) -> RegistryResult<(SnapshotManifest, Vec<ArtifactRow>, String, u64)> {
    if !snapshot.is_absolute() {
        return Err(RegistryError::new(
            "SNAPSHOT_NOT_ABSOLUTE",
            "snapshot path must be absolute",
        ));
    }
    reject_parent_components(snapshot, "SNAPSHOT_PATH_UNSAFE")?;
    inspect_existing_directory(snapshot)?;
    let manifest_path = snapshot.join("snapshot.json");
    inspect_existing_file(&manifest_path)?;
    let manifest_bytes = fs::read(&manifest_path).map_err(|_| {
        RegistryError::new(
            "SNAPSHOT_MANIFEST_MISSING",
            "snapshot manifest could not be read",
        )
    })?;
    let manifest_value = crate::parse_strict_value(&manifest_bytes).map_err(|_| {
        RegistryError::new(
            "SNAPSHOT_MANIFEST_INVALID",
            "snapshot manifest JSON is invalid or contains duplicate keys",
        )
    })?;
    let manifest: SnapshotManifest = serde_json::from_value(manifest_value).map_err(|_| {
        RegistryError::new(
            "SNAPSHOT_MANIFEST_INVALID",
            "snapshot manifest shape is invalid",
        )
    })?;
    if manifest.schema != SNAPSHOT_SCHEMA {
        return Err(RegistryError::new(
            "SNAPSHOT_SCHEMA_INVALID",
            "snapshot schema is not recognized",
        ));
    }
    let artifacts_path = snapshot.join("artifacts.jsonl");
    inspect_existing_file(&artifacts_path)?;
    let artifacts_bytes = fs::read(&artifacts_path).map_err(|_| {
        RegistryError::new(
            "SNAPSHOT_ARTIFACTS_MISSING",
            "snapshot artifacts could not be read",
        )
    })?;
    let computed = crate::sha256_hex(&artifacts_bytes);
    let text = std::str::from_utf8(&artifacts_bytes).map_err(|_| {
        RegistryError::new(
            "SNAPSHOT_ARTIFACTS_INVALID",
            "snapshot artifacts are not UTF-8",
        )
    })?;
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let value = crate::parse_strict_value(line.as_bytes()).map_err(|_| {
            RegistryError::new(
                "SNAPSHOT_ARTIFACTS_INVALID",
                "snapshot artifact row is invalid or contains duplicate keys",
            )
        })?;
        let row: ArtifactRow = serde_json::from_value(value).map_err(|_| {
            RegistryError::new(
                "SNAPSHOT_ARTIFACTS_INVALID",
                "snapshot artifact row shape is invalid",
            )
        })?;
        validate_artifact_row(&row)?;
        rows.push(row);
    }
    if manifest.artifact_entries != rows.len() as u64 {
        return Err(RegistryError::new(
            "SNAPSHOT_ENTRY_COUNT_MISMATCH",
            "snapshot manifest entry count does not match artifact rows",
        ));
    }
    let artifacts_len = artifacts_bytes.len() as u64;
    Ok((manifest, rows, computed, artifacts_len))
}

fn run_verify(config_path: &Path, snapshot: &Path) -> RegistryResult<u8> {
    let config = load_config(config_path)?;
    let (manifest, snapshot_rows, computed, artifacts_len) = load_snapshot(snapshot)?;
    let mut findings = Vec::new();
    let mut structurally_valid = true;
    if manifest.snapshot_id != computed {
        structurally_valid = false;
        findings.push(json!({
            "code": "SNAPSHOT_ID_MISMATCH",
            "detail": "snapshot_id does not match the artifacts.jsonl bytes"
        }));
    }
    if manifest.status != "PASS" {
        structurally_valid = false;
        findings.push(json!({
            "code": "SNAPSHOT_STATUS_NOT_PASS",
            "detail": "snapshot is not a complete PASS snapshot"
        }));
    }
    let mut previous_key: Option<(String, String)> = None;
    let mut casefold_keys = BTreeSet::new();
    for row in &snapshot_rows {
        let key = (row.root_id.clone(), row.path.clone());
        if let Some(previous) = &previous_key {
            if previous >= &key {
                structurally_valid = false;
                findings.push(json!({
                    "code": "NON_CANONICAL_ORDER",
                    "root_id": row.root_id.clone(),
                    "path": row.path.clone()
                }));
            }
        }
        previous_key = Some(key);
        let casefold = format!(
            "{}\0{}",
            row.root_id.to_lowercase(),
            row.path.to_lowercase()
        );
        if !casefold_keys.insert(casefold) {
            structurally_valid = false;
            findings.push(json!({
                "code": "CASE_FOLD_ALIAS",
                "root_id": row.root_id.clone(),
                "path": row.path.clone()
            }));
        }
    }
    let mut snapshot_map: BTreeMap<String, ArtifactRow> = BTreeMap::new();
    for row in &snapshot_rows {
        let key = format!("{}\0{}", row.root_id, row.path);
        if snapshot_map.insert(key, row.clone()).is_some() {
            structurally_valid = false;
            findings.push(json!({
                "code": "SNAPSHOT_DUPLICATE_KEY",
                "root_id": row.root_id.clone(),
                "path": row.path.clone()
            }));
        }
    }
    let mut outcome = scan_once(&config, None, true)?;
    compute_closure(&mut outcome, &config, None)?;
    outcome.artifacts.sort_by(|left, right| {
        left.root_id
            .cmp(&right.root_id)
            .then_with(|| left.path.cmp(&right.path))
    });
    let rows_exact = snapshot_rows == outcome.artifacts;
    if !rows_exact {
        structurally_valid = false;
        findings.push(json!({
            "code": "SNAPSHOT_ROW_SEMANTIC_MISMATCH",
            "detail": "one or more artifact rows differ in a derived field, order, or completeness"
        }));
    }
    let expected_manifest = build_manifest(
        &config,
        &outcome.artifacts,
        artifacts_len,
        &computed,
        &outcome,
    );
    let manifest_exact = manifest == expected_manifest;
    if !manifest_exact {
        structurally_valid = false;
        findings.push(json!({
            "code": "SNAPSHOT_MANIFEST_SEMANTIC_MISMATCH",
            "detail": "snapshot manifest metadata does not match freshly derived metadata"
        }));
    }
    let current_map: BTreeMap<String, &ArtifactRow> = outcome
        .artifacts
        .iter()
        .map(|row| (format!("{}\0{}", row.root_id, row.path), row))
        .collect();
    let mut changed = 0u64;
    let mut missing = 0u64;
    let mut new = 0u64;
    for (key, snapshot_row) in &snapshot_map {
        match current_map.get(key) {
            None => {
                missing += 1;
                findings.push(json!({
                    "code": "MISSING",
                    "root_id": snapshot_row.root_id.clone(),
                    "path": snapshot_row.path.clone()
                }));
            }
            Some(current) => {
                if *current != snapshot_row {
                    changed += 1;
                    findings.push(json!({
                        "code": "CHANGED",
                        "root_id": snapshot_row.root_id.clone(),
                        "path": snapshot_row.path.clone()
                    }));
                }
            }
        }
    }
    for (key, current) in &current_map {
        if !snapshot_map.contains_key(key) {
            new += 1;
            findings.push(json!({
                "code": "NEW",
                "root_id": current.root_id.clone(),
                "path": current.path.clone()
            }));
        }
    }
    for error in &outcome.errors {
        structurally_valid = false;
        findings.push(json!({
            "code": error.code.clone(),
            "root_id": error.root_id.clone(),
            "path": error.path.clone()
        }));
    }
    if outcome.closure.status != "STABLE" {
        structurally_valid = false;
    }
    let exact = structurally_valid
        && rows_exact
        && manifest_exact
        && changed == 0
        && missing == 0
        && new == 0
        && outcome.errors.is_empty();
    let exit = if exact { 0 } else { 1 };
    let header_snapshot = manifest.snapshot_id.clone();
    let snapshot_id_matches = header_snapshot == computed;
    print_cli(json!({
        "command": "verify",
        "ok": exact,
        "exit": exit,
        "status": if exact { "EXACT_MATCH" } else { "MISMATCH" },
        "snapshot_id": header_snapshot,
        "snapshot_id_recomputed": computed,
        "snapshot_id_matches": snapshot_id_matches,
        "manifest_semantics_checked": true,
        "row_fields_checked": [
            "artifact_id", "root_id", "path", "size", "sha256", "status", "kind",
            "backup_coverage", "error_code"
        ],
        "counts": {
            "snapshot": snapshot_rows.len(),
            "current": outcome.artifacts.len(),
            "changed": changed,
            "missing": missing,
            "new": new,
            "errors": outcome.errors.len()
        },
        "findings": findings
    }));
    Ok(exit)
}

fn parse_flags(args: &[OsString], allowed: &[&str]) -> RegistryResult<BTreeMap<String, PathBuf>> {
    let mut values = BTreeMap::new();
    let mut index = 0;
    while index < args.len() {
        let flag = args[index]
            .to_str()
            .ok_or_else(|| RegistryError::new("NON_UTF8_ARGUMENT", "arguments must be UTF-8"))?;
        if !allowed.contains(&flag) {
            return Err(RegistryError::new(
                "UNKNOWN_ARGUMENT",
                format!("unrecognized argument: {flag}"),
            ));
        }
        if index + 1 >= args.len() {
            return Err(RegistryError::new(
                "MISSING_ARGUMENT_VALUE",
                format!("{flag} requires a value"),
            ));
        }
        if values.contains_key(flag) {
            return Err(RegistryError::new(
                "DUPLICATE_ARGUMENT",
                format!("{flag} was provided more than once"),
            ));
        }
        let value = args[index + 1]
            .to_str()
            .ok_or_else(|| RegistryError::new("NON_UTF8_ARGUMENT", "arguments must be UTF-8"))?;
        values.insert(flag.to_string(), PathBuf::from(value));
        index += 2;
    }
    Ok(values)
}

fn print_cli(value: Value) {
    match serde_json::to_string(&value) {
        Ok(text) => println!("{text}"),
        Err(_) => println!(
            "{{\"command\":\"unknown\",\"ok\":false,\"exit\":2,\"findings\":[{{\"code\":\"OUTPUT_ERROR\"}}]}}"
        ),
    }
}

fn cli_error(command: &str, error: &RegistryError) -> u8 {
    print_cli(json!({
        "command": command,
        "ok": false,
        "exit": 2,
        "status": "ERROR",
        "findings": [{"code": error.code, "detail": error.detail}]
    }));
    2
}

pub fn run_cli<I>(arguments: I) -> u8
where
    I: IntoIterator<Item = OsString>,
{
    let arguments: Vec<OsString> = arguments.into_iter().collect();
    if arguments.is_empty() {
        return cli_error(
            "unknown",
            &RegistryError::new("COMMAND_REQUIRED", "scan or verify is required"),
        );
    }
    let command = match arguments[0].to_str() {
        Some(value) => value.to_string(),
        None => {
            return cli_error(
                "unknown",
                &RegistryError::new("NON_UTF8_ARGUMENT", "command must be UTF-8"),
            );
        }
    };
    let rest = &arguments[1..];
    let result = match command.as_str() {
        "scan" => parse_flags(rest, &["--config", "--output"]).and_then(|flags| {
            let config = flags
                .get("--config")
                .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--config is required"))?;
            let output = flags
                .get("--output")
                .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--output is required"))?;
            run_scan(config, output)
        }),
        "verify" => parse_flags(rest, &["--config", "--snapshot"]).and_then(|flags| {
            let config = flags
                .get("--config")
                .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--config is required"))?;
            let snapshot = flags
                .get("--snapshot")
                .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--snapshot is required"))?;
            run_verify(config, snapshot)
        }),
        "graph-build" => parse_flags(rest, &["--config", "--snapshot", "--output"])
            .and_then(|flags| graph::run_build(&flags)),
        "graph-verify" => parse_flags(rest, &["--config", "--snapshot", "--graph", "--scratch"])
            .and_then(|flags| graph::run_verify(&flags)),
        "graph-lookup" => parse_flags(
            rest,
            &[
                "--graph",
                "--assembly-sha",
                "--mvid",
                "--type",
                "--method",
                "--artifact-root-id",
                "--artifact-path",
            ],
        )
        .and_then(|flags| graph::run_lookup(&flags)),
        "source-status" => parse_flags(rest, &["--manifest", "--curated-root"])
            .and_then(|flags| source_closure::run_status(&flags)),
        "source-lookup" => parse_flags(
            rest,
            &[
                "--manifest",
                "--origin-id",
                "--function-id",
                "--artifact-root-id",
                "--artifact-path",
                "--assembly-sha",
                "--mvid",
                "--type",
                "--method",
                "--research-root",
                "--curated-root",
            ],
        )
        .and_then(|flags| source_closure::run_lookup(&flags)),
        "source-verify" => parse_flags(rest, &["--manifest", "--research-root", "--curated-root"])
            .and_then(|flags| source_closure::run_verify(&flags)),
        "source-inventory" => {
            parse_flags(rest, &["--manifest", "--research-root", "--curated-root"])
                .and_then(|flags| source_closure::run_inventory(&flags))
        }
        "source-access-index" => parse_flags(
            rest,
            &[
                "--manifest",
                "--output",
                "--curated-root",
                "--research-root",
            ],
        )
        .and_then(|flags| source_closure::run_access_index(&flags)),
        "source-locate" => parse_flags(
            rest,
            &[
                "--manifest",
                "--authority",
                "--mvid",
                "--type",
                "--method",
                "--research-root",
                "--curated-root",
                "--index",
            ],
        )
        .and_then(|flags| source_closure::run_locate(&flags)),
        _ => Err(RegistryError::new(
            "UNKNOWN_COMMAND",
            "scan, verify, graph-build, graph-verify, graph-lookup, source-status, source-lookup, source-verify, source-inventory, source-access-index, or source-locate is required",
        )),
    };
    match result {
        Ok(exit) => exit,
        Err(error) => cli_error(&command, &error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn observation(size: u64, mtime: u64, identity: Option<(u32, u32, u32)>) -> Observation {
        Observation {
            size,
            mtime,
            identity,
        }
    }

    #[test]
    fn hash_reader_matches_known_vectors() {
        let (empty, empty_len) = hash_reader(&mut Cursor::new(Vec::<u8>::new())).unwrap();
        assert_eq!(empty_len, 0);
        assert_eq!(
            empty,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let (hello, hello_len) = hash_reader(&mut Cursor::new(b"hello".to_vec())).unwrap();
        assert_eq!(hello_len, 5);
        assert_eq!(
            hello,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn stability_reason_detects_size_mtime_identity_and_post_close_changes() {
        let base = observation(5, 10, Some((1, 2, 3)));
        assert_eq!(stability_reason(&base, &base, 5, &base), None);
        assert_eq!(
            stability_reason(&base, &base, 4, &base),
            Some("SIZE_MISMATCH_READ")
        );
        let larger = observation(6, 10, Some((1, 2, 3)));
        assert_eq!(
            stability_reason(&base, &larger, 5, &base),
            Some("METADATA_CHANGED_DURING_READ")
        );
        let changed_time = observation(5, 11, Some((1, 2, 3)));
        assert_eq!(
            stability_reason(&base, &changed_time, 5, &base),
            Some("METADATA_CHANGED_DURING_READ")
        );
        let changed_identity = observation(5, 10, Some((1, 2, 4)));
        assert_eq!(
            stability_reason(&base, &changed_identity, 5, &base),
            Some("FILE_IDENTITY_CHANGED")
        );
        let post_changed = observation(5, 12, None);
        assert_eq!(
            stability_reason(&base, &base, 5, &post_changed),
            Some("METADATA_CHANGED_AFTER_CLOSE")
        );
    }
}
