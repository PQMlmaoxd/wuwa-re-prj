use super::*;
use crate::MethodIdentity;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

mod access;
mod multilang;

pub(super) fn run_access_index(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    access::run_access_index(flags)
}

pub(super) fn run_locate(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    access::run_locate(flags)
}

const MANIFEST_REL: &str = "metadata/source-closure.v1.json";
const MANIFEST_SCHEMA: &str = "wuwa-reconstruction-source-closure-manifest-v1";
const GRAPH_SCHEMA: &str = "wuwa-reconstruction-corpus-graph-v1";
const RESEARCH_SURVEY_ROOTS: [&str; 4] = [
    "src/reconstructed",
    "include/reconstructed",
    "_production",
    "_canary",
];
const CURATED_SURVEY_ROOTS: [&str; 3] = ["src/managed", "src/scripts", "src/models"];
const MULTILANG_CURATED_SURVEY_ROOTS: [&str; 2] = ["src/projections", "src/native"];
const MULTILANG_MANIFEST_REL: &str = "metadata/source-closure.multilang.v1.json";
const GRAPH_FILES: [&str; 8] = [
    "artifacts.links.jsonl",
    "cohorts.jsonl",
    "execution.json",
    "methods.jsonl",
    "missing.jsonl",
    "relationships.jsonl",
    "summary.json",
    "README.md",
];

// Production trust seam. The manifest pin is the primary-authorized accepted
// bytes. CLI --manifest only selects a file; it can never override this pin.
const PRODUCTION_MANIFEST_SHA256: Option<&str> =
    Some("42f70e404a13bfbe5545ba81b946630d60a5ec15f2218d004abedff6a95e1b1d");
// The successor pin is the primary-authorized accepted candidate bytes.
// Production source commands fail closed rather than silently running the
// predecessor-only view.
const PRODUCTION_MULTILANG_MANIFEST_SHA256: Option<&str> =
    Some("3e6a4ae50c0443b5ed20156cc0e7e4cb317bb13bcaf8328bb96e23927d6bbf8b");
const ACCEPTED_SNAPSHOT_ID: &str =
    "d805365e60cf1b1c156dceeb1b496779de082744e6d0c82dd126b1b5f452fcf6";
const ACCEPTED_GRAPH_ID: &str = "0f7cbec61d3b8e6a0fa767b06180c04156b0a65b72e02ff2c7a06ba531d4e643";
const REQUIRED_MANAGED_AUTHORITY: &str =
    "e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db";
const REQUIRED_MANAGED_MVID: &str = "2d6d37cb-76e5-4075-8220-3eeca41471e3";
const REQUIRED_MANAGED_TYPE_DEF: &str = "0x020048E7";
const REQUIRED_NEW_METHOD_IDS: [&str; 4] = [
    "d6fb119c90f7542603c9f65a56e4a7b8ea8b939a2213dd004b97b57736366b4b",
    "15317f6709492c324bf3b8a50f65eb5bc0c4ece88124f2ad50e184ead83f7465",
    "f04fa49831fe864ed87c2e555f50eee2cf3e724af1304683963ac69166137fe1",
    "40ae6f3a99e3ae22fe93416d189ba4e2193f22c15ff00442b469f2a4f1d06bd4",
];
const REQUIRED_IMPORT_IDS: [&str; 8] = [
    "IMP-01-DATALAYER-TS",
    "IMP-02-EVENT-JS",
    "IMP-03-EVENT-BRIDGE-JS",
    "IMP-04-CORECLR-NATIVE-THREAD",
    "IMP-05-CORECLR-THREAD-ORIGIN",
    "IMP-06-LIBXELL-HELPER-TARGET",
    "IMP-07-LIBXELL-SLEEP-IMPLEMENTATION",
    "IMP-08-RANGE-PENDING-DRAIN",
];
const REQUIRED_ORIGIN_IDS: [&str; 34] = [
    "SRC-TS-CONFIG-BOSSPILINGBUFF",
    "SRC-TS-CONFIG-BOSSRUSHBUFF",
    "SRC-TS-CONFIG-COMBOTEACHING",
    "SRC-TS-CONFIG-FLAGCHALLENGEROLEBUFF",
    "SRC-TS-CONFIG-FLOWTEXT",
    "SRC-TS-CONFIG-INTERACTDATA",
    "SRC-TS-CONFIG-KUROTATOGENERALGROWTH",
    "SRC-TS-CONFIG-PAYSHOPTAB",
    "SRC-TS-CONFIG-ROVERROGUEROOM",
    "SRC-TS-CONFIG-SKILLTREE",
    "SRC-TS-CONFIG-SORTRULE",
    "SRC-TS-CONFIG-TETRISMAP",
    "SRC-TS-LOADING-PIPELINE",
    "SRC-TS-STREAMING-COMPLETION-TRANSITION",
    "SRC-TS-DATALAYER-TRANSITION",
    "SRC-JS-EVENT-CORE",
    "SRC-JS-EVENT-CSHARP-BRIDGE",
    "SRC-JS-EVENT-CSHARP-BRIDGE-R15",
    "SRC-JS-CHARACTER-HIT-R27",
    "SRC-JS-ROLE-ELEMENT-R28",
    "SRC-JS-CHECK-PERF03B-SEMANTICS",
    "SRC-CPP-CORECLR-NATIVE-THREAD",
    "SRC-CPP-CORECLR-THREAD-ORIGIN",
    "SRC-CPP-LIBXELL-HELPER-TARGET",
    "SRC-CPP-LIBXELL-SLEEP-IMPLEMENTATION",
    "SRC-HPP-FEVENTSYSTEM",
    "SRC-HPP-KUROKVSTORE",
    "SRC-HPP-STREAMING-COMPLETION-BRIDGE",
    "SRC-HPP-WORLD-PARTITION-BRIDGE",
    "SRC-HPP-EVIDENCE-LABELS",
    "SRC-HPP-RESOURCE-INTERFACES",
    "SRC-HPP-STREAMING-INTERFACES",
    "SRC-HPP-TRANSITION-INTERFACES",
    "SRC-HPP-WORLD-PARTITION-INTERFACES",
];
const ACCEPTED_CLASS_COUNTS: [(&str, u64); 4] = [
    ("DIRECT_CIL_FULL", 246),
    ("SOURCE_ONLY", 280),
    ("UNKNOWN", 6),
    ("METADATA_ONLY", 1),
];

#[derive(Debug, Clone)]
struct AcceptedPayload {
    name: &'static str,
    bytes: u64,
    sha256: &'static str,
}

const ACCEPTED_GRAPH_PAYLOADS: [AcceptedPayload; 5] = [
    AcceptedPayload {
        name: "methods.jsonl",
        bytes: 764197857,
        sha256: "86441818f198af7bb220f3675098aaeb48f6f3ed6c47f2e1baa99b6aa20d4c03",
    },
    AcceptedPayload {
        name: "artifacts.links.jsonl",
        bytes: 54417149,
        sha256: "2f56bb62855a2bf4e6f5f8bad0c77cefdf315994b716ec6fd0c1dd1fd6fb33b4",
    },
    AcceptedPayload {
        name: "relationships.jsonl",
        bytes: 2795191,
        sha256: "9197b1bdd8d1068879107a1379f3fc28a672613021065e450407264e7e712b74",
    },
    AcceptedPayload {
        name: "cohorts.jsonl",
        bytes: 22589,
        sha256: "d98d7ade6fb6d270032effcce811245e1003de3b32361bcae312df1d21ca407e",
    },
    AcceptedPayload {
        name: "missing.jsonl",
        bytes: 3281,
        sha256: "376d817fe11d9f0dc8058f687b68c0bd8c587b56c36b93fca223b4cfacf1ff31",
    },
];

#[derive(Debug, Clone)]
struct Trust {
    manifest_sha256: Option<String>,
    multilang_manifest_sha256: Option<String>,
    successor_required: bool,
    snapshot_id: &'static str,
    graph_id: &'static str,
    payloads: &'static [AcceptedPayload],
}

fn trust_status() -> &'static str {
    if PRODUCTION_MANIFEST_SHA256.is_some() {
        "PRODUCTION_PIN_AUTHORIZED"
    } else {
        "PRIMARY_REVIEW_PENDING"
    }
}

fn production_trust() -> Trust {
    Trust {
        manifest_sha256: PRODUCTION_MANIFEST_SHA256.map(str::to_string),
        multilang_manifest_sha256: PRODUCTION_MULTILANG_MANIFEST_SHA256.map(str::to_string),
        successor_required: true,
        snapshot_id: ACCEPTED_SNAPSHOT_ID,
        graph_id: ACCEPTED_GRAPH_ID,
        payloads: &ACCEPTED_GRAPH_PAYLOADS,
    }
}

#[cfg(test)]
fn fixture_trust(manifest_sha256: String) -> Trust {
    Trust {
        manifest_sha256: Some(manifest_sha256),
        multilang_manifest_sha256: None,
        successor_required: false,
        snapshot_id: ACCEPTED_SNAPSHOT_ID,
        graph_id: ACCEPTED_GRAPH_ID,
        payloads: &ACCEPTED_GRAPH_PAYLOADS,
    }
}

#[cfg(test)]
fn fixture_trust_with_multilang(manifest_sha256: String, multilang_sha256: String) -> Trust {
    Trust {
        manifest_sha256: Some(manifest_sha256),
        multilang_manifest_sha256: Some(multilang_sha256),
        successor_required: true,
        snapshot_id: ACCEPTED_SNAPSHOT_ID,
        graph_id: ACCEPTED_GRAPH_ID,
        payloads: &ACCEPTED_GRAPH_PAYLOADS,
    }
}

fn multilang_trust_status() -> &'static str {
    if PRODUCTION_MULTILANG_MANIFEST_SHA256.is_some() {
        "SUCCESSOR_PIN_AUTHORIZED"
    } else {
        "SUCCESSOR_PRIMARY_AUDIT_PENDING"
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Disposition {
    Curated,
    ReferenceOnly,
    HistoricalAlternative,
    RecoveryRequired,
    Unresolved,
    DeclarationOnly,
    InternalOnly,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ArtifactRef {
    root_id: String,
    path: String,
    size: u64,
    sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NamedProof {
    name: String,
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SnapshotBaseline {
    root_id: String,
    path: String,
    snapshot_manifest: ArtifactRef,
    artifacts: ArtifactRef,
    snapshot_id: String,
    artifact_entries: u64,
    artifacts_bytes: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GraphBaseline {
    root_id: String,
    path: String,
    graph_id: String,
    counts: BTreeMap<String, u64>,
    payloads: Vec<NamedProof>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AcceptedComposition {
    old_c_file_count: u64,
    old_qualified_keys: u64,
    new_c_file_count: u64,
    supplemented_c_file_count: u64,
    supplemented_qualified_keys: u64,
    new_method_ids: Vec<String>,
    #[serde(default)]
    baseline_curated_paths: Vec<String>,
    #[serde(default)]
    accepted_class_counts: BTreeMap<String, u64>,
    #[serde(default)]
    supplemented_class_counts: BTreeMap<String, u64>,
    #[serde(default)]
    old_qualified_keys_sha256: String,
    #[serde(default)]
    graph_old_key_intersection: Option<u64>,
    #[serde(default)]
    graph_outside_primary_generated: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Baseline {
    snapshot: SnapshotBaseline,
    graph: GraphBaseline,
    accepted_composition: AcceptedComposition,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Authority {
    kind: String,
    authority_id: Option<String>,
    binary_sha256: Option<String>,
    mvid: Option<String>,
    type_def: Option<String>,
    rva: Option<String>,
    export: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProofAnchor {
    root_id: String,
    path: String,
    size: u64,
    sha256: String,
    locator: Option<String>,
    range_start: Option<u64>,
    range_end: Option<u64>,
    range_sha256: Option<String>,
    contains: Vec<String>,
    record_id: Option<String>,
    record_disposition: Option<String>,
    evidence_class: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ArtifactDisposition {
    origin_id: String,
    layer: String,
    origin_class: String,
    artifact: ArtifactRef,
    kind: String,
    disposition: Disposition,
    representation_type: String,
    primary_class: String,
    scope: String,
    implementation_scope: String,
    evidence_status: String,
    authority: Authority,
    dependencies: Vec<String>,
    uncertainties: Vec<String>,
    proof_anchors: Vec<ProofAnchor>,
    accepted_evidence: Vec<String>,
    alternative_ids: Vec<String>,
    public_distribution: String,
    identity_scheme: String,
    overlap: String,
    import_id: Option<String>,
    managed_methoddefs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LineRange {
    start_line: u64,
    end_line: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ChunkProof {
    input_start_line: u64,
    input_end_line: u64,
    output_start_line: u64,
    output_end_line: u64,
    sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Transform {
    input: ArtifactRef,
    banner: String,
    ranges: Vec<LineRange>,
    omitted_ranges: Vec<LineRange>,
    chunks: Vec<ChunkProof>,
    added_banner_sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MemberProof {
    capture_id: String,
    capture_path: String,
    capture_file_size: u64,
    capture_file_sha256: String,
    capture_start_line: u64,
    capture_end_line: u64,
    capture_section_sha256: String,
    capture_methoddef: String,
    capture_rva: String,
    capture_code_size: u64,
    evidence_path: String,
    evidence_file_size: u64,
    evidence_file_sha256: String,
    evidence_selector: String,
    evidence_methoddef: String,
    authority_sha256: String,
    mvid: String,
    type_def: String,
    required_il_tokens: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MemberSelector {
    method_id: String,
    role: String,
    type_token: String,
    method_token: String,
    representation: String,
    source_input_lines: Option<LineRange>,
    output_lines: Option<LineRange>,
    input_range_sha256: String,
    output_range_sha256: String,
    scope: String,
    proof: MemberProof,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FunctionSpan {
    start_line: u64,
    end_line: u64,
    sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NativeLocator {
    binary_sha256: String,
    rva: String,
    export: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FunctionView {
    function_id: String,
    origin_id: String,
    logical_name: String,
    kind: String,
    status: String,
    source: ArtifactRef,
    source_span: FunctionSpan,
    native_locator: Option<NativeLocator>,
    header_token: String,
    notes: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceImport {
    id: String,
    origin_id: Option<String>,
    layer: String,
    mode: String,
    source: ArtifactRef,
    destination: ArtifactRef,
    transform: Option<Transform>,
    identity_scheme: String,
    managed_methoddefs: Vec<String>,
    #[serde(default)]
    managed_authority_sha256: Option<String>,
    #[serde(default)]
    managed_mvid: Option<String>,
    #[serde(default)]
    managed_type_def: Option<String>,
    member_selectors: Vec<MemberSelector>,
    #[serde(default)]
    function_views: Vec<FunctionView>,
    dependencies: Vec<String>,
    scope: String,
    evidence_status: String,
    accepted_evidence: Vec<String>,
    proof_anchors: Vec<ProofAnchor>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BacklogRow {
    id: String,
    category: String,
    reason: String,
    source: Option<ArtifactRef>,
    evidence_status: String,
    disposition_required: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FamilyRule {
    family_id: String,
    origin_class: String,
    disposition_class: String,
    match_kind: String,
    path_rule: String,
    extensions: Vec<String>,
    snapshot_kind: Option<String>,
    snapshot_count: u64,
    evidence_scope: String,
    notes: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceManifest {
    schema: String,
    manifest_version: u64,
    campaign: String,
    status: String,
    not_final: bool,
    publication: String,
    baseline: Baseline,
    non_managed_artifacts: Vec<ArtifactDisposition>,
    imports: Vec<SourceImport>,
    backlog: Vec<BacklogRow>,
    family_rules: Vec<FamilyRule>,
    limits: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SnapshotRow {
    artifact_id: String,
    root_id: String,
    path: String,
    size: Option<u64>,
    sha256: Option<String>,
    status: String,
    kind: String,
    backup_coverage: String,
    error_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GraphPayloadFile {
    name: String,
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GraphSummaryLite {
    schema: String,
    graph_id: String,
    payload_files: Vec<GraphPayloadFile>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GraphArtifactRef {
    root_id: String,
    path: String,
    size: u64,
    sha256: String,
    kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GraphRepresentation {
    representation_id: String,
    representation_kind: String,
    evidence: Vec<String>,
    coverage: Option<String>,
    source_status: Option<String>,
    source_ref: Option<GraphArtifactRef>,
    raw_ref: Option<GraphArtifactRef>,
    semantic_ref: Option<GraphArtifactRef>,
    semantic_range: Option<LineRange>,
    proof_ref: Option<GraphArtifactRef>,
    proof_row: Option<u64>,
    raw_row: Option<u64>,
    source_range: Option<LineRange>,
    body_sha256: Option<String>,
    body_hash_verified: Option<bool>,
    table_heading: Option<String>,
    table_header: Option<String>,
    note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GraphMethodRow {
    method_id: String,
    assembly_sha256: String,
    mvid: String,
    type_token: String,
    method_token: String,
    root_owners: Vec<String>,
    representations: Vec<GraphRepresentation>,
    evidence_levels: BTreeMap<String, u64>,
    source_refs: Vec<GraphArtifactRef>,
    raw_refs: Vec<GraphArtifactRef>,
    cohorts: Vec<String>,
    limitations: Vec<String>,
}

struct Roots {
    research: PathBuf,
    curated: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CurrentState {
    Missing,
    Present(u64),
}

fn reg(code: &'static str, detail: impl Into<String>) -> RegistryError {
    RegistryError::new(code, detail)
}

fn flags_string(flags: &BTreeMap<String, PathBuf>, key: &str) -> Option<String> {
    flags
        .get(key)
        .map(|value| value.to_string_lossy().to_string())
}

fn absolute_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .map(|current| current.join(&path))
            .unwrap_or(path)
    }
}

fn curated_root_from_flags(flags: &BTreeMap<String, PathBuf>) -> PathBuf {
    absolute_path(
        flags
            .get("--curated-root")
            .cloned()
            .unwrap_or_else(|| PathBuf::from(".")),
    )
}

fn manifest_path(flags: &BTreeMap<String, PathBuf>) -> PathBuf {
    if let Some(path) = flags.get("--manifest") {
        return absolute_path(path.clone());
    }
    curated_root_from_flags(flags).join(MANIFEST_REL)
}

fn roots_from_flags(
    flags: &BTreeMap<String, PathBuf>,
    require_research: bool,
) -> RegistryResult<Roots> {
    let curated = curated_root_from_flags(flags);
    let research = flags.get("--research-root").cloned().map(absolute_path);
    inspect_existing_directory(&curated).map_err(|_| {
        reg(
            "CURATED_ROOT_INVALID",
            "curated root must be an existing non-reparse directory",
        )
    })?;
    if require_research && research.is_none() {
        return Err(reg(
            "RESEARCH_ROOT_REQUIRED",
            "--research-root is required for this command",
        ));
    }
    if let Some(root) = &research {
        inspect_existing_directory(root).map_err(|_| {
            reg(
                "RESEARCH_ROOT_INVALID",
                "research root must be an existing non-reparse directory",
            )
        })?;
    }
    Ok(Roots {
        research: research.unwrap_or_else(|| PathBuf::from(".")),
        curated,
    })
}

fn load_manifest(path: &Path) -> RegistryResult<(SourceManifest, Vec<u8>)> {
    inspect_existing_file(path)?;
    let bytes = fs::read(path).map_err(|_| {
        reg(
            "MANIFEST_MISSING",
            "source-closure manifest could not be read",
        )
    })?;
    let value = crate::parse_strict_value(&bytes)
        .map_err(|_| reg("MANIFEST_INVALID_JSON", "manifest is not strict JSON"))?;
    let manifest: SourceManifest = serde_json::from_value(value)
        .map_err(|_| reg("MANIFEST_INVALID_SHAPE", "manifest shape is invalid"))?;
    validate_manifest_structure(&manifest)?;
    Ok((manifest, bytes))
}

fn load_manifest_trusted(path: &Path, trust: &Trust) -> RegistryResult<(SourceManifest, Vec<u8>)> {
    let (manifest, bytes) = load_manifest(path)?;
    let expected = trust.manifest_sha256.as_deref().ok_or_else(|| {
        reg(
            "TRUST_PIN_UNAVAILABLE",
            "production manifest pin is not authorized; candidate CLI commands fail closed",
        )
    })?;
    if crate::sha256_hex(&bytes) != expected {
        return Err(reg(
            "TRUST_MANIFEST_HASH_MISMATCH",
            "manifest bytes do not match the explicit trust pin",
        ));
    }
    if manifest.baseline.snapshot.snapshot_id != trust.snapshot_id
        || manifest.baseline.graph.graph_id != trust.graph_id
    {
        return Err(reg(
            "TRUST_BASELINE_HASH_MISMATCH",
            "snapshot or graph identity does not match the accepted trust constants",
        ));
    }
    if manifest.baseline.graph.payloads.len() != trust.payloads.len() {
        return Err(reg(
            "TRUST_GRAPH_PAYLOAD_SET_MISMATCH",
            "graph payload set is not the exact accepted five-name set",
        ));
    }
    for accepted in trust.payloads {
        let found = manifest
            .baseline
            .graph
            .payloads
            .iter()
            .filter(|payload| payload.name == accepted.name)
            .collect::<Vec<_>>();
        if found.len() != 1
            || found[0].bytes != accepted.bytes
            || found[0].sha256 != accepted.sha256
        {
            return Err(reg(
                "TRUST_GRAPH_PAYLOAD_MISMATCH",
                format!("graph payload is not accepted: {}", accepted.name),
            ));
        }
    }
    Ok((manifest, bytes))
}

fn valid_hex64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn ext_of(path: &str) -> &str {
    path.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("")
}

fn route_check(path: &str, disposition: &Disposition) -> RegistryResult<()> {
    let ext = ext_of(path);
    if ext == "hpp" && *disposition != Disposition::DeclarationOnly {
        return Err(reg(
            "DISPOSITION_ROUTE_INVALID",
            format!("header row must remain DECLARATION_ONLY: {path}"),
        ));
    }
    if path.starts_with("src/reconstructed/config/")
        && ext == "ts"
        && *disposition != Disposition::DeclarationOnly
    {
        return Err(reg(
            "DISPOSITION_ROUTE_INVALID",
            format!("config schema row must remain DECLARATION_ONLY: {path}"),
        ));
    }
    if (path.ends_with("src/reconstructed/loading/LoadingPipeline.ts")
        || path.ends_with("src/reconstructed/transition/StreamingCompletionTransition.ts"))
        && *disposition != Disposition::ReferenceOnly
    {
        return Err(reg(
            "DISPOSITION_ROUTE_INVALID",
            format!("reference model row must remain REFERENCE_ONLY: {path}"),
        ));
    }
    if path == "evidence/performance/check_perf03b_semantics.js"
        && *disposition != Disposition::InternalOnly
    {
        return Err(reg(
            "DISPOSITION_ROUTE_INVALID",
            "internal checker row must remain INTERNAL_ONLY",
        ));
    }
    if path.starts_with("src/reconstructed/performance/native/")
        && ext == "cpp"
        && *disposition != Disposition::Curated
    {
        return Err(reg(
            "DISPOSITION_ROUTE_INVALID",
            format!("native source-like model must remain CURATED: {path}"),
        ));
    }
    Ok(())
}

fn validate_ranges(ranges: &[LineRange], label: &str) -> RegistryResult<()> {
    if ranges.is_empty() {
        return Err(reg("TRANSFORM_RANGE_INVALID", format!("{label} empty")));
    }
    let mut previous = 0_u64;
    for range in ranges {
        if range.start_line == 0 || range.end_line < range.start_line {
            return Err(reg("TRANSFORM_RANGE_INVALID", format!("{label} bounds")));
        }
        if range.start_line <= previous {
            return Err(reg(
                "TRANSFORM_RANGE_INVALID",
                format!("{label} overlap or ordering"),
            ));
        }
        previous = range.end_line;
    }
    Ok(())
}

fn validate_transform(transform: &Transform) -> RegistryResult<()> {
    validate_ranges(&transform.ranges, "selected ranges")?;
    validate_ranges(&transform.omitted_ranges, "omitted ranges")?;
    for selected in &transform.ranges {
        for omitted in &transform.omitted_ranges {
            if selected.start_line <= omitted.end_line && omitted.start_line <= selected.end_line {
                return Err(reg(
                    "TRANSFORM_PARTIAL_OVERLAP",
                    "selected and omitted ranges overlap",
                ));
            }
        }
    }
    if transform.chunks.len() != transform.ranges.len() {
        return Err(reg(
            "TRANSFORM_CHUNK_INVALID",
            "one chunk proof is required per selected range",
        ));
    }
    if !valid_hex64(&transform.added_banner_sha256) {
        return Err(reg("TRANSFORM_CHUNK_INVALID", "banner hash is invalid"));
    }
    let banner_hash = crate::sha256_hex(transform.banner.as_bytes());
    if banner_hash != transform.added_banner_sha256 {
        return Err(reg(
            "TRANSFORM_CHUNK_INVALID",
            "banner hash does not match banner bytes",
        ));
    }
    let mut expected_output = transform.banner.matches('\n').count() as u64 + 1;
    for (range, chunk) in transform.ranges.iter().zip(&transform.chunks) {
        if chunk.input_start_line != range.start_line
            || chunk.input_end_line != range.end_line
            || chunk.output_start_line != expected_output
            || chunk.output_end_line != expected_output + (range.end_line - range.start_line)
            || !valid_hex64(&chunk.sha256)
        {
            return Err(reg(
                "TRANSFORM_CHUNK_INVALID",
                "chunk proof does not describe the selected range",
            ));
        }
        expected_output = chunk.output_end_line + 1;
    }
    Ok(())
}

fn validate_member_identity(
    authority: &str,
    mvid: &str,
    type_def: &str,
    member: &MemberSelector,
) -> RegistryResult<()> {
    if !crate::validate_method_token(&member.method_token)
        || !(member.type_token.len() == 10
            && member.type_token.starts_with("0x02")
            && member.type_token[2..]
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit()))
    {
        return Err(reg("MANIFEST_METHOD_TOKEN_INVALID", "method token"));
    }
    if !member.type_token.eq_ignore_ascii_case(type_def) {
        return Err(reg(
            "MANIFEST_WRONG_TYPEDEF",
            "member type token does not match authority TypeDef",
        ));
    }
    let expected = method_id(authority, mvid, &member.type_token, &member.method_token);
    if expected != member.method_id {
        return Err(reg(
            "MANIFEST_UNRELATED_METHODDEF",
            "member identity does not match the full method key",
        ));
    }
    if authority != REQUIRED_MANAGED_AUTHORITY
        || !mvid.eq_ignore_ascii_case(REQUIRED_MANAGED_MVID)
        || !type_def.eq_ignore_ascii_case(REQUIRED_MANAGED_TYPE_DEF)
        || !member
            .proof
            .authority_sha256
            .eq_ignore_ascii_case(REQUIRED_MANAGED_AUTHORITY)
        || !member
            .proof
            .mvid
            .eq_ignore_ascii_case(REQUIRED_MANAGED_MVID)
        || !member
            .proof
            .type_def
            .eq_ignore_ascii_case(REQUIRED_MANAGED_TYPE_DEF)
    {
        return Err(reg(
            "MANIFEST_CS_AUTHORITY_INVALID",
            "managed authority, MVID, or TypeDef does not match the accepted identity",
        ));
    }
    if member.source_input_lines.is_none() || member.output_lines.is_none() {
        return Err(reg(
            "MANIFEST_MEMBER_RANGE_REQUIRED",
            "member must bind both source input and output ranges",
        ));
    }
    if !valid_hex64(&member.input_range_sha256)
        || !valid_hex64(&member.output_range_sha256)
        || !valid_hex64(&member.proof.capture_file_sha256)
        || !valid_hex64(&member.proof.evidence_file_sha256)
        || !valid_hex64(&member.proof.capture_section_sha256)
    {
        return Err(reg(
            "MANIFEST_MEMBER_RANGE_HASH_INVALID",
            "member range or capture proof hash is not lower-hex",
        ));
    }
    if member.proof.capture_methoddef != member.method_token
        || member.proof.evidence_methoddef != member.method_token
        || member.proof.capture_start_line == 0
        || member.proof.capture_end_line < member.proof.capture_start_line
        || member.proof.capture_code_size == 0
        || !member.proof.capture_rva.starts_with("0x")
        || member.proof.required_il_tokens.is_empty()
        || member.proof.evidence_selector.is_empty()
    {
        return Err(reg(
            "MANIFEST_MEMBER_PROOF_INVALID",
            "member capture/evidence proof is incomplete or mismatched",
        ));
    }
    let expected_capture = match member.method_token.as_str() {
        "0x06030B3D" => ("PERF11_INIT_EMIT_EVENT_DATA", "0xC02C04", 178_u64),
        "0x06030B84" => ("PERF11_EMIT_MY_PLAYER_OVERLAP_EVENT", "0xC06D34", 445_u64),
        "0x06030B87" => ("PERF11_EMIT_PENDING_EVENTS", "0xC07344", 574_u64),
        "0x06030BA8" => ("PERF11_EMIT_MY_PLAYER_OVERLAP_CALLBACK", "0xC091B3", 7_u64),
        _ => {
            return Err(reg(
                "MANIFEST_MEMBER_PROOF_INVALID",
                "member MethodDef is outside the finite four-key set",
            ));
        }
    };
    if member.proof.capture_id != expected_capture.0
        || !member
            .proof
            .capture_rva
            .eq_ignore_ascii_case(expected_capture.1)
        || member.proof.capture_code_size != expected_capture.2
    {
        return Err(reg(
            "MANIFEST_MEMBER_PROOF_INVALID",
            "member capture recipe does not match the canonical PERF11 section",
        ));
    }
    if member.role == "ASSOCIATED" {
        if member.method_token != "0x06030BA8"
            || member.proof.capture_code_size != 7
            || !member
                .proof
                .required_il_tokens
                .iter()
                .any(|token| token == "IL_0000: ldarg.0")
            || !member
                .proof
                .required_il_tokens
                .iter()
                .any(|token| token == "06030B87")
            || !member
                .proof
                .required_il_tokens
                .iter()
                .any(|token| token == "IL_0006: ret")
        {
            return Err(reg(
                "MANIFEST_CALLBACK_PROOF_INVALID",
                "associated callback must bind the complete 7-byte callback IL",
            ));
        }
    }
    Ok(())
}

fn validate_function_view(
    manifest: &SourceManifest,
    import: &SourceImport,
    view: &FunctionView,
) -> RegistryResult<()> {
    if !view.function_id.starts_with("FUNC-") || view.function_id.len() != 69 {
        return Err(reg(
            "MANIFEST_FUNCTION_ID_INVALID",
            "function_id must be FUNC- plus 64 lower-hex characters",
        ));
    }
    if !valid_hex64(&view.function_id[5..]) {
        return Err(reg("MANIFEST_FUNCTION_ID_INVALID", "function_id hash"));
    }
    if view.origin_id != import.origin_id.clone().unwrap_or_default() {
        return Err(reg(
            "MANIFEST_FUNCTION_ORIGIN_INVALID",
            "function view origin is not the import origin",
        ));
    }
    if view.source.root_id != "RESEARCH_ROOT"
        || view.source.path != import.source.path
        || view.source.size != import.source.size
        || view.source.sha256 != import.source.sha256
    {
        return Err(reg(
            "MANIFEST_FUNCTION_SOURCE_INVALID",
            "function source does not match the imported origin bytes",
        ));
    }
    if view.source_span.start_line == 0
        || view.source_span.end_line < view.source_span.start_line
        || !valid_hex64(&view.source_span.sha256)
    {
        return Err(reg(
            "MANIFEST_FUNCTION_SPAN_INVALID",
            "function source span is not a valid range/hash",
        ));
    }
    if view.header_token.is_empty() || view.header_token.contains('\n') {
        return Err(reg(
            "MANIFEST_FUNCTION_HEADER_INVALID",
            "function header token is empty or multiline",
        ));
    }
    let kinds = [
        "ORDINARY",
        "CONSTRUCTOR",
        "CALLBACK",
        "GETTER",
        "OPAQUE_NATIVE_MODEL",
        "PARTIAL",
    ];
    let statuses = [
        "BODY",
        "PARTIAL",
        "ASSOCIATED",
        "ASSOCIATED_CALLBACK",
        "SOURCE_LIKE_BODY",
    ];
    if !kinds.contains(&view.kind.as_str()) || !statuses.contains(&view.status.as_str()) {
        return Err(reg(
            "MANIFEST_FUNCTION_KIND_INVALID",
            "function kind or status is outside the finite vocabulary",
        ));
    }
    if view.kind == "CALLBACK"
        && view.status != "ASSOCIATED"
        && view.status != "ASSOCIATED_CALLBACK"
        && view.status != "BODY"
    {
        return Err(reg(
            "MANIFEST_FUNCTION_STATUS_INVALID",
            "callback status must remain associated",
        ));
    }
    let origin = manifest
        .non_managed_artifacts
        .iter()
        .find(|row| row.origin_id == view.origin_id)
        .ok_or_else(|| reg("MANIFEST_FUNCTION_ORIGIN_INVALID", "origin row missing"))?;
    if origin.layer == "CPP_HEADER" {
        return Err(reg(
            "MANIFEST_FUNCTION_ORIGIN_INVALID",
            "declaration-only headers cannot carry function views",
        ));
    }
    if import.layer == "CPP" {
        let locator = view.native_locator.as_ref().ok_or_else(|| {
            reg(
                "MANIFEST_NATIVE_LOCATOR_REQUIRED",
                "native function view needs an observed RVA locator",
            )
        })?;
        let authority_sha = origin.authority.binary_sha256.as_deref().ok_or_else(|| {
            reg(
                "MANIFEST_NATIVE_LOCATOR_REQUIRED",
                "origin binary SHA missing",
            )
        })?;
        if locator.binary_sha256 != authority_sha {
            return Err(reg(
                "MANIFEST_NATIVE_LOCATOR_INVALID",
                "native locator binary SHA does not match the origin authority",
            ));
        }
        if let Some(authority_rva) = origin.authority.rva.as_deref() {
            if !authority_rva
                .split('/')
                .any(|part| part.eq_ignore_ascii_case(&locator.rva))
            {
                return Err(reg(
                    "MANIFEST_NATIVE_LOCATOR_INVALID",
                    "native locator RVA is not part of the origin authority",
                ));
            }
        } else {
            return Err(reg(
                "MANIFEST_NATIVE_LOCATOR_INVALID",
                "origin authority has no RVA binding",
            ));
        }
        if let Some(locator_export) = locator.export.as_deref() {
            let authority_export = origin.authority.export.as_deref().unwrap_or("");
            if !authority_export
                .split('/')
                .any(|part| part == locator_export)
            {
                return Err(reg(
                    "MANIFEST_NATIVE_LOCATOR_INVALID",
                    "native locator export is not part of the origin authority",
                ));
            }
        }
    } else if view.native_locator.is_some() {
        return Err(reg(
            "MANIFEST_NATIVE_LOCATOR_INVALID",
            "non-native function view cannot carry a native locator",
        ));
    }
    let locator_key = if let Some(native) = &view.native_locator {
        format!("NATIVE_RVA:{}:{}", native.binary_sha256, native.rva)
    } else {
        format!(
            "SOURCE_SPAN:{}-{}",
            view.source_span.start_line, view.source_span.end_line
        )
    };
    let expected = format!(
        "FUNC-{}",
        crate::sha256_hex(
            format!(
                "{}\0{}\0{}\0{}",
                view.origin_id, view.logical_name, locator_key, view.source.sha256
            )
            .as_bytes()
        )
    );
    if expected != view.function_id {
        return Err(reg(
            "MANIFEST_FUNCTION_ID_INVALID",
            "function_id does not derive from origin, name, locator, and whole-file hash",
        ));
    }
    Ok(())
}

fn validate_manifest_structure(manifest: &SourceManifest) -> RegistryResult<()> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(reg("MANIFEST_SCHEMA_INVALID", "manifest schema"));
    }
    if manifest.manifest_version != 2 {
        return Err(reg("MANIFEST_SCHEMA_INVALID", "manifest version"));
    }
    if manifest.publication != "DISABLED_INTERNAL_ONLY" {
        return Err(reg(
            "MANIFEST_PUBLICATION_INVALID",
            "supplementary manifest must remain internal-only",
        ));
    }
    for artifact in [
        &manifest.baseline.snapshot.snapshot_manifest,
        &manifest.baseline.snapshot.artifacts,
    ] {
        crate::validate_relative_path(&artifact.path)
            .map_err(|_| reg("MANIFEST_PATH_INVALID", "baseline artifact path"))?;
    }
    crate::validate_relative_path(&manifest.baseline.snapshot.path)
        .map_err(|_| reg("MANIFEST_PATH_INVALID", "snapshot path"))?;
    crate::validate_relative_path(&manifest.baseline.graph.path)
        .map_err(|_| reg("MANIFEST_PATH_INVALID", "graph path"))?;
    if !valid_hex64(&manifest.baseline.snapshot.snapshot_id) {
        return Err(reg("MANIFEST_IDENTITY_INVALID", "snapshot id"));
    }
    if manifest.baseline.snapshot.artifact_entries != 41456
        || manifest.baseline.snapshot.artifacts_bytes != 18720457
        || manifest.baseline.snapshot.artifacts.size != 18720457
        || manifest.baseline.snapshot.artifacts.sha256 != manifest.baseline.snapshot.snapshot_id
    {
        return Err(reg(
            "MANIFEST_BASELINE_INVALID",
            "snapshot baseline identity is not the accepted value",
        ));
    }
    if manifest.baseline.graph.graph_id.len() != 64 || manifest.baseline.graph.payloads.len() != 5 {
        return Err(reg("MANIFEST_BASELINE_INVALID", "graph baseline"));
    }
    let composition = &manifest.baseline.accepted_composition;
    if composition.old_c_file_count != 104
        || composition.old_qualified_keys != 533
        || composition.new_c_file_count != 1
        || composition.supplemented_c_file_count != 105
        || composition.supplemented_qualified_keys != 537
        || composition.new_method_ids.len() != 4
        || composition
            .new_method_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != REQUIRED_NEW_METHOD_IDS
                .iter()
                .map(|value| value.to_string())
                .collect::<BTreeSet<_>>()
        || composition.baseline_curated_paths.len() != 104
        || composition
            .baseline_curated_paths
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .len()
            != 104
    {
        return Err(reg(
            "MANIFEST_BASELINE_INVALID",
            "supplemented composition must remain 104/533 + 1/4 = 105/537",
        ));
    }
    for path in &composition.baseline_curated_paths {
        if crate::validate_relative_path(path).is_err()
            || !path.starts_with("src/managed/cs-e3fc67839851/")
        {
            return Err(reg(
                "MANIFEST_BASELINE_INVALID",
                "baseline curated path is unsafe or outside the accepted import root",
            ));
        }
    }
    let expected_accepted = ACCEPTED_CLASS_COUNTS
        .iter()
        .map(|(name, count)| (name.to_string(), *count))
        .collect::<BTreeMap<_, _>>();
    if composition.accepted_class_counts != expected_accepted {
        return Err(reg(
            "MANIFEST_ACCEPTED_CLASS_COUNTS_INVALID",
            "accepted 533 evidence-class counts are not the preserved 246/280/6/1",
        ));
    }
    let expected_supplemented = BTreeMap::from([
        ("DIRECT_CIL_FULL".to_string(), 250_u64),
        ("SOURCE_ONLY".to_string(), 280),
        ("UNKNOWN".to_string(), 6),
        ("METADATA_ONLY".to_string(), 1),
    ]);
    if composition.supplemented_class_counts != expected_supplemented {
        return Err(reg(
            "MANIFEST_SUPPLEMENTED_CLASS_COUNTS_INVALID",
            "supplemented 537 evidence-class counts are not 250/280/6/1",
        ));
    }
    let mut origin_ids = BTreeSet::new();
    let mut origin_paths = BTreeSet::new();
    for row in &manifest.non_managed_artifacts {
        crate::validate_relative_path(&row.artifact.path)
            .map_err(|_| reg("MANIFEST_PATH_INVALID", "non-managed artifact path"))?;
        if row.artifact.root_id != "RESEARCH_ROOT" {
            return Err(reg(
                "MANIFEST_ROOT_INVALID",
                "non-managed artifact must resolve under RESEARCH_ROOT",
            ));
        }
        if row.proof_anchors.is_empty() && !matches!(row.disposition, Disposition::DeclarationOnly)
        {
            return Err(reg(
                "MANIFEST_PROOF_REQUIRED",
                "mandatory non-managed proof anchors must not be empty",
            ));
        }
        if !origin_ids.insert(row.origin_id.clone()) {
            return Err(reg("MANIFEST_DUPLICATE_ID", "duplicate origin_id"));
        }
        if !origin_paths.insert((row.artifact.root_id.clone(), row.artifact.path.clone())) {
            return Err(reg("MANIFEST_CONFLICT_PATH", "duplicate artifact path"));
        }
        if !valid_hex64(&row.artifact.sha256) {
            return Err(reg("MANIFEST_HASH_INVALID", "artifact hash"));
        }
        route_check(&row.artifact.path, &row.disposition)?;
        if !row.managed_methoddefs.is_empty() {
            return Err(reg(
                "MANIFEST_NATIVE_METHODDEF_INVALID",
                "non-managed artifacts must not claim managed MethodDefs",
            ));
        }
        for anchor in &row.proof_anchors {
            if !valid_hex64(&anchor.sha256) || anchor.evidence_class.is_empty() {
                return Err(reg("MANIFEST_PROOF_INVALID", "origin proof hash/class"));
            }
            if anchor.range_start.is_some()
                || anchor.range_end.is_some()
                || anchor.range_sha256.is_some()
            {
                if anchor.range_start.is_none()
                    || anchor.range_end.is_none()
                    || !anchor
                        .range_sha256
                        .as_deref()
                        .map(valid_hex64)
                        .unwrap_or(false)
                    || anchor.contains.is_empty()
                {
                    return Err(reg(
                        "MANIFEST_PROOF_RANGE_INVALID",
                        "origin bounded proof requires range hash and contains tokens",
                    ));
                }
            }
        }
        if row.public_distribution != "INTERNAL_ONLY_NOT_REVIEWED" {
            return Err(reg(
                "MANIFEST_PUBLICATION_INVALID",
                "non-managed artifact distribution",
            ));
        }
    }
    if origin_ids
        != REQUIRED_ORIGIN_IDS
            .iter()
            .map(|value| value.to_string())
            .collect::<BTreeSet<_>>()
    {
        return Err(reg(
            "MANIFEST_ORIGIN_SET_MISMATCH",
            "34 non-managed origins are not the exact accepted set",
        ));
    }
    let mut import_ids = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    let mut function_ids = BTreeSet::new();
    let mut deterministic = 0_u64;
    for import in &manifest.imports {
        if !import_ids.insert(import.id.clone()) {
            return Err(reg("MANIFEST_DUPLICATE_ID", "duplicate import id"));
        }
        crate::validate_relative_path(&import.source.path)
            .map_err(|_| reg("MANIFEST_PATH_INVALID", "import source path"))?;
        crate::validate_relative_path(&import.destination.path)
            .map_err(|_| reg("MANIFEST_PATH_INVALID", "import destination path"))?;
        if import.source.root_id != "RESEARCH_ROOT" || import.destination.root_id != "CURATED_ROOT"
        {
            return Err(reg(
                "MANIFEST_ROOT_INVALID",
                "imports must resolve RESEARCH_ROOT -> CURATED_ROOT",
            ));
        }
        if import.proof_anchors.is_empty() {
            return Err(reg(
                "MANIFEST_PROOF_REQUIRED",
                "import proof anchors must not be empty",
            ));
        }
        if !destinations.insert(import.destination.path.clone()) {
            return Err(reg(
                "MANIFEST_CONFLICT_PATH",
                "duplicate import destination",
            ));
        }
        if import.source.path == import.destination.path {
            return Err(reg(
                "MANIFEST_CONFLICT_PATH",
                "import source equals destination",
            ));
        }
        if !valid_hex64(&import.source.sha256) || !valid_hex64(&import.destination.sha256) {
            return Err(reg("MANIFEST_HASH_INVALID", "import hash"));
        }
        if import.mode == "EXACT_BYTE_COPY" {
            if import.layer == "CS"
                || !import.managed_methoddefs.is_empty()
                || !import.member_selectors.is_empty()
                || import.transform.is_some()
            {
                return Err(reg(
                    "MANIFEST_NATIVE_METHODDEF_INVALID",
                    "exact non-CS copy cannot claim managed identity",
                ));
            }
            if import.source.size != import.destination.size
                || import.source.sha256 != import.destination.sha256
            {
                return Err(reg(
                    "MANIFEST_BYTE_COPY_DRIFT",
                    "exact copy must preserve source bytes",
                ));
            }
            if import.function_views.is_empty() {
                return Err(reg(
                    "MANIFEST_FUNCTION_SET_REQUIRED",
                    "non-CS import must declare its finite saved-function selectors",
                ));
            }
        } else if import.mode == "DETERMINISTIC_LINE_SELECTION" {
            deterministic += 1;
            if import.layer != "CS" || import.managed_methoddefs.is_empty() {
                return Err(reg(
                    "MANIFEST_CS_IDENTITY_REQUIRED",
                    "deterministic C# view must claim managed identities",
                ));
            }
            let transform = import.transform.as_ref().ok_or_else(|| {
                reg(
                    "MANIFEST_TRANSFORM_REQUIRED",
                    "deterministic view needs transform",
                )
            })?;
            validate_transform(transform)?;
            if transform.input.path != import.source.path
                || transform.input.size != import.source.size
                || transform.input.sha256 != import.source.sha256
            {
                return Err(reg(
                    "MANIFEST_TRANSFORM_INPUT_INVALID",
                    "transform input must match source artifact",
                ));
            }
            let mut method_ids = BTreeSet::new();
            let mut ordinary = 0_u64;
            let mut associated = 0_u64;
            for member in &import.member_selectors {
                let authority = import.managed_authority_sha256.as_deref().ok_or_else(|| {
                    reg("MANIFEST_CS_IDENTITY_REQUIRED", "managed authority missing")
                })?;
                let mvid = import
                    .managed_mvid
                    .as_deref()
                    .ok_or_else(|| reg("MANIFEST_CS_IDENTITY_REQUIRED", "managed MVID missing"))?;
                let type_def = import.managed_type_def.as_deref().ok_or_else(|| {
                    reg("MANIFEST_CS_IDENTITY_REQUIRED", "managed TypeDef missing")
                })?;
                validate_member_identity(authority, mvid, type_def, member)?;
                if !method_ids.insert(member.method_id.clone()) {
                    return Err(reg("MANIFEST_DUPLICATE_ID", "duplicate member identity"));
                }
                match member.role.as_str() {
                    "ORDINARY" => ordinary += 1,
                    "ASSOCIATED" => associated += 1,
                    _ => {
                        return Err(reg("MANIFEST_METHOD_ROLE_INVALID", "member role"));
                    }
                }
            }
            if ordinary != 3 || associated != 1 {
                return Err(reg(
                    "MANIFEST_METHOD_COUNT_INVALID",
                    "new C# view must select 3 bodies plus 1 associated callback",
                ));
            }
            if method_ids.iter().cloned().collect::<BTreeSet<_>>()
                != composition
                    .new_method_ids
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>()
            {
                return Err(reg(
                    "MANIFEST_METHOD_SET_INVALID",
                    "manifest new method ids do not match member selectors",
                ));
            }
        } else {
            return Err(reg("MANIFEST_MODE_INVALID", "unknown import mode"));
        }
        if import.layer == "CS" && !import.function_views.is_empty() {
            return Err(reg(
                "MANIFEST_FUNCTION_SET_INVALID",
                "managed C# view must not carry non-managed function selectors",
            ));
        }
        for view in &import.function_views {
            validate_function_view(manifest, import, view)?;
            if !function_ids.insert(view.function_id.clone()) {
                return Err(reg(
                    "MANIFEST_FUNCTION_ID_DUPLICATE",
                    "function_id is duplicated",
                ));
            }
        }
        for anchor in &import.proof_anchors {
            crate::validate_relative_path(&anchor.path)
                .map_err(|_| reg("MANIFEST_PATH_INVALID", "proof anchor path"))?;
            if !valid_hex64(&anchor.sha256) || anchor.evidence_class.is_empty() {
                return Err(reg("MANIFEST_HASH_INVALID", "proof anchor hash/class"));
            }
            if anchor.range_start.is_some()
                || anchor.range_end.is_some()
                || anchor.range_sha256.is_some()
            {
                if anchor.range_start.is_none()
                    || anchor.range_end.is_none()
                    || !anchor
                        .range_sha256
                        .as_deref()
                        .map(valid_hex64)
                        .unwrap_or(false)
                    || anchor.contains.is_empty()
                {
                    return Err(reg(
                        "MANIFEST_PROOF_RANGE_INVALID",
                        "bounded proof anchors require range hash and contains tokens",
                    ));
                }
            }
        }
    }
    if deterministic != 1 {
        return Err(reg(
            "MANIFEST_IMPORT_SET_INVALID",
            "exactly one deterministic C# view is expected",
        ));
    }
    if import_ids
        != REQUIRED_IMPORT_IDS
            .iter()
            .map(|value| value.to_string())
            .collect::<BTreeSet<_>>()
    {
        return Err(reg(
            "MANIFEST_IMPORT_SET_INVALID",
            "eight import ids are not the exact accepted set",
        ));
    }
    let mut family_ids = BTreeSet::new();
    for rule in &manifest.family_rules {
        if !family_ids.insert(rule.family_id.clone()) {
            return Err(reg("MANIFEST_FAMILY_DUPLICATE", "duplicate family id"));
        }
        if !matches!(
            rule.match_kind.as_str(),
            "PREFIX" | "SUFFIX" | "EXACT" | "OTHERWISE"
        ) {
            return Err(reg("MANIFEST_FAMILY_INVALID", "family match kind"));
        }
    }
    if manifest
        .family_rules
        .iter()
        .map(|rule| rule.snapshot_count)
        .sum::<u64>()
        != 41456
    {
        return Err(reg(
            "MANIFEST_FAMILY_INVALID",
            "family snapshot counts must total the accepted 41456 rows",
        ));
    }
    Ok(())
}

fn method_id(authority: &str, mvid: &str, type_def: &str, method_def: &str) -> String {
    crate::sha256_hex(
        format!(
            "{}\0{}\0{}\0{}",
            authority.to_ascii_lowercase(),
            mvid.to_ascii_lowercase(),
            type_def.to_ascii_lowercase(),
            method_def.to_ascii_lowercase()
        )
        .as_bytes(),
    )
}

fn resolve_ref<'a>(roots: &Roots, artifact: &ArtifactRef) -> RegistryResult<PathBuf> {
    let base = match artifact.root_id.as_str() {
        "RESEARCH_ROOT" => &roots.research,
        "CURATED_ROOT" => &roots.curated,
        _ => return Err(reg("ROOT_ID_INVALID", "artifact root_id")),
    };
    crate::validate_relative_path(&artifact.path)
        .map_err(|_| reg("UNSAFE_RELATIVE_PATH", "artifact path"))?;
    let relative = Path::new(&artifact.path);
    match inspect_relative_entry(base, relative) {
        Ok(EntryKind::File) => Ok(base.join(relative)),
        Ok(EntryKind::Directory) => Err(reg("PATH_NOT_FILE", "artifact path is a directory")),
        Err("MISSING_PATH") => Err(reg("MISSING_PATH", "artifact path is missing")),
        Err("UNSAFE_REPARSE_OR_SYMLINK") => Err(reg(
            "UNSAFE_REPARSE_OR_SYMLINK",
            "artifact path crosses a reparse object",
        )),
        Err(code) => Err(reg(
            "PATH_INVALID",
            format!("artifact path rejected with {code}"),
        )),
    }
}

fn read_artifact(roots: &Roots, artifact: &ArtifactRef) -> RegistryResult<Vec<u8>> {
    let path = resolve_ref(roots, artifact)?;
    inspect_existing_file(&path)?;
    let bytes = fs::read(&path).map_err(|_| reg("FILE_READ_ERROR", "artifact read failed"))?;
    if bytes.len() as u64 != artifact.size || crate::sha256_hex(&bytes) != artifact.sha256 {
        return Err(reg(
            "ARTIFACT_HASH_MISMATCH",
            format!("artifact bytes differ for {}", artifact.path),
        ));
    }
    Ok(bytes)
}

fn current_state(roots: &Roots, root_id: &str, path: &str) -> RegistryResult<CurrentState> {
    let base = match root_id {
        "RESEARCH_ROOT" => &roots.research,
        "CURATED_ROOT" => &roots.curated,
        _ => return Err(reg("ROOT_ID_INVALID", "current state root")),
    };
    crate::validate_relative_path(path).map_err(|_| reg("UNSAFE_RELATIVE_PATH", "current path"))?;
    let relative = Path::new(path);
    match inspect_relative_entry(base, relative) {
        Ok(EntryKind::File) => {
            let metadata = fs::symlink_metadata(base.join(relative))
                .map_err(|_| reg("PATH_METADATA_ERROR", "current metadata"))?;
            Ok(CurrentState::Present(metadata.len()))
        }
        Ok(EntryKind::Directory) => Ok(CurrentState::Missing),
        Err("MISSING_PATH") => Ok(CurrentState::Missing),
        Err("UNSAFE_REPARSE_OR_SYMLINK") => Err(reg(
            "UNSAFE_REPARSE_OR_SYMLINK",
            "current path crosses a reparse object",
        )),
        Err(code) => Err(reg(
            "PATH_INVALID",
            format!("current path rejected with {code}"),
        )),
    }
}

fn line_span(bytes: &[u8], range: &LineRange) -> Result<Vec<u8>, String> {
    if range.start_line == 0 || range.end_line < range.start_line {
        return Err("invalid line range".to_string());
    }
    let mut line = 1_u64;
    let mut start = 0_usize;
    let mut output = Vec::new();
    let mut found_end = false;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            if line >= range.start_line && line <= range.end_line {
                output.extend_from_slice(&bytes[start..=index]);
            }
            if line == range.end_line {
                found_end = true;
            }
            line += 1;
            start = index + 1;
        }
    }
    if start < bytes.len() {
        if line >= range.start_line && line <= range.end_line {
            output.extend_from_slice(&bytes[start..]);
        }
        if line == range.end_line {
            found_end = true;
        }
    }
    if !found_end {
        return Err("line range exceeds input".to_string());
    }
    Ok(output)
}

fn line_count(bytes: &[u8]) -> u64 {
    let mut count = bytes.iter().filter(|byte| **byte == b'\n').count() as u64;
    if !bytes.is_empty() && !bytes.ends_with(b"\n") {
        count += 1;
    }
    count
}

fn check_anchor(roots: &Roots, anchor: &ProofAnchor) -> Result<(), String> {
    let artifact = ArtifactRef {
        root_id: anchor.root_id.clone(),
        path: anchor.path.clone(),
        size: anchor.size,
        sha256: anchor.sha256.clone(),
    };
    let path = resolve_ref(roots, &artifact).map_err(|error| error.detail.clone())?;
    let bytes = fs::read(&path).map_err(|_| "proof anchor unreadable".to_string())?;
    if bytes.len() as u64 != anchor.size || crate::sha256_hex(&bytes) != anchor.sha256 {
        return Err("proof anchor whole-file hash mismatch".to_string());
    }
    let bounded_text;
    let text = if let (Some(start), Some(end), Some(expected)) =
        (anchor.range_start, anchor.range_end, &anchor.range_sha256)
    {
        let span = line_span(
            &bytes,
            &LineRange {
                start_line: start,
                end_line: end,
            },
        )?;
        if crate::sha256_hex(&span) != *expected {
            return Err("proof anchor line-range hash mismatch".to_string());
        }
        bounded_text = String::from_utf8_lossy(&span).to_string();
        bounded_text.as_str()
    } else if anchor.range_start.is_some()
        || anchor.range_end.is_some()
        || anchor.range_sha256.is_some()
    {
        return Err("proof anchor range is incomplete".to_string());
    } else {
        bounded_text = String::from_utf8_lossy(&bytes).to_string();
        bounded_text.as_str()
    };
    for needle in &anchor.contains {
        if !text.contains(needle) {
            return Err(format!("proof anchor missing required token: {needle}"));
        }
    }
    if let Some(record_id) = &anchor.record_id {
        let mut found = false;
        for line in text.lines() {
            let value = match crate::parse_strict_value(line.as_bytes()) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if value.get("id").and_then(Value::as_str) != Some(record_id.as_str()) {
                continue;
            }
            if let Some(required) = &anchor.record_disposition {
                if value.get("disposition").and_then(Value::as_str) != Some(required.as_str()) {
                    continue;
                }
            }
            found = true;
            break;
        }
        if !found {
            return Err("proof anchor accepted record was not found".to_string());
        }
    }
    Ok(())
}

fn merge_json(mut base: Value, extra: Value) -> Value {
    if let (Some(base), Some(extra)) = (base.as_object_mut(), extra.as_object()) {
        for (key, value) in extra {
            base.insert(key.clone(), value.clone());
        }
    }
    base
}

fn finding(code: &str, path: &str, detail: &str) -> Value {
    json!({"code": code, "path": path, "detail": detail})
}

fn read_proof_file(roots: &Roots, path: &str, size: u64, sha256: &str) -> Result<Vec<u8>, String> {
    let artifact = ArtifactRef {
        root_id: "RESEARCH_ROOT".to_string(),
        path: path.to_string(),
        size,
        sha256: sha256.to_string(),
    };
    read_artifact(roots, &artifact).map_err(|error| error.detail)
}

fn check_member_capture(roots: &Roots, member: &MemberSelector) -> Result<(), String> {
    let capture = read_proof_file(
        roots,
        &member.proof.capture_path,
        member.proof.capture_file_size,
        &member.proof.capture_file_sha256,
    )?;
    let range = LineRange {
        start_line: member.proof.capture_start_line,
        end_line: member.proof.capture_end_line,
    };
    let span = line_span(&capture, &range)?;
    if crate::sha256_hex(&span) != member.proof.capture_section_sha256 {
        return Err("member capture section hash mismatch".to_string());
    }
    let text = String::from_utf8_lossy(&span);
    for token in &member.proof.required_il_tokens {
        if !text.contains(token) {
            return Err(format!("member capture missing token: {token}"));
        }
    }
    if !text.contains(&format!("methoddef={}", member.proof.capture_methoddef))
        || !text.contains(&format!("rva={}", member.proof.capture_rva))
        || !text.contains(&format!("code_size={}", member.proof.capture_code_size))
    {
        return Err("member capture header does not bind MethodDef/RVA/code size".to_string());
    }
    let evidence = read_proof_file(
        roots,
        &member.proof.evidence_path,
        member.proof.evidence_file_size,
        &member.proof.evidence_file_sha256,
    )?;
    let value = crate::parse_strict_value(&evidence)
        .map_err(|_| "PERF11 evidence is not strict JSON".to_string())?;
    let selected = value
        .get("selected_methods")
        .and_then(Value::as_array)
        .ok_or_else(|| "PERF11 evidence selected_methods missing".to_string())?;
    let entry = selected
        .iter()
        .find(|entry| {
            entry.get("methoddef").and_then(Value::as_str)
                == Some(member.proof.evidence_methoddef.as_str())
                && entry.get("name").and_then(Value::as_str)
                    == Some(member.proof.evidence_selector.as_str())
        })
        .ok_or_else(|| "PERF11 evidence selector does not bind this MethodDef".to_string())?;
    if entry.get("direct_cil_lines").and_then(Value::as_str)
        != Some(
            format!(
                "{}-{}",
                member.proof.capture_start_line, member.proof.capture_end_line
            )
            .as_str(),
        )
    {
        return Err("PERF11 evidence lines do not match the bounded capture range".to_string());
    }
    if let Some(section_sha) = entry
        .get("direct_cil_section_sha256")
        .and_then(Value::as_str)
    {
        if section_sha != member.proof.capture_section_sha256 {
            return Err("PERF11 evidence section hash mismatch".to_string());
        }
    }
    Ok(())
}

fn check_function_view(
    roots: &Roots,
    import: &SourceImport,
    view: &FunctionView,
) -> Result<(), String> {
    let source = read_artifact(roots, &view.source).map_err(|error| error.detail)?;
    let lines = source
        .split_inclusive(|byte| *byte == b'\n')
        .collect::<Vec<_>>();
    if view.source_span.start_line as usize > lines.len()
        || view.source_span.end_line as usize > lines.len()
    {
        return Err("function source span exceeds the saved source".to_string());
    }
    let span = line_span(
        &source,
        &LineRange {
            start_line: view.source_span.start_line,
            end_line: view.source_span.end_line,
        },
    )?;
    if crate::sha256_hex(&span) != view.source_span.sha256 {
        return Err("function source span hash mismatch".to_string());
    }
    let header_line = lines[view.source_span.start_line as usize - 1];
    if !String::from_utf8_lossy(header_line).contains(&view.header_token) {
        return Err("function header token is not on the target start line".to_string());
    }
    if import.layer == "CPP" {
        let locator = view
            .native_locator
            .as_ref()
            .ok_or_else(|| "native function locator missing".to_string())?;
        if locator.binary_sha256.is_empty() || locator.rva.is_empty() {
            return Err("native function locator incomplete".to_string());
        }
    }
    Ok(())
}

fn verify_imports(roots: &Roots, manifest: &SourceManifest) -> (Vec<Value>, BTreeMap<String, u64>) {
    let mut findings = Vec::new();
    let mut counts = BTreeMap::new();
    let mut exact = 0_u64;
    let mut transformed = 0_u64;
    let mut proof_checked = 0_u64;
    let mut functions_checked = 0_u64;
    for import in &manifest.imports {
        match read_artifact(roots, &import.source) {
            Ok(_) => {}
            Err(error) => findings.push(finding(
                "IMPORT_SOURCE_INVALID",
                &import.source.path,
                &error.detail,
            )),
        }
        match read_artifact(roots, &import.destination) {
            Ok(destination) => {
                if import.mode == "EXACT_BYTE_COPY" {
                    exact += 1;
                    let source = read_artifact(roots, &import.source).unwrap_or_default();
                    if source != destination {
                        findings.push(finding(
                            "IMPORT_BYTE_COPY_DRIFT",
                            &import.destination.path,
                            "destination bytes differ from source",
                        ));
                    }
                } else if let Some(transform) = &import.transform {
                    transformed += 1;
                    let source = read_artifact(roots, &transform.input).unwrap_or_default();
                    let mut rebuilt = transform.banner.as_bytes().to_vec();
                    let mut chunk_index = 0_usize;
                    let mut chunk_ok = true;
                    let mut output_line = transform.banner.matches('\n').count() as u64 + 1;
                    for range in &transform.ranges {
                        match line_span(&source, range) {
                            Ok(span) => {
                                let chunk = &transform.chunks[chunk_index];
                                if crate::sha256_hex(&span) != chunk.sha256 {
                                    findings.push(finding(
                                        "TRANSFORM_CHANGED_CHUNK",
                                        &transform.input.path,
                                        "selected chunk hash mismatch",
                                    ));
                                    chunk_ok = false;
                                }
                                let span_lines = line_count(&span);
                                if chunk.output_start_line != output_line
                                    || chunk.output_end_line != output_line + span_lines - 1
                                {
                                    findings.push(finding(
                                        "TRANSFORM_OUTPUT_RANGE_INVALID",
                                        &transform.input.path,
                                        "chunk output range mismatch",
                                    ));
                                    chunk_ok = false;
                                }
                                output_line += span_lines;
                                rebuilt.extend_from_slice(&span);
                            }
                            Err(error) => {
                                findings.push(finding(
                                    "TRANSFORM_RANGE_INVALID",
                                    &transform.input.path,
                                    &error,
                                ));
                                chunk_ok = false;
                            }
                        }
                        chunk_index += 1;
                    }
                    if chunk_ok && rebuilt != destination {
                        findings.push(finding(
                            "TRANSFORM_OUTPUT_MISMATCH",
                            &import.destination.path,
                            "reconstructed view differs from destination bytes",
                        ));
                    }
                    if rebuilt.len() as u64 != import.destination.size
                        || crate::sha256_hex(&rebuilt) != import.destination.sha256
                    {
                        findings.push(finding(
                            "TRANSFORM_OUTPUT_HASH_MISMATCH",
                            &import.destination.path,
                            "reconstructed view size/hash mismatch",
                        ));
                    }
                    for member in &import.member_selectors {
                        if let (Some(input_range), Some(output_range)) =
                            (&member.source_input_lines, &member.output_lines)
                        {
                            match (
                                line_span(&source, input_range),
                                line_span(&destination, output_range),
                            ) {
                                (Ok(left), Ok(right)) if left == right => {
                                    if crate::sha256_hex(&left) != member.input_range_sha256
                                        || crate::sha256_hex(&right) != member.output_range_sha256
                                    {
                                        findings.push(finding(
                                            "MEMBER_RANGE_HASH_MISMATCH",
                                            &member.method_id,
                                            "member input/output range hash mismatch",
                                        ));
                                    }
                                }
                                (Ok(_), Ok(_)) => findings.push(finding(
                                    "MEMBER_SOURCE_RANGE_MISMATCH",
                                    &member.method_id,
                                    "member output range does not preserve selected source range",
                                )),
                                (Err(error), _) | (_, Err(error)) => findings.push(finding(
                                    "MEMBER_RANGE_INVALID",
                                    &member.method_id,
                                    &error,
                                )),
                            }
                        }
                        if let Err(detail) = check_member_capture(roots, member) {
                            findings.push(finding(
                                "MEMBER_CAPTURE_PROOF_INVALID",
                                &member.method_id,
                                &detail,
                            ));
                        }
                    }
                }
            }
            Err(error) => findings.push(finding(
                "IMPORT_DESTINATION_INVALID",
                &import.destination.path,
                &error.detail,
            )),
        }
        for anchor in &import.proof_anchors {
            proof_checked += 1;
            if let Err(detail) = check_anchor(roots, anchor) {
                findings.push(finding("PROOF_ANCHOR_MISMATCH", &anchor.path, &detail));
            }
        }
        for view in &import.function_views {
            functions_checked += 1;
            if let Err(detail) = check_function_view(roots, import, view) {
                findings.push(finding("FUNCTION_VIEW_INVALID", &view.function_id, &detail));
            }
        }
    }
    counts.insert("exact_imports".to_string(), exact);
    counts.insert("deterministic_views".to_string(), transformed);
    counts.insert("proof_anchors".to_string(), proof_checked);
    counts.insert("function_views".to_string(), functions_checked);
    (findings, counts)
}

fn verify_non_managed(roots: &Roots, manifest: &SourceManifest) -> (Vec<Value>, u64) {
    let mut findings = Vec::new();
    let mut checked = 0_u64;
    for row in &manifest.non_managed_artifacts {
        checked += 1;
        if let Err(error) = read_artifact(roots, &row.artifact) {
            findings.push(finding(
                "ORIGIN_ARTIFACT_INVALID",
                &row.artifact.path,
                &error.detail,
            ));
        }
        for anchor in &row.proof_anchors {
            if let Err(detail) = check_anchor(roots, anchor) {
                findings.push(finding("PROOF_ANCHOR_MISMATCH", &anchor.path, &detail));
            }
        }
    }
    (findings, checked)
}

fn verify_file_hash(path: &Path, size: u64, expected: &str) -> RegistryResult<()> {
    inspect_existing_file(path)?;
    let mut file = File::open(path).map_err(|_| reg("FILE_READ_ERROR", "file open failed"))?;
    let mut hasher = Sha256::new();
    let mut bytes = 0_u64;
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| reg("FILE_READ_ERROR", "file read failed"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes += read as u64;
    }
    if bytes != size || hex_digest(hasher.finalize()) != expected {
        return Err(reg("PAYLOAD_HASH_MISMATCH", "payload bytes differ"));
    }
    Ok(())
}

fn load_snapshot_verified(
    roots: &Roots,
    manifest: &SourceManifest,
) -> RegistryResult<Vec<SnapshotRow>> {
    let baseline = &manifest.baseline.snapshot;
    let bytes = read_artifact(roots, &baseline.artifacts)?;
    if crate::sha256_hex(&bytes) != baseline.snapshot_id
        || bytes.len() as u64 != baseline.artifacts_bytes
    {
        return Err(reg("SNAPSHOT_ID_MISMATCH", "snapshot id bytes"));
    }
    let manifest_bytes = read_artifact(roots, &baseline.snapshot_manifest)?;
    let value = crate::parse_strict_value(&manifest_bytes)
        .map_err(|_| reg("SNAPSHOT_MANIFEST_INVALID", "snapshot manifest JSON"))?;
    if value.get("snapshot_id").and_then(Value::as_str) != Some(baseline.snapshot_id.as_str()) {
        return Err(reg("SNAPSHOT_MANIFEST_INVALID", "snapshot manifest id"));
    }
    if value.get("artifact_entries").and_then(Value::as_u64) != Some(baseline.artifact_entries) {
        return Err(reg("SNAPSHOT_MANIFEST_INVALID", "snapshot entry count"));
    }
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for line in bytes.split(|byte| *byte == b'\n') {
        if line.is_empty() {
            continue;
        }
        let value = crate::parse_strict_value(line)
            .map_err(|_| reg("SNAPSHOT_ROW_INVALID", "snapshot row JSON"))?;
        let row: SnapshotRow = serde_json::from_value(value)
            .map_err(|_| reg("SNAPSHOT_ROW_INVALID", "snapshot row shape"))?;
        if !matches!(row.root_id.as_str(), "RESEARCH_ROOT" | "CURATED_ROOT")
            || !super::validate_snapshot_path(&row.path)
            || !valid_hex64(&row.artifact_id)
            || crate::sha256_hex(format!("{}\0{}", row.root_id, row.path).as_bytes())
                != row.artifact_id
        {
            return Err(reg("SNAPSHOT_ROW_INVALID", "snapshot identity/path"));
        }
        if !seen.insert((row.root_id.clone(), row.path.clone())) {
            return Err(reg("SNAPSHOT_ROW_INVALID", "duplicate snapshot identity"));
        }
        rows.push(row);
    }
    if rows.len() as u64 != baseline.artifact_entries {
        return Err(reg("SNAPSHOT_ROW_INVALID", "snapshot row count"));
    }
    Ok(rows)
}

fn load_graph_summary(
    roots: &Roots,
    manifest: &SourceManifest,
    verify_methods: bool,
) -> RegistryResult<GraphSummaryLite> {
    let baseline = &manifest.baseline.graph;
    let graph_dir = roots.curated.join(&baseline.path);
    inspect_existing_directory(&graph_dir)?;
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(&graph_dir)
        .map_err(|_| reg("GRAPH_DIRECTORY_ERROR", "graph directory read"))?
    {
        let entry = entry.map_err(|_| reg("GRAPH_DIRECTORY_ERROR", "graph entry read"))?;
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|_| reg("GRAPH_DIRECTORY_ERROR", "graph entry metadata"))?;
        if is_reparse(&metadata) {
            return Err(reg("UNSAFE_REPARSE_OR_SYMLINK", "graph directory entry"));
        }
        names.insert(entry.file_name().to_string_lossy().to_string());
    }
    let expected_names = GRAPH_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<BTreeSet<_>>();
    if names != expected_names {
        return Err(reg("GRAPH_FILE_SET_MISMATCH", "graph directory set"));
    }
    for payload in &baseline.payloads {
        let verify = verify_methods || payload.name != "methods.jsonl";
        if verify {
            verify_file_hash(
                &graph_dir.join(&payload.name),
                payload.bytes,
                &payload.sha256,
            )?;
        }
    }
    let summary_bytes = fs::read(graph_dir.join("summary.json"))
        .map_err(|_| reg("GRAPH_SUMMARY_MISSING", "graph summary read"))?;
    let value = crate::parse_strict_value(&summary_bytes)
        .map_err(|_| reg("GRAPH_SUMMARY_INVALID", "graph summary JSON"))?;
    let summary: GraphSummaryLite = serde_json::from_value(value)
        .map_err(|_| reg("GRAPH_SUMMARY_INVALID", "graph summary shape"))?;
    if summary.schema != GRAPH_SCHEMA || summary.graph_id != baseline.graph_id {
        return Err(reg("GRAPH_SUMMARY_INVALID", "graph identity"));
    }
    let payloads = summary
        .payload_files
        .iter()
        .map(|file| (file.name.clone(), (file.bytes, file.sha256.clone())))
        .collect::<BTreeMap<_, _>>();
    for payload in &baseline.payloads {
        match payloads.get(&payload.name) {
            Some((bytes, sha)) if *bytes == payload.bytes && *sha == payload.sha256 => {}
            _ => return Err(reg("GRAPH_PAYLOAD_HASH_MISMATCH", "graph payload manifest")),
        }
    }
    let mut hasher = Sha256::new();
    for name in [
        "methods.jsonl",
        "artifacts.links.jsonl",
        "relationships.jsonl",
        "cohorts.jsonl",
        "missing.jsonl",
    ] {
        let payload = baseline
            .payloads
            .iter()
            .find(|payload| payload.name == name)
            .ok_or_else(|| reg("GRAPH_PAYLOAD_HASH_MISMATCH", "missing payload row"))?;
        hasher.update(name.as_bytes());
        hasher.update([0]);
        hasher.update(payload.sha256.as_bytes());
        hasher.update(b"\n");
    }
    if hex_digest(hasher.finalize()) != baseline.graph_id {
        return Err(reg("GRAPH_ID_MISMATCH", "graph payload id"));
    }
    Ok(summary)
}

fn expected_non_managed_paths(rows: &[SnapshotRow]) -> BTreeSet<String> {
    rows.iter()
        .filter(|row| {
            row.root_id == "RESEARCH_ROOT"
                && ((row.path.starts_with("src/reconstructed/")
                    && matches!(ext_of(&row.path), "ts" | "js" | "cpp" | "hpp"))
                    || (row.path.starts_with("include/reconstructed/")
                        && ext_of(&row.path) == "hpp")
                    || row.path == "evidence/performance/check_perf03b_semantics.js")
        })
        .map(|row| row.path.clone())
        .collect()
}

fn check_expected_non_managed(
    manifest: &SourceManifest,
    rows: &[SnapshotRow],
) -> (Vec<Value>, BTreeSet<String>) {
    let expected = expected_non_managed_paths(rows);
    let actual = manifest
        .non_managed_artifacts
        .iter()
        .map(|row| row.artifact.path.clone())
        .collect::<BTreeSet<_>>();
    let missing = expected.difference(&actual).cloned().collect::<Vec<_>>();
    let extra = actual.difference(&expected).cloned().collect::<Vec<_>>();
    let mut findings = Vec::new();
    if !missing.is_empty() || !extra.is_empty() {
        findings.push(json!({
            "code": "NON_MANAGED_SET_MISMATCH",
            "missing_from_manifest": missing,
            "extra_in_manifest": extra,
        }));
    }
    if expected.len() != 34 || actual.len() != 34 {
        findings.push(json!({
            "code": "NON_MANAGED_COUNT_MISMATCH",
            "snapshot_expected": expected.len(),
            "manifest": actual.len(),
        }));
    }
    (findings, expected)
}

fn rule_matches(rule: &FamilyRule, row: &SnapshotRow) -> bool {
    if let Some(kind) = &rule.snapshot_kind {
        if &row.kind != kind {
            return false;
        }
    }
    let ext = ext_of(&row.path);
    match rule.match_kind.as_str() {
        "OTHERWISE" => true,
        "EXACT" => row.path == rule.path_rule,
        "PREFIX" => {
            row.path.starts_with(&rule.path_rule)
                && (rule.extensions.is_empty() || rule.extensions.iter().any(|item| item == ext))
        }
        "SUFFIX" => rule.extensions.is_empty() || rule.extensions.iter().any(|item| item == ext),
        _ => false,
    }
}

fn family_partition(
    manifest: &SourceManifest,
    rows: &[SnapshotRow],
) -> RegistryResult<BTreeMap<String, u64>> {
    let mut counts = manifest
        .family_rules
        .iter()
        .map(|rule| (rule.family_id.clone(), 0_u64))
        .collect::<BTreeMap<_, _>>();
    for row in rows {
        let mut matched = None;
        for rule in &manifest.family_rules {
            if rule_matches(rule, row) {
                matched = Some(rule);
                break;
            }
        }
        let rule = matched.ok_or_else(|| {
            reg(
                "FAMILY_UNCLASSIFIED",
                format!("snapshot row is not covered by a family rule: {}", row.path),
            )
        })?;
        *counts
            .get_mut(&rule.family_id)
            .expect("family count initialized") += 1;
    }
    for rule in &manifest.family_rules {
        if counts.get(&rule.family_id).copied().unwrap_or(0) != rule.snapshot_count {
            return Err(reg(
                "FAMILY_COUNT_MISMATCH",
                format!("family count differs from manifest for {}", rule.family_id),
            ));
        }
    }
    Ok(counts)
}

fn is_under(path: &str, roots: &[&str]) -> bool {
    roots
        .iter()
        .any(|root| path == *root || path.starts_with(&format!("{root}/")))
}

fn walk_metadata(
    base: &Path,
    relative_root: &str,
    output: &mut BTreeMap<String, u64>,
    errors: &mut Vec<Value>,
) -> RegistryResult<()> {
    let root = base.join(relative_root);
    if !root.exists() {
        return Ok(());
    }
    inspect_existing_directory(&root)?;
    let mut stack = vec![root];
    while let Some(directory) = stack.pop() {
        let mut entries = fs::read_dir(&directory)
            .map_err(|_| reg("SURVEY_READ_ERROR", "survey directory read"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| reg("SURVEY_READ_ERROR", "survey entry read"))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if matches!(
                name.as_str(),
                "__pycache__" | ".pytest_cache" | ".ruff_cache" | "target"
            ) {
                continue;
            }
            let metadata = fs::symlink_metadata(&path)
                .map_err(|_| reg("SURVEY_METADATA_ERROR", "survey metadata"))?;
            if is_reparse(&metadata) {
                errors.push(json!({
                    "code": "UNSAFE_REPARSE_OR_SYMLINK",
                    "path": path.strip_prefix(base).unwrap_or(&path).to_string_lossy().replace('\\', "/")
                }));
                continue;
            }
            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file() {
                if path.extension().and_then(|value| value.to_str()) == Some("pyc") {
                    continue;
                }
                let relative = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                output.insert(relative, metadata.len());
            } else {
                errors.push(json!({
                    "code": "PATH_NOT_REGULAR",
                    "path": path.strip_prefix(base).unwrap_or(&path).to_string_lossy().replace('\\', "/")
                }));
            }
        }
    }
    Ok(())
}

struct SurveyReport {
    research_current: BTreeMap<String, u64>,
    curated_current: BTreeMap<String, u64>,
    missing: Vec<Value>,
    changed: Vec<Value>,
    new_research: Vec<Value>,
    new_curated: Vec<Value>,
    errors: Vec<Value>,
}

fn survey_roots_with_extra(
    roots: &Roots,
    extra_curated_roots: &[&str],
) -> RegistryResult<SurveyReport> {
    let mut research_current = BTreeMap::new();
    let mut curated_current = BTreeMap::new();
    let mut errors = Vec::new();
    for root in RESEARCH_SURVEY_ROOTS {
        walk_metadata(&roots.research, root, &mut research_current, &mut errors)?;
    }
    for root in CURATED_SURVEY_ROOTS {
        walk_metadata(&roots.curated, root, &mut curated_current, &mut errors)?;
    }
    for root in extra_curated_roots {
        walk_metadata(&roots.curated, root, &mut curated_current, &mut errors)?;
    }
    Ok(SurveyReport {
        research_current,
        curated_current,
        missing: Vec::new(),
        changed: Vec::new(),
        new_research: Vec::new(),
        new_curated: Vec::new(),
        errors,
    })
}

fn classify_new(path: &str, size: u64) -> Value {
    json!({"path": path, "size": size, "extension": ext_of(path)})
}

#[cfg(test)]
fn compare_survey(rows: &[SnapshotRow], manifest: &SourceManifest, survey: &mut SurveyReport) {
    compare_survey_with_extra(rows, manifest, survey, &[], &BTreeSet::new());
}

fn compare_survey_with_extra(
    rows: &[SnapshotRow],
    manifest: &SourceManifest,
    survey: &mut SurveyReport,
    extra_curated_roots: &[&str],
    extra_expected: &BTreeSet<String>,
) {
    let research_rows = rows
        .iter()
        .filter(|row| row.root_id == "RESEARCH_ROOT")
        .map(|row| (row.path.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let curated_rows = rows
        .iter()
        .filter(|row| row.root_id == "CURATED_ROOT")
        .map(|row| (row.path.clone(), row))
        .collect::<BTreeMap<_, _>>();
    for row in rows {
        if row.root_id == "RESEARCH_ROOT" && is_under(&row.path, &RESEARCH_SURVEY_ROOTS) {
            match survey.research_current.get(&row.path) {
                None => survey
                    .missing
                    .push(json!({"root_id": row.root_id, "path": row.path})),
                Some(size) if Some(*size) != row.size => survey.changed.push(json!({
                    "root_id": row.root_id, "path": row.path,
                    "expected_size": row.size, "current_size": *size
                })),
                Some(_) => {}
            }
        } else if row.root_id == "CURATED_ROOT"
            && (is_under(&row.path, &CURATED_SURVEY_ROOTS)
                || is_under(&row.path, extra_curated_roots))
        {
            match survey.curated_current.get(&row.path) {
                None => survey
                    .missing
                    .push(json!({"root_id": row.root_id, "path": row.path})),
                Some(size) if Some(*size) != row.size => survey.changed.push(json!({
                    "root_id": row.root_id, "path": row.path,
                    "expected_size": row.size, "current_size": *size
                })),
                Some(_) => {}
            }
        }
    }
    for (path, size) in &survey.research_current {
        if !research_rows.contains_key(path) {
            survey.new_research.push(classify_new(path, *size));
        }
    }
    for (path, size) in &survey.curated_current {
        if !curated_rows.contains_key(path) {
            survey.new_curated.push(classify_new(path, *size));
        }
    }
    survey.missing.sort_by(|left, right| {
        left.get("path")
            .and_then(Value::as_str)
            .unwrap_or("")
            .cmp(right.get("path").and_then(Value::as_str).unwrap_or(""))
    });
    survey.changed.sort_by(|left, right| {
        left.get("path")
            .and_then(Value::as_str)
            .unwrap_or("")
            .cmp(right.get("path").and_then(Value::as_str).unwrap_or(""))
    });
    survey.new_research.sort_by(|left, right| {
        left.get("path")
            .and_then(Value::as_str)
            .unwrap_or("")
            .cmp(right.get("path").and_then(Value::as_str).unwrap_or(""))
    });
    survey.new_curated.sort_by(|left, right| {
        left.get("path")
            .and_then(Value::as_str)
            .unwrap_or("")
            .cmp(right.get("path").and_then(Value::as_str).unwrap_or(""))
    });
    let expected_curated = manifest
        .baseline
        .accepted_composition
        .baseline_curated_paths
        .iter()
        .cloned()
        .chain(
            manifest
                .imports
                .iter()
                .map(|import| import.destination.path.clone()),
        )
        .chain(extra_expected.iter().cloned())
        .collect::<BTreeSet<_>>();
    let actual_curated = survey
        .curated_current
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let missing_imports = expected_curated
        .difference(&actual_curated)
        .cloned()
        .collect::<Vec<_>>();
    let extra_imports = actual_curated
        .difference(&expected_curated)
        .cloned()
        .collect::<Vec<_>>();
    if !missing_imports.is_empty() || !extra_imports.is_empty() {
        survey.errors.push(json!({
            "code": "NEW_CURATED_FILE_SET_MISMATCH",
            "missing": missing_imports,
            "extra": extra_imports,
        }));
    }
    if !survey.new_research.is_empty() {
        survey.errors.push(json!({
            "code": "NEW_RESEARCH_SOURCE_FILES",
            "count": survey.new_research.len(),
        }));
    }
    if !survey.missing.is_empty() {
        survey.errors.push(json!({
            "code": "REQUIRED_SOURCE_MISSING",
            "count": survey.missing.len(),
            "sample": survey.missing.iter().take(50).cloned().collect::<Vec<_>>(),
        }));
    }
    if !survey.changed.is_empty() {
        survey.errors.push(json!({
            "code": "REQUIRED_SOURCE_SIZE_CHANGED",
            "count": survey.changed.len(),
            "sample": survey.changed.iter().take(50).cloned().collect::<Vec<_>>(),
        }));
    }
}

fn survey_family_rows(
    rows: &[SnapshotRow],
    manifest: &SourceManifest,
    survey: &SurveyReport,
) -> Vec<Value> {
    let mut output = Vec::new();
    for (rule_index, rule) in manifest.family_rules.iter().enumerate() {
        let family_rows = rows
            .iter()
            .filter(|row| {
                manifest
                    .family_rules
                    .iter()
                    .position(|candidate| rule_matches(candidate, row))
                    == Some(rule_index)
            })
            .collect::<Vec<_>>();
        let mut surveyed = 0_u64;
        let mut present = 0_u64;
        let mut missing = 0_u64;
        let mut changed = 0_u64;
        for row in &family_rows {
            let current = if row.root_id == "RESEARCH_ROOT"
                && is_under(&row.path, &RESEARCH_SURVEY_ROOTS)
            {
                Some(survey.research_current.get(&row.path))
            } else if row.root_id == "CURATED_ROOT" && is_under(&row.path, &CURATED_SURVEY_ROOTS) {
                Some(survey.curated_current.get(&row.path))
            } else {
                None
            };
            if let Some(current) = current {
                surveyed += 1;
                match current {
                    None => missing += 1,
                    Some(size) if Some(*size) != row.size => changed += 1,
                    Some(_) => present += 1,
                }
            }
        }
        output.push(json!({
            "family_id": rule.family_id,
            "origin_class": rule.origin_class,
            "disposition_class": rule.disposition_class,
            "snapshot_count": rule.snapshot_count,
            "surveyed": surveyed,
            "present": present,
            "missing": missing,
            "changed_size": changed,
            "not_surveyed": rule.snapshot_count.saturating_sub(surveyed),
            "evidence_scope": rule.evidence_scope,
            "notes": rule.notes,
        }));
    }
    output
}

fn normalize_key(identity: &MethodIdentity) -> String {
    format!(
        "{}\0{}\0{}\0{}",
        identity.full_authority_sha256.to_ascii_lowercase(),
        identity.mvid.to_ascii_lowercase(),
        identity.type_def.to_ascii_lowercase(),
        identity.method_def.to_ascii_lowercase()
    )
}

fn old_key_hash(keys: &BTreeSet<String>) -> String {
    let mut hasher = Sha256::new();
    for key in keys {
        hasher.update(key.as_bytes());
        hasher.update(b"\n");
    }
    hex_digest(hasher.finalize())
}

struct GraphJoin {
    rows: u64,
    unique: u64,
    duplicates: Vec<String>,
    invalid: Vec<String>,
    representation_kinds: BTreeMap<String, u64>,
    evidence: BTreeMap<String, u64>,
    coverage: BTreeMap<String, u64>,
    axis_counts: BTreeMap<String, u64>,
    old_found: u64,
    old_absent: Vec<String>,
    new_found: Vec<String>,
    body_sha_rows: u64,
    body_hash_verified_true: u64,
    body_hash_verified_false: u64,
    raw_body_hash_only: u64,
    canonical_rows: u64,
    p1_rows: u64,
    direct_rows: u64,
    p1_body_rows: u64,
    source_association_rows: u64,
    raw_associated_rows: u64,
    accepted_current_rows: u64,
    new_scoped_rows: u64,
    body_present_outside_primary_exact: u64,
    candidate_raw_associated: u64,
}

struct HashingReader<R: Read> {
    inner: R,
    hasher: Sha256,
    bytes: u64,
}

impl<R: Read> Read for HashingReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buffer)?;
        self.hasher.update(&buffer[..read]);
        self.bytes += read as u64;
        Ok(read)
    }
}

fn classify_axes(
    row: &GraphMethodRow,
    accepted_class_by_id: &BTreeMap<String, String>,
    new_ids: &BTreeSet<String>,
    axis_counts: &mut BTreeMap<String, u64>,
    join: &mut GraphJoin,
) {
    let direct = row.representations.iter().any(|rep| {
        rep.evidence.iter().any(|item| {
            item == "DIRECT_CIL_FULL" || item == "DIRECT_IL_VERIFIED" || item == "FULL_CIL"
        }) || rep
            .coverage
            .as_deref()
            .map(|value| value.contains("DIRECT_IL_VERIFIED") || value.contains("FULL_CIL"))
            .unwrap_or(false)
    });
    let p1_body = row.representations.iter().any(|rep| {
        rep.representation_kind == "P1_METHOD_PROVENANCE"
            && rep.body_sha256.is_some()
            && rep.source_ref.is_some()
            && rep.proof_ref.is_some()
            && rep
                .coverage
                .as_deref()
                .map(|value| value == "DECOMPILE_OK/RESOLVED")
                .unwrap_or(false)
    });
    let canonical = row
        .representations
        .iter()
        .any(|rep| rep.representation_kind == "CANONICAL_METHOD_PROVENANCE");
    let source_association = row
        .representations
        .iter()
        .any(|rep| rep.source_ref.is_some());
    let raw = row.representations.iter().any(|rep| rep.raw_ref.is_some());
    let accepted = accepted_class_by_id.get(&row.method_id);
    let new_scoped = new_ids.contains(&row.method_id);
    let graph_label = if p1_body {
        "P1_DECOMPILE_OK_BODY_PRESENT"
    } else if direct {
        "GRAPH_DECLARED_CIL_EVIDENCE"
    } else if canonical {
        "CANONICAL_METADATA_RAW_PROVENANCE_SOURCE_ASSOCIATION"
    } else if source_association {
        "GRAPH_DECLARED_SOURCE_ONLY"
    } else if raw {
        "GRAPH_METADATA_RAW_ASSOCIATED"
    } else {
        "ABSTRACT_OR_UNRESOLVED_METADATA_ONLY"
    };
    let label = if let Some(class) = accepted {
        match class.as_str() {
            "DIRECT_CIL_FULL" => "ACCEPTED_DIRECT_CIL_FULL",
            "SOURCE_ONLY" => "ACCEPTED_SOURCE_ONLY",
            "UNKNOWN" => "ACCEPTED_UNKNOWN",
            "METADATA_ONLY" => "ACCEPTED_METADATA_ONLY",
            _ => "ACCEPTED_OTHER",
        }
    } else if new_scoped {
        "NEW_SCOPED_DIRECT_CIL_BACKED"
    } else {
        graph_label
    };
    *axis_counts.entry(label.to_string()).or_insert(0) += 1;
    if accepted.is_some() {
        join.accepted_current_rows += 1;
    }
    if direct {
        join.direct_rows += 1;
    }
    if p1_body {
        join.p1_body_rows += 1;
    }
    if canonical {
        join.canonical_rows += 1;
    }
    if source_association {
        join.source_association_rows += 1;
    }
    if raw {
        join.raw_associated_rows += 1;
    }
    if new_scoped {
        join.new_scoped_rows += 1;
    }
    if accepted.is_none() && !new_scoped && p1_body {
        join.body_present_outside_primary_exact += 1;
    }
    if row
        .representations
        .iter()
        .any(|rep| rep.body_sha256.is_some())
    {
        join.body_sha_rows += 1;
    }
    if row
        .representations
        .iter()
        .any(|rep| rep.body_hash_verified == Some(true))
    {
        join.body_hash_verified_true += 1;
    }
    if row
        .representations
        .iter()
        .any(|rep| rep.body_hash_verified == Some(false))
    {
        join.body_hash_verified_false += 1;
    }
    if row
        .representations
        .iter()
        .any(|rep| rep.evidence.iter().any(|item| item == "RAW_BODY_HASH_ONLY"))
    {
        join.raw_body_hash_only += 1;
    }
    if row
        .representations
        .iter()
        .any(|rep| rep.representation_kind == "P1_METHOD_PROVENANCE")
    {
        join.p1_rows += 1;
    }
    let candidate_raw_associated = row.representations.iter().any(|rep| {
        rep.body_sha256.is_some()
            || rep.raw_ref.is_some()
            || rep
                .coverage
                .as_deref()
                .map(|value| value.contains("DECOMPILE_OK"))
                .unwrap_or(false)
            || rep.evidence.iter().any(|item| item.contains("DIRECT"))
    });
    if candidate_raw_associated {
        join.candidate_raw_associated += 1;
    }
}

fn stream_graph_methods(
    roots: &Roots,
    manifest: &SourceManifest,
    old_ids: &BTreeSet<String>,
    accepted_class_by_id: &BTreeMap<String, String>,
    new_ids: &BTreeSet<String>,
) -> RegistryResult<GraphJoin> {
    let baseline = &manifest.baseline.graph;
    let payload = baseline
        .payloads
        .iter()
        .find(|payload| payload.name == "methods.jsonl")
        .ok_or_else(|| reg("GRAPH_PAYLOAD_HASH_MISMATCH", "methods payload missing"))?;
    let path = roots.curated.join(&baseline.path).join("methods.jsonl");
    inspect_existing_file(&path)?;
    let file = File::open(&path).map_err(|_| reg("GRAPH_METHODS_MISSING", "methods open"))?;
    let hashing = HashingReader {
        inner: file,
        hasher: Sha256::new(),
        bytes: 0,
    };
    let mut reader = BufReader::with_capacity(1024 * 1024, hashing);
    let mut seen = HashSet::new();
    let mut join = GraphJoin {
        rows: 0,
        unique: 0,
        duplicates: Vec::new(),
        invalid: Vec::new(),
        representation_kinds: BTreeMap::new(),
        evidence: BTreeMap::new(),
        coverage: BTreeMap::new(),
        axis_counts: BTreeMap::new(),
        old_found: 0,
        old_absent: Vec::new(),
        new_found: Vec::new(),
        body_sha_rows: 0,
        body_hash_verified_true: 0,
        body_hash_verified_false: 0,
        raw_body_hash_only: 0,
        canonical_rows: 0,
        p1_rows: 0,
        direct_rows: 0,
        p1_body_rows: 0,
        source_association_rows: 0,
        raw_associated_rows: 0,
        accepted_current_rows: 0,
        new_scoped_rows: 0,
        body_present_outside_primary_exact: 0,
        candidate_raw_associated: 0,
    };
    let mut line = String::new();
    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|_| reg("GRAPH_METHODS_INVALID", "methods read"))?;
        if read == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let value = crate::parse_strict_value(line.as_bytes())
            .map_err(|_| reg("GRAPH_METHODS_INVALID", "method row strict JSON"))?;
        let row: GraphMethodRow = serde_json::from_value(value)
            .map_err(|_| reg("GRAPH_METHODS_INVALID", "method row shape"))?;
        let computed = method_id(
            &row.assembly_sha256,
            &row.mvid,
            &row.type_token,
            &row.method_token,
        );
        if computed != row.method_id {
            join.invalid.push(row.method_id.clone());
        }
        if !seen.insert(row.method_id.clone()) {
            join.duplicates.push(row.method_id.clone());
        }
        join.rows += 1;
        for rep in &row.representations {
            *join
                .representation_kinds
                .entry(rep.representation_kind.clone())
                .or_insert(0) += 1;
            if let Some(coverage) = &rep.coverage {
                *join.coverage.entry(coverage.clone()).or_insert(0) += 1;
            }
            for evidence in &rep.evidence {
                *join.evidence.entry(evidence.clone()).or_insert(0) += 1;
            }
        }
        let mut scratch = BTreeMap::new();
        classify_axes(&row, accepted_class_by_id, new_ids, &mut scratch, &mut join);
        for (axis, count) in &scratch {
            *join.axis_counts.entry(axis.clone()).or_insert(0) += count;
        }
        if old_ids.contains(&row.method_id) {
            join.old_found += 1;
        }
        if new_ids.contains(&row.method_id) {
            join.new_found.push(row.method_id.clone());
        }
    }
    let (hash, bytes) = {
        let hashing = reader.into_inner();
        (hex_digest(hashing.hasher.finalize()), hashing.bytes)
    };
    if bytes != payload.bytes || hash != payload.sha256 {
        return Err(reg(
            "GRAPH_METHODS_HASH_MISMATCH",
            "methods payload changed during streaming read",
        ));
    }
    join.unique = seen.len() as u64;
    join.old_absent = old_ids
        .iter()
        .filter(|id| !seen.contains(*id))
        .cloned()
        .collect();
    for (axis, count) in &join.axis_counts {
        let _ = (axis, count);
    }
    Ok(join)
}

fn old_identities(
    roots: &Roots,
    manifest: &SourceManifest,
) -> RegistryResult<(BTreeSet<String>, String)> {
    let methods = crate::current_imported_method_identities(&roots.curated).map_err(|code| {
        reg(
            "CURRENT_AUTHORITY_ERROR",
            format!("current authority: {code}"),
        )
    })?;
    let normalized = methods.iter().map(normalize_key).collect::<BTreeSet<_>>();
    if normalized.len() as u64 != manifest.baseline.accepted_composition.old_qualified_keys {
        return Err(reg(
            "CURRENT_AUTHORITY_COUNT_DRIFT",
            "current imported qualified keys differ from the 533 baseline",
        ));
    }
    let ids = normalized
        .iter()
        .map(|key| {
            let parts = key.split('\0').collect::<Vec<_>>();
            method_id(parts[0], parts[1], parts[2], parts[3])
        })
        .collect::<BTreeSet<_>>();
    Ok((ids, old_key_hash(&normalized)))
}

fn accepted_classes(
    roots: &Roots,
    manifest: &SourceManifest,
) -> RegistryResult<(BTreeMap<String, String>, BTreeMap<String, u64>)> {
    let classes = crate::current_imported_method_classes(&roots.curated).map_err(|code| {
        reg(
            "CURRENT_AUTHORITY_ERROR",
            format!("current classes: {code}"),
        )
    })?;
    let mut by_id: BTreeMap<String, String> = BTreeMap::new();
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for entry in classes {
        let id = method_id(
            &entry.identity.full_authority_sha256,
            &entry.identity.mvid,
            &entry.identity.type_def,
            &entry.identity.method_def,
        );
        let rank = |class: &str| match class {
            "DIRECT_CIL_FULL" => 3_u8,
            "UNKNOWN" => 2,
            "SOURCE_ONLY" => 1,
            "METADATA_ONLY" => 0,
            _ => 0,
        };
        if let Some(existing) = by_id.get(&id) {
            if rank(&entry.evidence_status) > rank(existing) {
                by_id.insert(id, entry.evidence_status.clone());
            }
            continue;
        }
        by_id.insert(id, entry.evidence_status.clone());
    }
    for class in by_id.values() {
        *counts.entry(class.clone()).or_insert(0) += 1;
    }
    if by_id.len() as u64 != manifest.baseline.accepted_composition.old_qualified_keys {
        return Err(reg(
            "CURRENT_AUTHORITY_COUNT_DRIFT",
            "current accepted class map does not contain exactly 533 keys",
        ));
    }
    let expected = ACCEPTED_CLASS_COUNTS
        .iter()
        .map(|(name, count)| (name.to_string(), *count))
        .collect::<BTreeMap<_, _>>();
    if counts != expected {
        return Err(reg(
            "CURRENT_AUTHORITY_CLASS_DRIFT",
            "current accepted classes are not 246 DIRECT_CIL_FULL / 280 SOURCE_ONLY / 6 UNKNOWN / 1 METADATA_ONLY",
        ));
    }
    if counts != manifest.baseline.accepted_composition.accepted_class_counts {
        return Err(reg(
            "CURRENT_AUTHORITY_CLASS_DRIFT",
            "current accepted classes differ from the manifest candidate counts",
        ));
    }
    Ok((by_id, counts))
}

fn supplemented_class_counts(accepted: &BTreeMap<String, u64>) -> BTreeMap<String, u64> {
    let mut counts = accepted.clone();
    *counts.entry("DIRECT_CIL_FULL".to_string()).or_insert(0) += 4;
    counts
}

fn all_payload_finding(manifest: &SourceManifest, snapshot_rows: &[SnapshotRow]) -> Vec<Value> {
    let mut findings = Vec::new();
    let (set_findings, expected) = check_expected_non_managed(manifest, snapshot_rows);
    findings.extend(set_findings);
    if expected.len() != 34 {
        findings.push(json!({
            "code": "SNAPSHOT_NON_MANAGED_COUNT",
            "expected": expected.len()
        }));
    }
    findings
}

#[cfg(test)]
fn structural_status_value(manifest: &SourceManifest, bytes: &[u8], path: &Path) -> Value {
    structural_status_value_with_successor(manifest, bytes, path, None)
}

fn structural_status_value_with_successor(
    manifest: &SourceManifest,
    bytes: &[u8],
    path: &Path,
    successor: Option<&multilang::Successor>,
) -> Value {
    let mut dispositions = BTreeMap::new();
    let mut layers = BTreeMap::new();
    for row in &manifest.non_managed_artifacts {
        let label = serde_json::to_value(&row.disposition)
            .ok()
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_else(|| format!("{:?}", row.disposition));
        *dispositions.entry(label).or_insert(0) += 1;
        *layers.entry(row.layer.clone()).or_insert(0) += 1;
    }
    let family_total = manifest
        .family_rules
        .iter()
        .map(|rule| rule.snapshot_count)
        .sum::<u64>();
    json!({
        "command": "source-status",
        "ok": true,
        "exit": 0,
        "status": "STRUCTURAL_OK",
        "trust_status": trust_status(),
        "multilang_trust_status": multilang_trust_status(),
        "production_manifest_pin": PRODUCTION_MANIFEST_SHA256,
        "production_multilang_manifest_pin": PRODUCTION_MULTILANG_MANIFEST_SHA256,
        "multilang": successor.map(multilang::Successor::summary_value).unwrap_or(Value::Null),
        "verification": "NOT_CHECKED",
        "semantic_status": "NOT_CHECKED",
        "research_verification": "NOT_CHECKED",
        "manifest": {
            "path": path.to_string_lossy().replace('\\', "/"),
            "bytes": bytes.len(),
            "sha256": crate::sha256_hex(bytes)
        },
        "counts": {
            "non_managed_artifacts": manifest.non_managed_artifacts.len(),
            "imports": manifest.imports.len(),
            "backlog": manifest.backlog.len(),
            "family_rules": manifest.family_rules.len(),
            "family_snapshot_total": family_total,
            "function_views": manifest.imports.iter().map(|import| import.function_views.len()).sum::<usize>(),
            "old_c_files": manifest.baseline.accepted_composition.old_c_file_count,
            "old_qualified_keys": manifest.baseline.accepted_composition.old_qualified_keys,
            "new_c_files": manifest.baseline.accepted_composition.new_c_file_count,
            "supplemented_c_files": manifest.baseline.accepted_composition.supplemented_c_file_count,
            "supplemented_qualified_keys": manifest.baseline.accepted_composition.supplemented_qualified_keys
        },
        "dispositions": dispositions,
        "layers": layers,
        "findings": [],
        "limits": [
            "Structural validation only; research bytes and proof anchors were not read.",
            "The manifest is supplementary; it does not replace the 104-file/533-key baseline.",
            "Coverage reports are not self-proving authority."
        ]
    })
}

pub(super) fn run_status(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    let (exit, value) = status_with_trust(flags, &production_trust())?;
    print_cli(value);
    Ok(exit)
}

fn status_with_trust(
    flags: &BTreeMap<String, PathBuf>,
    trust: &Trust,
) -> RegistryResult<(u8, Value)> {
    let path = manifest_path(flags);
    let (manifest, bytes) = load_manifest_trusted(&path, trust)?;
    let roots = roots_from_flags(flags, false)?;
    let successor = if trust.successor_required {
        Some(multilang::load_candidate(
            &roots, &manifest, &bytes, trust, false,
        )?)
    } else {
        None
    };
    Ok((
        0,
        structural_status_value_with_successor(&manifest, &bytes, &path, successor.as_ref()),
    ))
}

fn current_lookup_row(roots: &Roots, row: &ArtifactDisposition) -> Value {
    match current_state(roots, &row.artifact.root_id, &row.artifact.path) {
        Ok(CurrentState::Present(size)) => json!({
            "status": "PRESENT",
            "size": size,
            "declared_size": row.artifact.size,
            "size_matches": size == row.artifact.size
        }),
        Ok(CurrentState::Missing) => json!({"status": "MISSING"}),
        Err(error) => json!({"status": "BLOCKED", "code": error.code, "detail": error.detail}),
    }
}

fn find_import_member<'a>(
    manifest: &'a SourceManifest,
    method_id: &str,
) -> Option<(&'a SourceImport, &'a MemberSelector)> {
    for import in &manifest.imports {
        for member in &import.member_selectors {
            if member.method_id == method_id {
                return Some((import, member));
            }
        }
    }
    None
}

fn load_links_verified(roots: &Roots, manifest: &SourceManifest) -> RegistryResult<Vec<Value>> {
    let payload = manifest
        .baseline
        .graph
        .payloads
        .iter()
        .find(|payload| payload.name == "artifacts.links.jsonl")
        .ok_or_else(|| reg("GRAPH_PAYLOAD_HASH_MISMATCH", "links payload missing"))?;
    let path = roots
        .curated
        .join(&manifest.baseline.graph.path)
        .join("artifacts.links.jsonl");
    inspect_existing_file(&path)?;
    let bytes = fs::read(&path).map_err(|_| reg("GRAPH_LINKS_MISSING", "links read"))?;
    if bytes.len() as u64 != payload.bytes || crate::sha256_hex(&bytes) != payload.sha256 {
        return Err(reg("GRAPH_PAYLOAD_HASH_MISMATCH", "links bytes"));
    }
    let mut rows = Vec::new();
    for line in bytes.split(|byte| *byte == b'\n') {
        if line.is_empty() {
            continue;
        }
        let value = crate::parse_strict_value(line)
            .map_err(|_| reg("GRAPH_LINKS_INVALID", "links row JSON"))?;
        rows.push(value);
    }
    Ok(rows)
}

fn find_method_stream(
    roots: &Roots,
    manifest: &SourceManifest,
    target: &str,
) -> RegistryResult<Option<Value>> {
    let baseline = &manifest.baseline.graph;
    let payload = baseline
        .payloads
        .iter()
        .find(|payload| payload.name == "methods.jsonl")
        .ok_or_else(|| reg("GRAPH_PAYLOAD_HASH_MISMATCH", "methods payload missing"))?;
    let path = roots.curated.join(&baseline.path).join("methods.jsonl");
    inspect_existing_file(&path)?;
    let file = File::open(&path).map_err(|_| reg("GRAPH_METHODS_MISSING", "methods open"))?;
    let hashing = HashingReader {
        inner: file,
        hasher: Sha256::new(),
        bytes: 0,
    };
    let mut reader = BufReader::with_capacity(1024 * 1024, hashing);
    let mut line = String::new();
    let mut found = None;
    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|_| reg("GRAPH_METHODS_INVALID", "methods read"))?;
        if read == 0 {
            break;
        }
        if !line.contains(target) {
            continue;
        }
        let value = crate::parse_strict_value(line.as_bytes())
            .map_err(|_| reg("GRAPH_METHODS_INVALID", "method row JSON"))?;
        if value.get("method_id").and_then(Value::as_str) != Some(target) {
            continue;
        }
        let row: GraphMethodRow = serde_json::from_value(value)
            .map_err(|_| reg("GRAPH_METHODS_INVALID", "method row shape"))?;
        let computed = method_id(
            &row.assembly_sha256,
            &row.mvid,
            &row.type_token,
            &row.method_token,
        );
        if computed != row.method_id {
            return Err(reg("GRAPH_METHODS_INVALID", "method id full-key mismatch"));
        }
        found = Some(serde_json::to_value(row).unwrap_or(Value::Null));
    }
    let (hash, bytes) = {
        let hashing = reader.into_inner();
        (hex_digest(hashing.hasher.finalize()), hashing.bytes)
    };
    if bytes != payload.bytes || hash != payload.sha256 {
        return Err(reg(
            "GRAPH_METHODS_HASH_MISMATCH",
            "methods payload changed during streaming read",
        ));
    }
    Ok(found)
}

fn find_function_view<'a>(
    manifest: &'a SourceManifest,
    function_id: &str,
) -> Option<(&'a SourceImport, &'a FunctionView)> {
    for import in &manifest.imports {
        for view in &import.function_views {
            if view.function_id == function_id {
                return Some((import, view));
            }
        }
    }
    None
}

fn artifact_ref_value(artifact: &ArtifactRef) -> Value {
    json!({
        "root_id": artifact.root_id,
        "path": artifact.path,
        "size": artifact.size,
        "sha256": artifact.sha256
    })
}

fn accepted_curated_imports(
    roots: &Roots,
    manifest: &SourceManifest,
    successor: Option<&multilang::Successor>,
    path: &str,
) -> RegistryResult<Vec<Value>> {
    let mut found: BTreeMap<String, Value> = BTreeMap::new();
    let active = crate::load_active_receipt(&roots.curated)
        .map_err(|error| reg("CURRENT_AUTHORITY_ERROR", error.code))?;
    if let Some(active) = active {
        let receipt = json!({"path": active.ledger_path, "version": active.version});
        for file in active.files {
            if file.imported.path != path {
                continue;
            }
            let value = json!({
                "kind": "ACTIVE_CURATED_IMPORT",
                "receipt": receipt,
                "record_or_file_id": file.id,
                "original_path": file.original_path,
                "destination": {
                    "root_id": "CURATED_ROOT",
                    "path": file.imported.path,
                    "size": file.imported.size,
                    "sha256": file.imported.sha256
                },
                "mode": "ACTIVE_IMPORTED_BYTES",
                "method_count": file.methods.len()
            });
            let key = format!(
                "ACTIVE_CURATED_IMPORT\0{}\0{}\0{}",
                file.id, file.imported.path, file.imported.sha256
            );
            found.insert(key, value);
        }
    }
    for import in &manifest.imports {
        if import.destination.path != path {
            continue;
        }
        let value = json!({
            "kind": if import.mode == "DETERMINISTIC_LINE_SELECTION" {
                "PREDECESSOR_SUPPLEMENT_SELECTION"
            } else {
                "PREDECESSOR_EXACT_IMPORT"
            },
            "import_id": import.id,
            "origin_id": import.origin_id,
            "source": artifact_ref_value(&import.source),
            "destination": artifact_ref_value(&import.destination),
            "mode": import.mode,
            "disposition": "SUPPLEMENT_IMPORT",
            "scope": import.scope,
            "evidence_status": import.evidence_status,
            "member_selector_count": import.member_selectors.len()
        });
        let key = format!(
            "PREDECESSOR_IMPORT\0{}\0{}\0{}",
            import.id, import.destination.path, import.destination.sha256
        );
        found.insert(key, value);
    }
    if let Some(successor) = successor {
        if let Some(mut value) = successor.physical_lookup_value(path) {
            if let Some(object) = value.as_object_mut() {
                object.insert(
                    "kind".to_string(),
                    Value::String("SUCCESSOR_PHYSICAL_IMPORT".to_string()),
                );
            }
            let key = format!(
                "SUCCESSOR_PHYSICAL_IMPORT\0{}\0{}\0{}",
                value["origin_id"], path, value["physical_import"]["sha256"]
            );
            found.insert(key, value);
        }
    }
    Ok(found.into_values().collect())
}

fn verify_current_artifact(
    roots: &Roots,
    root_id: &str,
    path: &str,
    declared_size: Option<u64>,
    declared_sha256: Option<&str>,
) -> RegistryResult<(bool, Value)> {
    match current_state(roots, root_id, path)? {
        CurrentState::Missing => Ok((
            false,
            json!({
                "status": "MISSING",
                "exists": false,
                "declared_size": declared_size,
                "declared_sha256": declared_sha256,
                "hash_checked": false
            }),
        )),
        CurrentState::Present(size) => {
            if let Some(expected) = declared_size
                && size != expected
            {
                return Err(reg(
                    "CURRENT_ARTIFACT_SIZE_MISMATCH",
                    "current artifact size differs from the accepted declared size",
                ));
            }
            let artifact = ArtifactRef {
                root_id: root_id.to_string(),
                path: path.to_string(),
                size: declared_size.unwrap_or(size),
                sha256: declared_sha256.unwrap_or_default().to_string(),
            };
            let resolved = resolve_ref(roots, &artifact)?;
            let bytes = fs::read(&resolved)
                .map_err(|_| reg("FILE_READ_ERROR", "current artifact read failed"))?;
            if let Some(expected) = declared_size
                && bytes.len() as u64 != expected
            {
                return Err(reg(
                    "CURRENT_ARTIFACT_SIZE_MISMATCH",
                    "current artifact size changed during read",
                ));
            }
            let actual_sha = crate::sha256_hex(&bytes);
            if let Some(expected) = declared_sha256
                && actual_sha != expected
            {
                return Err(reg(
                    "CURRENT_ARTIFACT_HASH_MISMATCH",
                    "current artifact bytes differ from the accepted declared hash",
                ));
            }
            Ok((
                true,
                json!({
                    "status": "PRESENT",
                    "exists": true,
                    "size": bytes.len(),
                    "sha256": actual_sha,
                    "declared_size": declared_size,
                    "declared_sha256": declared_sha256,
                    "size_matches": declared_size.map(|expected| expected == bytes.len() as u64),
                    "hash_matches": declared_sha256.map(|expected| expected == actual_sha),
                    "hash_checked": true,
                    // Line count is derived from the same bytes that were just
                    // no-follow resolved and hash-verified. It is not a fresh
                    // read of a possibly changed artifact.
                    "range_line_count": line_count(&bytes)
                }),
            ))
        }
    }
}

fn declared_bytes_from_accepted(
    accepted: &[Value],
    row: Option<&SnapshotRow>,
) -> RegistryResult<(Option<u64>, Option<String>)> {
    let mut declared_size = row.and_then(|row| row.size);
    let mut declared_sha256 = row.and_then(|row| row.sha256.clone());
    for import in accepted {
        let destination = import
            .get("destination")
            .or_else(|| import.get("physical_import"))
            .unwrap_or(&Value::Null);
        let size = destination.get("size").and_then(Value::as_u64);
        let sha = destination
            .get("sha256")
            .and_then(Value::as_str)
            .map(str::to_string);
        if size.is_none() || sha.is_none() {
            continue;
        }
        match (declared_size, declared_sha256.as_deref()) {
            (Some(current_size), Some(current_sha))
                if current_size != size.unwrap() || current_sha != sha.as_deref().unwrap() =>
            {
                return Err(reg(
                    "SOURCE_LOCATION_CONFLICT",
                    "accepted imports disagree on the destination bytes",
                ));
            }
            (None, _) => {
                declared_size = size;
                declared_sha256 = sha;
            }
            (_, None) => {
                declared_sha256 = sha;
            }
            _ => {}
        }
    }
    Ok((declared_size, declared_sha256))
}

fn accepted_artifact_lookup(
    roots: &Roots,
    manifest: &SourceManifest,
    successor: Option<&multilang::Successor>,
    accepted: Vec<Value>,
    artifact_path: &str,
    research: bool,
) -> RegistryResult<(u8, Value)> {
    let (declared_size, declared_sha256) = declared_bytes_from_accepted(&accepted, None)?;
    let (current_ok, current) = if research {
        verify_current_artifact(
            roots,
            "CURATED_ROOT",
            artifact_path,
            declared_size,
            declared_sha256.as_deref(),
        )?
    } else {
        (
            true,
            json!({"status": "NOT_CHECKED", "reason": "--research-root not supplied"}),
        )
    };
    let status = if current_ok {
        "FOUND"
    } else {
        "PARTIAL_MISSING_BACKING"
    };
    let mut limits = manifest.limits.clone();
    limits.push(
        "Exact accepted import short-circuit: the frozen 41456-row snapshot and 54MB link registry were not read."
            .to_string(),
    );
    let value = json!({
        "command": "source-lookup",
        "ok": current_ok,
        "exit": if current_ok { 0 } else { 1 },
        "status": status,
        "trust_status": trust_status(),
        "multilang_trust_status": multilang_trust_status(),
        "production_manifest_pin": PRODUCTION_MANIFEST_SHA256,
        "production_multilang_manifest_pin": PRODUCTION_MULTILANG_MANIFEST_SHA256,
        "mode": "ARTIFACT",
        "multilang": successor.and_then(|successor| successor.physical_lookup_value(artifact_path)),
        "baseline": Value::Null,
        "baseline_scope": "POST_SNAPSHOT_ACCEPTED_IMPORT",
        "accepted_imports": accepted,
        "accepted_import_count": accepted.len(),
        "post_snapshot_accepted": true,
        "supplement": Value::Null,
        "graph_link": Value::Null,
        "current": current,
        "graph_link_scope": "NOT_REQUESTED_EXACT_ACCEPTED_IMPORT",
        "historical_registry_join": "NOT_REQUESTED_EXACT_ACCEPTED_IMPORT",
        "verification": "NOT_CHECKED",
        "semantic_status": "NOT_CHECKED",
        "limits": limits
    });
    Ok((if current_ok { 0 } else { 1 }, value))
}

fn artifact_lookup_with_successor(
    roots: &Roots,
    manifest: &SourceManifest,
    successor: Option<&multilang::Successor>,
    root_id: &str,
    artifact_path: &str,
    research: bool,
) -> RegistryResult<(u8, Value)> {
    let accepted = if root_id == "CURATED_ROOT" {
        accepted_curated_imports(roots, manifest, successor, artifact_path)?
    } else {
        Vec::new()
    };
    if root_id == "CURATED_ROOT" && !accepted.is_empty() {
        return accepted_artifact_lookup(
            roots,
            manifest,
            successor,
            accepted,
            artifact_path,
            research,
        );
    }
    let snapshot_rows = load_snapshot_verified(roots, manifest)?;
    let row = snapshot_rows
        .iter()
        .find(|row| row.root_id == root_id && row.path == artifact_path)
        .ok_or_else(|| {
            reg(
                "SOURCE_NOT_FOUND",
                "artifact is neither in the frozen snapshot nor an exact accepted import",
            )
        })?;
    let supplement = manifest
        .non_managed_artifacts
        .iter()
        .find(|entry| entry.artifact.root_id == root_id && entry.artifact.path == artifact_path)
        .map(|entry| {
            json!({
                "origin_id": entry.origin_id,
                "disposition": entry.disposition,
                "primary_class": entry.primary_class,
                "scope": entry.scope,
                "evidence_status": entry.evidence_status,
                "import_id": entry.import_id
            })
        });
    let links = load_links_verified(roots, manifest)?;
    let join = links.iter().find_map(|link| {
        let artifact = link.get("artifact")?;
        if artifact.get("root_id").and_then(Value::as_str) != Some(root_id)
            || artifact.get("path").and_then(Value::as_str) != Some(artifact_path)
        {
            return None;
        }
        Some(json!({
            "roles": link.get("roles").cloned().unwrap_or(Value::Null),
            "method_ids": link.get("method_ids").cloned().unwrap_or(Value::Null),
            "cohorts": link.get("cohorts").cloned().unwrap_or(Value::Null),
            "notes": link.get("notes").cloned().unwrap_or(Value::Null)
        }))
    });
    let (declared_size, declared_sha256) = declared_bytes_from_accepted(&accepted, Some(row))?;
    let (current_ok, current) = if research {
        verify_current_artifact(
            roots,
            root_id,
            artifact_path,
            declared_size,
            declared_sha256.as_deref(),
        )?
    } else {
        (
            true,
            json!({"status": "NOT_CHECKED", "reason": "--research-root not supplied"}),
        )
    };
    let status = if current_ok {
        "FOUND"
    } else {
        "PARTIAL_MISSING_BACKING"
    };
    let value = json!({
        "command": "source-lookup",
        "ok": current_ok,
        "exit": if current_ok { 0 } else { 1 },
        "status": status,
        "trust_status": trust_status(),
        "multilang_trust_status": multilang_trust_status(),
        "production_manifest_pin": PRODUCTION_MANIFEST_SHA256,
        "production_multilang_manifest_pin": PRODUCTION_MULTILANG_MANIFEST_SHA256,
        "mode": "ARTIFACT",
        "multilang": successor
            .and_then(|successor| successor.artifact_lookup_value(root_id, artifact_path)),
        "baseline": row,
        "baseline_scope": "41456_FROZEN_SNAPSHOT",
        "accepted_imports": accepted,
        "accepted_import_count": 0,
        "post_snapshot_accepted": false,
        "supplement": supplement,
        "graph_link": join,
        "current": current,
        "graph_link_scope": "PINNED_LINKS_JOIN; methods not streamed",
        "historical_registry_join": "REQUESTED_RESEARCH_OR_UNKNOWN_CURATED_PATH",
        "verification": "NOT_CHECKED",
        "semantic_status": "NOT_CHECKED",
        "limits": manifest.limits
    });
    Ok((if current_ok { 0 } else { 1 }, value))
}

pub(super) fn run_lookup(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    let path = manifest_path(flags);
    let trust = production_trust();
    let (manifest, bytes) = load_manifest_trusted(&path, &trust)?;
    let roots = roots_from_flags(flags, false)?;
    let successor = if trust.successor_required {
        Some(multilang::load_candidate(
            &roots, &manifest, &bytes, &trust, true,
        )?)
    } else {
        None
    };
    let origin = flags_string(flags, "--origin-id");
    let function_id = flags_string(flags, "--function-id");
    let artifact_root = flags_string(flags, "--artifact-root-id");
    let artifact_path = flags_string(flags, "--artifact-path");
    let managed = [
        flags_string(flags, "--assembly-sha"),
        flags_string(flags, "--mvid"),
        flags_string(flags, "--type"),
        flags_string(flags, "--method"),
    ];
    let modes = usize::from(origin.is_some() || function_id.is_some())
        + usize::from(artifact_root.is_some() || artifact_path.is_some())
        + usize::from(managed.iter().any(Option::is_some));
    if modes != 1 {
        return Err(reg(
            "LOOKUP_MODE_INVALID",
            "choose exactly one of --origin-id, artifact root/path, or full managed identity",
        ));
    }
    let research = flags.get("--research-root").is_some();
    if let Some(origin) = origin {
        let row = manifest
            .non_managed_artifacts
            .iter()
            .find(|row| row.origin_id == origin)
            .ok_or_else(|| reg("SOURCE_NOT_FOUND", "origin-id is not in the supplement"))?;
        let current = if research {
            current_lookup_row(&roots, row)
        } else {
            json!({"status": "NOT_CHECKED", "reason": "--research-root not supplied"})
        };
        let import = row
            .import_id
            .as_ref()
            .and_then(|id| manifest.imports.iter().find(|import| &import.id == id));
        let function_views = import
            .map(|import| json!(import.function_views))
            .unwrap_or(Value::Null);
        let selected_function = function_id.as_ref().and_then(|id| {
            import.and_then(|import| {
                import
                    .function_views
                    .iter()
                    .find(|view| &view.function_id == id)
                    .map(|view| {
                        json!({
                            "import_id": import.id,
                            "origin_id": view.origin_id,
                            "function": view
                        })
                    })
            })
        });
        if function_id.is_some() && selected_function.is_none() {
            return Err(reg(
                "FUNCTION_NOT_FOUND",
                "function-id is not in the selected origin",
            ));
        }
        let import = import.map(|import| {
            json!({
                "id": import.id,
                "destination": import.destination,
                "mode": import.mode,
                "managed_methoddefs": import.managed_methoddefs
            })
        });
        print_cli(json!({
            "command": "source-lookup",
            "ok": true,
            "exit": 0,
            "status": "FOUND",
            "trust_status": trust_status(),
            "multilang_trust_status": multilang_trust_status(),
            "production_manifest_pin": PRODUCTION_MANIFEST_SHA256,
            "production_multilang_manifest_pin": PRODUCTION_MULTILANG_MANIFEST_SHA256,
            "mode": "ORIGIN_ID",
            "manifest_sha256": crate::sha256_hex(&bytes),
            "row": row,
            "multilang": successor
                .as_ref()
                .and_then(|successor| successor.lookup_value(&origin)),
            "import": import,
            "function_views": function_views,
            "function": selected_function,
            "current": current,
            "verification": "NOT_CHECKED",
            "limits": manifest.limits
        }));
        return Ok(0);
    }
    if let Some(root_id) = artifact_root {
        let artifact_path = artifact_path.ok_or_else(|| {
            reg(
                "LOOKUP_MODE_INVALID",
                "--artifact-path is required with --artifact-root-id",
            )
        })?;
        let (exit, value) = artifact_lookup_with_successor(
            &roots,
            &manifest,
            successor.as_ref(),
            &root_id,
            &artifact_path,
            research,
        )?;
        print_cli(value);
        return Ok(exit);
    }
    if let Some(function_id) = function_id {
        let (import, view) = find_function_view(&manifest, &function_id)
            .ok_or_else(|| reg("FUNCTION_NOT_FOUND", "function-id is not in the supplement"))?;
        print_cli(json!({
            "command": "source-lookup",
            "ok": true,
            "exit": 0,
            "status": "FOUND",
            "trust_status": trust_status(),
            "multilang_trust_status": multilang_trust_status(),
            "production_manifest_pin": PRODUCTION_MANIFEST_SHA256,
            "production_multilang_manifest_pin": PRODUCTION_MULTILANG_MANIFEST_SHA256,
            "mode": "FUNCTION_ID",
            "multilang": Value::Null,
            "manifest_sha256": crate::sha256_hex(&bytes),
            "import_id": import.id,
            "origin_id": view.origin_id,
            "function": view,
            "verification": "NOT_CHECKED",
            "limits": manifest.limits
        }));
        return Ok(0);
    }
    let [Some(assembly), Some(mvid), Some(type_def), Some(method_def)] = managed else {
        return Err(reg(
            "LOOKUP_MODE_INVALID",
            "full managed identity requires all four fields",
        ));
    };
    let target = method_id(&assembly, &mvid, &type_def, &method_def);
    let graph_roots = Roots {
        research: roots.research.clone(),
        curated: roots.curated.clone(),
    };
    let found = find_method_stream(&graph_roots, &manifest, &target)?;
    let member = find_import_member(&manifest, &target).map(|(import, member)| {
        json!({
            "import_id": import.id,
            "origin_id": import.origin_id,
            "destination": import.destination,
            "representation": member.representation,
            "scope": member.scope,
            "proof": member.proof,
            "evidence_status": import.evidence_status
        })
    });
    let status = if found.is_some() || member.is_some() {
        "FOUND"
    } else {
        "NOT_FOUND"
    };
    print_cli(json!({
        "command": "source-lookup",
        "ok": status == "FOUND",
        "exit": if status == "FOUND" { 0 } else { 1 },
        "status": status,
        "trust_status": trust_status(),
        "multilang_trust_status": multilang_trust_status(),
        "production_manifest_pin": PRODUCTION_MANIFEST_SHA256,
        "production_multilang_manifest_pin": PRODUCTION_MULTILANG_MANIFEST_SHA256,
        "mode": "MANAGED_IDENTITY",
        "multilang": Value::Null,
        "method_id": target,
        "method": found,
        "source_view": member,
        "verification_mode": "GRAPH_METHODS_STREAMED_WITH_HASH_AND_FULL_KEY_FILTER; INPUT_CORPUS_NOT_RECHECKED",
        "verification": "NOT_CHECKED",
        "limits": manifest.limits
    }));
    Ok(if status == "FOUND" { 0 } else { 1 })
}

fn survey_summary(survey: &SurveyReport, manifest: &SourceManifest) -> Value {
    let baseline_paths = manifest
        .baseline
        .accepted_composition
        .baseline_curated_paths
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let supplement_paths = manifest
        .imports
        .iter()
        .map(|import| import.destination.path.clone())
        .collect::<BTreeSet<_>>();
    let baseline_present = baseline_paths
        .iter()
        .filter(|path| survey.curated_current.contains_key(*path))
        .count();
    let supplement_present = supplement_paths
        .iter()
        .filter(|path| survey.curated_current.contains_key(*path))
        .count();
    let unexpected_new_curated = survey
        .curated_current
        .keys()
        .filter(|path| !baseline_paths.contains(*path) && !supplement_paths.contains(*path))
        .cloned()
        .collect::<Vec<_>>();
    let new_cs = survey
        .new_research
        .iter()
        .filter(|row| row.get("extension").and_then(Value::as_str) == Some("cs"))
        .count();
    let new_non_cs = survey
        .new_research
        .iter()
        .filter(|row| {
            matches!(
                row.get("extension").and_then(Value::as_str),
                Some("ts" | "js" | "cpp" | "hpp" | "h")
            )
        })
        .count();
    json!({
        "research_files_observed": survey.research_current.len(),
        "curated_files_observed": survey.curated_current.len(),
        "missing_count": survey.missing.len(),
        "changed_size_count": survey.changed.len(),
        "new_research_count": survey.new_research.len(),
        "new_research_cs_count": new_cs,
        "new_research_non_cs_count": new_non_cs,
        "new_curated_count": survey.new_curated.len(),
        "baseline_curated_expected": baseline_paths.len(),
        "baseline_curated_present": baseline_present,
        "supplement_curated_expected": supplement_paths.len(),
        "supplement_curated_present": supplement_present,
        "unexpected_new_curated": unexpected_new_curated.iter().take(100).cloned().collect::<Vec<_>>(),
        "unexpected_new_curated_count": unexpected_new_curated.len(),
        "errors": survey.errors,
        "missing": survey.missing.iter().take(200).cloned().collect::<Vec<_>>(),
        "changed_size": survey.changed.iter().take(200).cloned().collect::<Vec<_>>(),
        "new_research": survey.new_research.iter().take(200).cloned().collect::<Vec<_>>(),
        "new_curated": survey.new_curated.iter().take(200).cloned().collect::<Vec<_>>(),
        "truncated": survey.missing.len() > 200 || survey.changed.len() > 200
            || survey.new_research.len() > 200 || survey.new_curated.len() > 200
    })
}

fn survey_summary_with_successor(
    survey: &SurveyReport,
    manifest: &SourceManifest,
    successor: Option<&multilang::Successor>,
) -> Value {
    let mut value = survey_summary(survey, manifest);
    let Some(successor) = successor else {
        return value;
    };
    let baseline = manifest
        .baseline
        .accepted_composition
        .baseline_curated_paths
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let predecessor_supplement = manifest
        .imports
        .iter()
        .map(|import| import.destination.path.clone())
        .collect::<BTreeSet<_>>();
    let new_paths = successor.expected_paths();
    let supplement_expected = predecessor_supplement.len() + new_paths.len();
    let supplement_present = predecessor_supplement
        .iter()
        .chain(new_paths.iter())
        .filter(|path| survey.curated_current.contains_key(*path))
        .count();
    let unexpected = survey
        .curated_current
        .keys()
        .filter(|path| {
            !baseline.contains(*path)
                && !predecessor_supplement.contains(*path)
                && !new_paths.contains(*path)
        })
        .cloned()
        .collect::<Vec<_>>();
    let new_present = new_paths
        .iter()
        .filter(|path| survey.curated_current.contains_key(*path))
        .count();
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "supplement_curated_expected".to_string(),
            json!(supplement_expected),
        );
        object.insert(
            "supplement_curated_present".to_string(),
            json!(supplement_present),
        );
        object.insert("new_curated_expected".to_string(), json!(new_paths.len()));
        object.insert("new_curated_present".to_string(), json!(new_present));
        object.insert(
            "expected_curated_total".to_string(),
            json!(baseline.len() + supplement_expected),
        );
        object.insert(
            "unexpected_new_curated".to_string(),
            json!(unexpected.iter().take(100).cloned().collect::<Vec<_>>()),
        );
        object.insert(
            "unexpected_new_curated_count".to_string(),
            json!(unexpected.len()),
        );
        object.insert(
            "multilang_manifest_sha256".to_string(),
            json!(successor.sha256()),
        );
        object.insert(
            "multilang_physical_files".to_string(),
            json!(successor.summary_value()["summary"]["physical_files"].clone()),
        );
    }
    value
}

pub(super) fn run_verify(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    let (exit, value) = verify_with_trust(flags, &production_trust())?;
    print_cli(value);
    Ok(exit)
}

fn verify_with_trust(
    flags: &BTreeMap<String, PathBuf>,
    trust: &Trust,
) -> RegistryResult<(u8, Value)> {
    let path = manifest_path(flags);
    let (manifest, bytes) = load_manifest_trusted(&path, trust)?;
    let roots = roots_from_flags(flags, true)?;
    let successor = if trust.successor_required {
        Some(multilang::load_candidate(
            &roots, &manifest, &bytes, trust, true,
        )?)
    } else {
        None
    };
    let snapshot_rows = load_snapshot_verified(&roots, &manifest)?;
    let _summary = load_graph_summary(&roots, &manifest, false)?;
    let (mut findings, _artifact_count) = verify_non_managed(&roots, &manifest);
    let (import_findings, import_counts) = verify_imports(&roots, &manifest);
    findings.extend(import_findings);
    let set_findings = all_payload_finding(&manifest, &snapshot_rows);
    findings.extend(set_findings);
    let family_counts = family_partition(&manifest, &snapshot_rows)?;
    let (old_ids, old_hash) = old_identities(&roots, &manifest)?;
    let (_, accepted_counts) = accepted_classes(&roots, &manifest)?;
    let supplemented_counts = supplemented_class_counts(&accepted_counts);
    if supplemented_counts
        != manifest
            .baseline
            .accepted_composition
            .supplemented_class_counts
    {
        findings.push(json!({
            "code": "SUPPLEMENTED_CLASS_COUNTS_MISMATCH",
            "expected": manifest.baseline.accepted_composition.supplemented_class_counts,
            "actual": supplemented_counts,
        }));
    }
    let new_ids = manifest
        .baseline
        .accepted_composition
        .new_method_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for key in &new_ids {
        if old_ids.contains(key) {
            findings.push(json!({"code": "NEW_KEY_COLLIDES_WITH_OLD", "method_id": key}));
        }
    }
    let pinned = &manifest
        .baseline
        .accepted_composition
        .old_qualified_keys_sha256;
    if !pinned.is_empty() && pinned != &old_hash {
        findings.push(json!({
            "code": "OLD_QUALIFIED_KEYS_HASH_MISMATCH",
            "expected": pinned,
            "actual": old_hash
        }));
    }
    let extra_roots: &[&str] = successor
        .as_ref()
        .map(multilang::Successor::extra_survey_roots)
        .unwrap_or(&[]);
    let extra_expected = successor
        .as_ref()
        .map(multilang::Successor::expected_paths)
        .unwrap_or_default();
    let mut survey = survey_roots_with_extra(&roots, extra_roots)?;
    compare_survey_with_extra(
        &snapshot_rows,
        &manifest,
        &mut survey,
        extra_roots,
        &extra_expected,
    );
    if !survey.errors.is_empty() {
        findings.extend(survey.errors.clone());
    }
    let ok = findings.is_empty();
    let exit = if ok { 0 } else { 1 };
    let mut verification_scope = vec![
        "snapshot artifacts.jsonl hash and snapshot_id recomputation",
        "graph summary and non-methods payload hashes; methods payload full-key join is performed once by source-inventory",
        "34 non-managed origin artifacts",
        "8 curated source destinations",
        "deterministic C# view rebuilt from selected input ranges",
        "proof anchors: full-file hashes, selected range hashes, required tokens, accepted review rows",
        "41456-row family partition",
        "current accepted 104-file/533-key authority count and key hash",
    ];
    if successor.is_some() {
        verification_scope.push("successor 34-origin/26-physical/131-file validation");
    }
    verification_scope.push("metadata-only current presence survey");
    let value = json!({
        "command": "source-verify",
        "ok": ok,
        "exit": exit,
        "status": if ok { "BYTE_AND_PROOF_VERIFIED" } else { "FAIL" },
        "trust_status": trust_status(),
        "multilang_trust_status": multilang_trust_status(),
        "production_manifest_pin": PRODUCTION_MANIFEST_SHA256,
        "production_multilang_manifest_pin": PRODUCTION_MULTILANG_MANIFEST_SHA256,
        "multilang": successor.as_ref().map(multilang::Successor::summary_value).unwrap_or(Value::Null),
        "verification_scope": verification_scope,
        "semantic_status": "NOT_CHECKED",
        "graph_semantics": "NOT_VERIFIED",
        "manifest": {
            "path": path.to_string_lossy().replace('\\', "/"),
            "sha256": crate::sha256_hex(&bytes)
        },
        "counts": {
            "snapshot_rows": snapshot_rows.len(),
            "family_total": family_counts.values().sum::<u64>(),
            "non_managed_artifacts": manifest.non_managed_artifacts.len(),
            "imports": manifest.imports.len(),
            "old_qualified_keys": old_ids.len(),
            "new_qualified_keys": new_ids.len(),
            "supplemented_qualified_keys": old_ids.len() + new_ids.len(),
            "supplemented_c_files": manifest.baseline.accepted_composition.supplemented_c_file_count,
            "exact_imports": import_counts.get("exact_imports").copied().unwrap_or(0),
            "deterministic_views": import_counts.get("deterministic_views").copied().unwrap_or(0),
            "proof_anchors": import_counts.get("proof_anchors").copied().unwrap_or(0),
            "function_views": import_counts.get("function_views").copied().unwrap_or(0)
        },
        "old_qualified_keys_sha256": old_hash,
        "old_qualified_keys_pin": if pinned.is_empty() { "PENDING".to_string() } else { "PINNED".to_string() },
        "accepted_class_counts": accepted_counts,
        "supplemented_class_counts": supplemented_counts,
        "supplemented_totals": {
            "reported": true,
            "old_c_files": manifest.baseline.accepted_composition.old_c_file_count,
            "new_c_files": manifest.baseline.accepted_composition.new_c_file_count,
            "c_files": manifest.baseline.accepted_composition.supplemented_c_file_count,
            "qualified_keys": old_ids.len() + new_ids.len(),
            "v7_created": false
        },
        "family_counts": family_counts,
        "survey": survey_summary_with_successor(&survey, &manifest, successor.as_ref()),
        "findings": findings,
        "limits": manifest.limits
    });
    Ok((exit, value))
}

pub(super) fn run_inventory(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    let (exit, value) = inventory_with_trust(flags, &production_trust())?;
    print_cli(value);
    Ok(exit)
}

fn inventory_with_trust(
    flags: &BTreeMap<String, PathBuf>,
    trust: &Trust,
) -> RegistryResult<(u8, Value)> {
    let path = manifest_path(flags);
    let (manifest, bytes) = load_manifest_trusted(&path, trust)?;
    let roots = roots_from_flags(flags, true)?;
    let successor = if trust.successor_required {
        Some(multilang::load_candidate(
            &roots, &manifest, &bytes, trust, true,
        )?)
    } else {
        None
    };
    let snapshot_rows = load_snapshot_verified(&roots, &manifest)?;
    let _summary = load_graph_summary(&roots, &manifest, false)?;
    let family_counts = family_partition(&manifest, &snapshot_rows)?;
    let (set_findings, expected_non_managed) =
        check_expected_non_managed(&manifest, &snapshot_rows);
    let (mut findings, _) = verify_non_managed(&roots, &manifest);
    findings.extend(set_findings.clone());
    let (import_findings, import_counts) = verify_imports(&roots, &manifest);
    findings.extend(import_findings);
    let (old_ids, old_hash) = old_identities(&roots, &manifest)?;
    let (accepted_class_by_id, accepted_counts) = accepted_classes(&roots, &manifest)?;
    let supplemented_counts = supplemented_class_counts(&accepted_counts);
    let new_ids = manifest
        .baseline
        .accepted_composition
        .new_method_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for key in &new_ids {
        if old_ids.contains(key) {
            findings.push(json!({"code": "NEW_KEY_COLLIDES_WITH_OLD", "method_id": key}));
        }
    }
    let join = stream_graph_methods(&roots, &manifest, &old_ids, &accepted_class_by_id, &new_ids)?;
    let extra_roots: &[&str] = successor
        .as_ref()
        .map(multilang::Successor::extra_survey_roots)
        .unwrap_or(&[]);
    let extra_expected = successor
        .as_ref()
        .map(multilang::Successor::expected_paths)
        .unwrap_or_default();
    let mut survey = survey_roots_with_extra(&roots, extra_roots)?;
    compare_survey_with_extra(
        &snapshot_rows,
        &manifest,
        &mut survey,
        extra_roots,
        &extra_expected,
    );
    if !survey.errors.is_empty() {
        findings.extend(survey.errors.clone());
    }
    let exclusive = join.axis_counts.clone();
    let exclusive_sum = exclusive.values().sum::<u64>();
    if exclusive_sum != join.rows {
        findings.push(json!({
            "code": "GRAPH_AXIS_SUM_MISMATCH",
            "exclusive_sum": exclusive_sum,
            "rows": join.rows
        }));
    }
    if join.rows
        != manifest
            .baseline
            .graph
            .counts
            .get("known_methods_all5_incl_abstract")
            .copied()
            .unwrap_or(0)
    {
        findings.push(json!({
            "code": "GRAPH_ROW_COUNT_MISMATCH",
            "actual": join.rows,
            "expected": manifest.baseline.graph.counts.get("known_methods_all5_incl_abstract")
        }));
    }
    if join.accepted_current_rows != old_ids.len() as u64
        || join.new_scoped_rows != new_ids.len() as u64
    {
        findings.push(json!({
            "code": "GRAPH_PRIMARY_PRECEDENCE_MISMATCH",
            "accepted_rows": join.accepted_current_rows,
            "expected_accepted": old_ids.len(),
            "new_rows": join.new_scoped_rows,
            "expected_new": new_ids.len()
        }));
    }
    if supplemented_counts
        != manifest
            .baseline
            .accepted_composition
            .supplemented_class_counts
    {
        findings.push(json!({
            "code": "SUPPLEMENTED_CLASS_COUNTS_MISMATCH",
            "expected": manifest.baseline.accepted_composition.supplemented_class_counts,
            "actual": supplemented_counts,
        }));
    }
    if !join.invalid.is_empty() {
        findings.push(json!({
            "code": "GRAPH_METHOD_ID_INVALID",
            "count": join.invalid.len(),
            "sample": join.invalid.iter().take(20).cloned().collect::<Vec<_>>()
        }));
    }
    if !join.duplicates.is_empty() {
        findings.push(json!({
            "code": "GRAPH_METHOD_ID_DUPLICATE",
            "count": join.duplicates.len(),
            "sample": join.duplicates.iter().take(20).cloned().collect::<Vec<_>>()
        }));
    }
    let pinned = &manifest
        .baseline
        .accepted_composition
        .old_qualified_keys_sha256;
    if !pinned.is_empty() && pinned != &old_hash {
        findings.push(json!({
            "code": "OLD_QUALIFIED_KEYS_HASH_MISMATCH",
            "expected": pinned,
            "actual": old_hash
        }));
    }
    let outside = join
        .rows
        .saturating_sub(join.old_found)
        .saturating_sub(join.new_found.len() as u64);
    let ok = findings.is_empty();
    let exit = if ok { 0 } else { 1 };
    let family_rows = survey_family_rows(&snapshot_rows, &manifest, &survey);
    let family_accounting = json!({
        "snapshot_total": snapshot_rows.len(),
        "family_total": family_counts.values().sum::<u64>(),
        "zero_loss": family_counts.values().sum::<u64>() == snapshot_rows.len() as u64,
        "families": family_rows,
    });
    let current_survey = survey_summary_with_successor(&survey, &manifest, successor.as_ref());
    let baseline_value = json!({
        "snapshot_rows": snapshot_rows.len(),
        "snapshot_id": manifest.baseline.snapshot.snapshot_id.clone(),
        "graph_id": manifest.baseline.graph.graph_id.clone(),
        "known_methods": manifest.baseline.graph.counts.get("known_methods_all5_incl_abstract").copied().unwrap_or(0),
    });
    let non_managed_value = json!({
        "manifest_rows": manifest.non_managed_artifacts.len(),
        "snapshot_expected_rows": expected_non_managed.len(),
        "exact_set": set_findings.is_empty(),
    });
    let imports_value = json!({
        "files": manifest.imports.len(),
        "exact_imports": import_counts.get("exact_imports").copied().unwrap_or(0),
        "deterministic_views": import_counts.get("deterministic_views").copied().unwrap_or(0),
        "proof_anchors": import_counts.get("proof_anchors").copied().unwrap_or(0),
        "function_views": import_counts.get("function_views").copied().unwrap_or(0),
    });
    let graph_identity = json!({
        "graph_rows": join.rows,
        "unique_method_ids": join.unique,
        "duplicate_method_ids": join.duplicates.len(),
        "invalid_method_ids": join.invalid.len(),
        "representation_kinds": join.representation_kinds.clone(),
        "evidence_counts": join.evidence.clone(),
        "coverage_counts": join.coverage.clone(),
        "exclusive_axis_counts": exclusive.clone(),
        "exclusive_axis_sum": exclusive_sum,
        "denominator_graph_rows": join.rows,
        "old_qualified_keys": old_ids.len(),
        "old_qualified_keys_sha256": old_hash.clone(),
        "old_keys_found": join.old_found,
        "old_keys_absent": join.old_absent.len(),
        "old_keys_absent_sample": join.old_absent.iter().take(100).cloned().collect::<Vec<_>>(),
        "old_absent_truncated": join.old_absent.len() > 100,
        "accepted_class_counts": accepted_counts.clone(),
        "supplemented_class_counts": supplemented_counts.clone(),
    });
    let graph_counts = json!({
        "new_qualified_keys": new_ids.len(),
        "new_keys_found_in_graph": join.new_found.len(),
        "new_keys_found_sample": join.new_found.iter().take(20).cloned().collect::<Vec<_>>(),
        "graph_outside_supplemented_primary": outside,
        "accepted_current_rows": join.accepted_current_rows,
        "new_scoped_rows": join.new_scoped_rows,
        "body_sha_rows": join.body_sha_rows,
        "body_hash_verified_true": join.body_hash_verified_true,
        "body_hash_verified_false": join.body_hash_verified_false,
        "raw_body_hash_only_rows": join.raw_body_hash_only,
        "canonical_rows": join.canonical_rows,
        "p1_rows": join.p1_rows,
        "direct_rows": join.direct_rows,
        "p1_body_rows": join.p1_body_rows,
        "source_association_rows": join.source_association_rows,
        "raw_associated_rows": join.raw_associated_rows,
    });
    let graph_labels = json!({
        "body_present_outside_supplement_exact": join.body_present_outside_primary_exact,
        "candidate_raw_associated_rows": join.candidate_raw_associated,
        "candidate_raw_associated_definition": "raw/body/source association candidate; not a recovered source-body count",
        "graph_axis_precedence": "current accepted 533 and new 4 override graph declarations; remaining rows use P1 body, declared CIL, canonical metadata association, source-only, raw-associated, or abstract/unresolved",
        "graph_declared_labels_are_not_semantic_pass": true,
        "abstract_metadata_authority": "current accepted class from the current manifest/fullkey; graph inference is not authoritative",
        "outside_primary_body_present_keyed_by_full_method_id": true,
        "semantic_status": "NOT_VERIFIED",
        "method_id_validation": "FULL_KEY_RECOMPUTED_PER_ROW",
        "payload_hash_validated_during_read": true,
    });
    let source_graph_join = merge_json(merge_json(graph_identity, graph_counts), graph_labels);
    let supplemented_totals = json!({
        "old_c_files": manifest.baseline.accepted_composition.old_c_file_count,
        "old_qualified_keys": old_ids.len(),
        "new_c_files": manifest.baseline.accepted_composition.new_c_file_count,
        "new_qualified_keys": new_ids.len(),
        "c_files": manifest.baseline.accepted_composition.supplemented_c_file_count,
        "qualified_keys": old_ids.len() + new_ids.len(),
        "v7_created": false,
    });
    let mut top = json!({
        "command": "source-inventory",
        "ok": ok,
        "exit": exit,
        "status": if ok { "CENSUS_OK" } else { "FAIL" },
        "trust_status": trust_status(),
        "multilang_trust_status": multilang_trust_status(),
        "production_manifest_pin": PRODUCTION_MANIFEST_SHA256,
        "production_multilang_manifest_pin": PRODUCTION_MULTILANG_MANIFEST_SHA256,
        "multilang": successor.as_ref().map(multilang::Successor::summary_value).unwrap_or(Value::Null),
        "coverage_role": "REPORT_NOT_SELF_PROVING_AUTHORITY",
        "manifest": {
            "path": path.to_string_lossy().replace('\\', "/"),
            "sha256": crate::sha256_hex(&bytes),
        },
        "baseline": baseline_value,
        "non_managed": non_managed_value,
        "imports": imports_value,
        "family_accounting": family_accounting,
        "current_survey": current_survey,
        "source_graph_join": source_graph_join,
        "accepted_class_counts": accepted_counts.clone(),
        "supplemented_class_counts": supplemented_counts.clone(),
        "supplemented_totals": supplemented_totals,
        "findings": findings,
        "limits": manifest.limits.clone(),
    });
    if let Value::Object(ref mut object) = top {
        object.insert(
            "coverage_schema".to_string(),
            json!("wuwa-reconstruction-source-closure-coverage-v1"),
        );
    }
    Ok((exit, top))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start: u64, end: u64) -> LineRange {
        LineRange {
            start_line: start,
            end_line: end,
        }
    }

    fn actual_paths() -> (PathBuf, PathBuf, PathBuf) {
        let cwd = std::env::current_dir().unwrap();
        let curated = if cwd.join("metadata/source-closure.v1.json").exists() {
            cwd.clone()
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
        let research = curated
            .parent()
            .filter(|parent| parent.join("src/reconstructed").exists())
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| curated.clone());
        let manifest = curated.join("metadata/source-closure.v1.json");
        (curated, research, manifest)
    }

    fn actual_roots() -> Roots {
        let (curated, research, _) = actual_paths();
        Roots { research, curated }
    }

    #[test]
    fn default_manifest_follows_selected_curated_root_but_explicit_manifest_keeps_cwd_semantics() {
        let (curated, _, _) = actual_paths();
        let mut flags = BTreeMap::new();
        flags.insert("--curated-root".to_string(), curated.clone());
        assert_eq!(manifest_path(&flags), curated.join(MANIFEST_REL));

        let current = std::env::current_dir().expect("process current dir");
        flags.insert(
            "--manifest".to_string(),
            PathBuf::from("metadata/explicit-relative.json"),
        );
        assert_eq!(
            manifest_path(&flags),
            current.join("metadata/explicit-relative.json")
        );

        let absolute = curated.join("metadata/explicit-absolute.json");
        flags.insert("--manifest".to_string(), absolute.clone());
        assert_eq!(manifest_path(&flags), absolute);
    }

    #[test]
    fn missing_selected_curated_root_is_strict_and_not_replaced_by_cwd() {
        let missing = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("test-fixtures")
            .join(format!(
                "source-closure-missing-root-{}",
                std::process::id()
            ));
        assert!(
            !missing.exists(),
            "fixture root unexpectedly exists: {}",
            missing.display()
        );
        let mut flags = BTreeMap::new();
        flags.insert("--curated-root".to_string(), missing.clone());
        let error = match roots_from_flags(&flags, false) {
            Ok(_) => panic!("missing curated root must fail"),
            Err(error) => error,
        };
        assert_eq!(error.code, "CURATED_ROOT_INVALID");
        assert_eq!(manifest_path(&flags), missing.join(MANIFEST_REL));
    }

    impl SurveyReport {
        fn clone_for_test(&self) -> SurveyReport {
            SurveyReport {
                research_current: self.research_current.clone(),
                curated_current: self.curated_current.clone(),
                missing: self.missing.clone(),
                changed: self.changed.clone(),
                new_research: self.new_research.clone(),
                new_curated: self.new_curated.clone(),
                errors: Vec::new(),
            }
        }
    }

    fn minimal_manifest_value() -> Value {
        let source =
            json!({"root_id":"RESEARCH_ROOT","path":"src/a.ts","size":3,"sha256":"a".repeat(64)});
        let dest = json!({"root_id":"CURATED_ROOT","path":"src/out/a.ts","size":3,"sha256":"a".repeat(64)});
        json!({
            "schema": MANIFEST_SCHEMA,
            "manifest_version": 2,
            "campaign": "RE-RECOVERED-SOURCE-CLOSURE01",
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
            "non_managed_artifacts":[{
                "origin_id":"SRC-TS-A","layer":"TS","origin_class":"RECONSTRUCTED",
                "artifact":source,"kind":"source","disposition":"DECLARATION_ONLY",
                "representation_type":"T","primary_class":"P","scope":"S","implementation_scope":"I",
                "evidence_status":"E","authority":{"kind":"K","authority_id":null,"binary_sha256":null,"mvid":null,"type_def":null,"rva":null,"export":null},
                "dependencies":[],"uncertainties":[],"proof_anchors":[],"accepted_evidence":[],
                "alternative_ids":[],"public_distribution":"INTERNAL_ONLY_NOT_REVIEWED",
                "identity_scheme":"I","overlap":"O","import_id":null,"managed_methoddefs":[]
            }],
            "imports":[{
                "id":"IMP-01","origin_id":"SRC-TS-A","layer":"TS","mode":"EXACT_BYTE_COPY",
                "source":source,"destination":dest,"transform":null,"identity_scheme":"I",
                "managed_methoddefs":[],"member_selectors":[],"dependencies":[],"scope":"S",
                "evidence_status":"IMPORTED_EXACT_BYTE","accepted_evidence":[],"proof_anchors":[]
            }],
            "backlog":[],
            "family_rules":[{
                "family_id":"F","origin_class":"O","disposition_class":"D","match_kind":"OTHERWISE",
                "path_rule":"","extensions":[],"snapshot_kind":null,"snapshot_count":41456,
                "evidence_scope":"E","notes":"N"
            }],
            "limits":[]
        })
    }

    fn manifest_value() -> SourceManifest {
        serde_json::from_value(minimal_manifest_value()).expect("fixture manifest")
    }

    #[test]
    #[ignore = "bounded actual-profile candidate receipt; run explicitly after manifest recipe changes"]
    fn actual_candidate_profile_emit() {
        let (curated, research, manifest) = actual_paths();
        let bytes = fs::read(&manifest).unwrap();
        let trust = fixture_trust(crate::sha256_hex(&bytes));
        let mut flags = BTreeMap::new();
        flags.insert("--manifest".to_string(), manifest);
        flags.insert("--curated-root".to_string(), curated.clone());
        flags.insert("--research-root".to_string(), research);
        let (status_exit, status) = status_with_trust(&flags, &trust).unwrap();
        assert_eq!(status_exit, 0);
        let (verify_exit, verify) = verify_with_trust(&flags, &trust).unwrap();
        assert_eq!(
            verify_exit,
            0,
            "{}",
            serde_json::to_string_pretty(&verify).unwrap()
        );
        let (inventory_exit, inventory) = inventory_with_trust(&flags, &trust).unwrap();
        assert_eq!(
            inventory_exit,
            0,
            "{}",
            serde_json::to_string_pretty(&inventory).unwrap()
        );
        let out = curated
            .join("tools/re/local-recovery/recovered-source-closure01/reports/d01-candidate");
        fs::create_dir_all(&out).unwrap();
        fs::write(
            out.join("source-status.candidate.json"),
            serde_json::to_string_pretty(&status).unwrap() + "\n",
        )
        .unwrap();
        fs::write(
            out.join("source-verify.candidate.json"),
            serde_json::to_string_pretty(&verify).unwrap() + "\n",
        )
        .unwrap();
        fs::write(
            out.join("source-inventory.candidate.json"),
            serde_json::to_string_pretty(&inventory).unwrap() + "\n",
        )
        .unwrap();
    }

    #[test]
    fn method_id_matches_full_key_formula() {
        let first = method_id("E3FC", "MVID", "0x020048E7", "0x06030B3D");
        let second = method_id("e3fc", "mvid", "0x020048e7", "0x06030b3d");
        assert_eq!(first, second);
        assert_ne!(first, method_id("e3fc", "mvid", "0x020048E7", "0x06030B84"));
    }

    #[test]
    fn line_span_rejects_neighbor_overflow_and_handles_last_line() {
        let bytes = b"a\nb\nc";
        assert_eq!(line_span(bytes, &range(2, 2)).unwrap(), b"b\n");
        assert_eq!(line_span(bytes, &range(3, 3)).unwrap(), b"c");
        assert!(line_span(bytes, &range(4, 4)).is_err());
        assert!(line_span(bytes, &range(0, 1)).is_err());
    }

    #[test]
    fn transform_rejects_partial_overlap_and_changed_chunk() {
        let base = Transform {
            input: ArtifactRef {
                root_id: "RESEARCH_ROOT".to_string(),
                path: "src/a.cs".to_string(),
                size: 1,
                sha256: "a".repeat(64),
            },
            banner: "// banner\n".to_string(),
            ranges: vec![range(1, 2)],
            omitted_ranges: vec![range(2, 3)],
            chunks: vec![ChunkProof {
                input_start_line: 1,
                input_end_line: 2,
                output_start_line: 2,
                output_end_line: 3,
                sha256: "b".repeat(64),
            }],
            added_banner_sha256: crate::sha256_hex(b"// banner\n"),
        };
        assert!(validate_transform(&base).is_err());
        let mut changed = base.clone();
        changed.omitted_ranges = vec![range(3, 4)];
        changed.chunks[0].sha256 = "0".repeat(64);
        assert!(validate_transform(&changed).is_ok());
        changed.added_banner_sha256 = "0".repeat(64);
        assert!(validate_transform(&changed).is_err());
    }

    #[test]
    fn declaration_and_reference_misroutes_are_rejected() {
        assert!(route_check("src/reconstructed/config/A.ts", &Disposition::Curated).is_err());
        assert!(route_check("src/reconstructed/native/A.hpp", &Disposition::Curated).is_err());
        assert!(
            route_check(
                "src/reconstructed/loading/LoadingPipeline.ts",
                &Disposition::Curated
            )
            .is_err()
        );
        assert!(
            route_check(
                "evidence/performance/check_perf03b_semantics.js",
                &Disposition::Curated
            )
            .is_err()
        );
        assert!(
            route_check(
                "src/reconstructed/performance/native/A.cpp",
                &Disposition::Curated
            )
            .is_ok()
        );
    }

    #[test]
    fn duplicate_origin_and_conflict_path_are_rejected() {
        let mut value = minimal_manifest_value();
        let duplicate = value["non_managed_artifacts"][0].clone();
        value["non_managed_artifacts"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        let manifest: SourceManifest = serde_json::from_value(value).unwrap();
        assert!(validate_manifest_structure(&manifest).is_err());

        let mut value = minimal_manifest_value();
        let mut second = value["imports"][0].clone();
        second["id"] = json!("IMP-02");
        value["imports"].as_array_mut().unwrap().push(second);
        let manifest: SourceManifest = serde_json::from_value(value).unwrap();
        assert!(validate_manifest_structure(&manifest).is_err());
    }

    #[test]
    fn exact_copy_drift_is_rejected() {
        let mut value = minimal_manifest_value();
        value["imports"][0]["destination"]["sha256"] = json!("b".repeat(64));
        let manifest: SourceManifest = serde_json::from_value(value).unwrap();
        assert!(validate_manifest_structure(&manifest).is_err());
    }

    #[test]
    fn native_methoddef_claim_is_rejected() {
        let mut value = minimal_manifest_value();
        value["imports"][0]["managed_methoddefs"] = json!(["0x06000001"]);
        let manifest: SourceManifest = serde_json::from_value(value).unwrap();
        assert!(validate_manifest_structure(&manifest).is_err());
    }

    #[test]
    fn unbounded_or_traversal_hash_paths_are_rejected() {
        let mut value = minimal_manifest_value();
        value["non_managed_artifacts"][0]["artifact"]["sha256"] = json!("*".repeat(64));
        let manifest: SourceManifest = serde_json::from_value(value).unwrap();
        assert!(validate_manifest_structure(&manifest).is_err());

        let mut value = minimal_manifest_value();
        value["non_managed_artifacts"][0]["artifact"]["path"] = json!("../escape.ts");
        let manifest: SourceManifest = serde_json::from_value(value).unwrap();
        assert!(validate_manifest_structure(&manifest).is_err());
    }

    #[test]
    fn structural_status_never_claims_semantic_pass() {
        let manifest = manifest_value();
        let value = structural_status_value(&manifest, b"{}", Path::new(MANIFEST_REL));
        assert_eq!(value["verification"], "NOT_CHECKED");
        assert_eq!(value["semantic_status"], "NOT_CHECKED");
        assert_eq!(value["status"], "STRUCTURAL_OK");
    }

    fn member_proof(method_token: &str) -> MemberProof {
        let (capture_id, rva, size) = match method_token {
            "0x06030B3D" => ("PERF11_INIT_EMIT_EVENT_DATA", "0xC02C04", 178_u64),
            "0x06030B84" => ("PERF11_EMIT_MY_PLAYER_OVERLAP_EVENT", "0xC06D34", 445_u64),
            "0x06030B87" => ("PERF11_EMIT_PENDING_EVENTS", "0xC07344", 574_u64),
            _ => ("PERF11_EMIT_MY_PLAYER_OVERLAP_CALLBACK", "0xC091B3", 7_u64),
        };
        MemberProof {
            capture_id: capture_id.to_string(),
            capture_path: "evidence/performance/PERF11_DIRECT_CIL.txt".to_string(),
            capture_file_size: 1,
            capture_file_sha256: "a".repeat(64),
            capture_start_line: 1,
            capture_end_line: 2,
            capture_section_sha256: "b".repeat(64),
            capture_methoddef: method_token.to_string(),
            capture_rva: rva.to_string(),
            capture_code_size: size,
            evidence_path: "evidence/performance/PERF11_EVIDENCE.json".to_string(),
            evidence_file_size: 1,
            evidence_file_sha256: "c".repeat(64),
            evidence_selector: "selector".to_string(),
            evidence_methoddef: method_token.to_string(),
            authority_sha256: REQUIRED_MANAGED_AUTHORITY.to_string(),
            mvid: REQUIRED_MANAGED_MVID.to_string(),
            type_def: REQUIRED_MANAGED_TYPE_DEF.to_string(),
            required_il_tokens: vec!["methoddef".to_string()],
        }
    }

    fn member(type_token: &str, method_token: &str, id: String) -> MemberSelector {
        MemberSelector {
            method_id: id,
            role: "ORDINARY".to_string(),
            type_token: type_token.to_string(),
            method_token: method_token.to_string(),
            representation: "TEST".to_string(),
            source_input_lines: Some(range(1, 1)),
            output_lines: Some(range(1, 1)),
            input_range_sha256: "d".repeat(64),
            output_range_sha256: "e".repeat(64),
            scope: "TEST".to_string(),
            proof: member_proof(method_token),
        }
    }

    #[test]
    fn unrelated_methoddef_and_wrong_typedef_are_rejected() {
        let authority = REQUIRED_MANAGED_AUTHORITY;
        let mvid = REQUIRED_MANAGED_MVID;
        let type_def = REQUIRED_MANAGED_TYPE_DEF;
        let method_def = "0x06030B3D";
        let good = member(
            type_def,
            method_def,
            method_id(authority, mvid, type_def, method_def),
        );
        assert!(validate_member_identity(authority, mvid, type_def, &good).is_ok());
        let mut wrong_type = good.clone();
        wrong_type.type_token = "0x02000002".to_string();
        assert!(validate_member_identity(authority, mvid, type_def, &wrong_type).is_err());
        let mut wrong_method = good.clone();
        wrong_method.method_token = "0x06000002".to_string();
        assert!(validate_member_identity(authority, mvid, type_def, &wrong_method).is_err());
        let mut wrong_mvid = good.clone();
        wrong_mvid.method_id = method_id(authority, "other-mvid", type_def, method_def);
        assert!(validate_member_identity(authority, mvid, type_def, &wrong_mvid).is_err());
    }

    #[test]
    fn successor_fixture_seam_preserves_predecessor_and_reports_131() {
        let (curated, research, manifest_path) = actual_paths();
        let parent_bytes = fs::read(&manifest_path).unwrap();
        let successor_path = curated.join(MULTILANG_MANIFEST_REL);
        let successor_bytes = fs::read(&successor_path).unwrap();
        assert_eq!(
            crate::sha256_hex(&parent_bytes),
            "42f70e404a13bfbe5545ba81b946630d60a5ec15f2218d004abedff6a95e1b1d"
        );
        assert_eq!(
            crate::sha256_hex(&successor_bytes),
            "3e6a4ae50c0443b5ed20156cc0e7e4cb317bb13bcaf8328bb96e23927d6bbf8b"
        );
        let trust = fixture_trust_with_multilang(
            crate::sha256_hex(&parent_bytes),
            crate::sha256_hex(&successor_bytes),
        );
        let roots = Roots {
            research: research.clone(),
            curated: curated.clone(),
        };
        let (parent, bytes) = load_manifest_trusted(&manifest_path, &trust).unwrap();
        let successor = multilang::load_candidate(&roots, &parent, &bytes, &trust, true).unwrap();
        assert_eq!(
            successor.summary_value()["summary"]["expected_curated_files"],
            131
        );
        assert_eq!(successor.summary_value()["summary"]["new_imports"], 19);

        let mut flags = BTreeMap::new();
        flags.insert("--manifest".to_string(), manifest_path);
        flags.insert("--curated-root".to_string(), curated);
        flags.insert("--research-root".to_string(), research);
        let (exit, value) = verify_with_trust(&flags, &trust).unwrap();
        assert_eq!(exit, 0, "{}", serde_json::to_string_pretty(&value).unwrap());
        let (status_exit, status) = status_with_trust(&flags, &trust).unwrap();
        assert_eq!(status_exit, 0);
        assert_eq!(status["status"], "STRUCTURAL_OK");
        assert_eq!(status["multilang"]["summary"]["new_imports"], 19);
        assert_eq!(value["multilang"]["summary"]["physical_files"], 26);
        assert_eq!(value["survey"]["expected_curated_total"], 131);
        assert_eq!(value["survey"]["new_curated_present"], 19);
        assert_eq!(value["survey"]["unexpected_new_curated_count"], 0);
    }

    #[test]
    fn trusted_seam_rejects_wrong_manifest_or_baseline() {
        let (_, _, manifest_path) = actual_paths();
        let bytes = fs::read(&manifest_path).unwrap();
        let wrong = fixture_trust("0".repeat(64));
        assert!(load_manifest_trusted(&manifest_path, &wrong).is_err());
        let correct = fixture_trust(crate::sha256_hex(&bytes));
        let wrong_graph = Trust {
            manifest_sha256: correct.manifest_sha256.clone(),
            multilang_manifest_sha256: None,
            successor_required: false,
            snapshot_id: ACCEPTED_SNAPSHOT_ID,
            graph_id: "0000000000000000000000000000000000000000000000000000000000000000",
            payloads: &ACCEPTED_GRAPH_PAYLOADS,
        };
        assert!(load_manifest_trusted(&manifest_path, &wrong_graph).is_err());
        assert!(load_manifest_trusted(&manifest_path, &correct).is_ok());
    }

    #[test]
    fn census_fails_missing_changed_and_unexpected_files() {
        let manifest = manifest_value();
        let row = SnapshotRow {
            artifact_id: "a".repeat(64),
            root_id: "RESEARCH_ROOT".to_string(),
            path: "src/reconstructed/missing.ts".to_string(),
            size: Some(1),
            sha256: Some("b".repeat(64)),
            status: "OK".to_string(),
            kind: "source".to_string(),
            backup_coverage: "UNKNOWN".to_string(),
            error_code: None,
        };
        let historical = SnapshotRow {
            artifact_id: "c".repeat(64),
            root_id: "RESEARCH_ROOT".to_string(),
            path: "_production/e01/historical.cs".to_string(),
            size: Some(1),
            sha256: Some("d".repeat(64)),
            status: "OK".to_string(),
            kind: "source".to_string(),
            backup_coverage: "UNKNOWN".to_string(),
            error_code: None,
        };
        let mut survey = SurveyReport {
            research_current: BTreeMap::new(),
            curated_current: BTreeMap::new(),
            missing: Vec::new(),
            changed: Vec::new(),
            new_research: Vec::new(),
            new_curated: Vec::new(),
            errors: Vec::new(),
        };
        compare_survey(&[row.clone(), historical.clone()], &manifest, &mut survey);
        assert!(
            survey
                .errors
                .iter()
                .any(|error| error["code"] == "REQUIRED_SOURCE_MISSING")
        );
        assert_eq!(survey.missing.len(), 2);

        let mut changed = survey.clone_for_test();
        changed.research_current.insert(row.path.clone(), 2);
        compare_survey(&[row.clone(), historical.clone()], &manifest, &mut changed);
        assert!(
            changed
                .errors
                .iter()
                .any(|error| error["code"] == "REQUIRED_SOURCE_SIZE_CHANGED")
        );

        let mut unexpected = survey.clone_for_test();
        unexpected
            .research_current
            .insert("src/reconstructed/new.ts".to_string(), 1);
        compare_survey(&[row, historical], &manifest, &mut unexpected);
        assert!(
            unexpected
                .errors
                .iter()
                .any(|error| error["code"] == "NEW_RESEARCH_SOURCE_FILES")
        );

        let mut curated = survey.clone_for_test();
        curated
            .curated_current
            .insert("src/managed/cs-e3fc67839851/unexpected.cs".to_string(), 1);
        compare_survey(&[], &manifest, &mut curated);
        assert!(
            curated
                .errors
                .iter()
                .any(|error| error["code"] == "NEW_CURATED_FILE_SET_MISMATCH")
        );
    }

    #[test]
    fn proof_tokens_must_bind_within_the_verified_range() {
        let root = std::env::current_dir()
            .unwrap()
            .join("target/source-closure-proof-test");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("proof.txt"), b"line-one\nline-two\n").unwrap();
        let roots = Roots {
            research: root.clone(),
            curated: std::env::current_dir().unwrap(),
        };
        let bytes = fs::read(root.join("proof.txt")).unwrap();
        let first = b"line-one\n";
        let anchor = ProofAnchor {
            root_id: "RESEARCH_ROOT".to_string(),
            path: "proof.txt".to_string(),
            size: bytes.len() as u64,
            sha256: crate::sha256_hex(&bytes),
            locator: None,
            range_start: Some(1),
            range_end: Some(1),
            range_sha256: Some(crate::sha256_hex(first)),
            contains: vec!["line-two".to_string()],
            record_id: None,
            record_disposition: None,
            evidence_class: "TEST".to_string(),
        };
        assert!(check_anchor(&roots, &anchor).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn member_and_function_proof_bind_the_target_selector() {
        let roots = actual_roots();
        let (_, _, manifest_path) = actual_paths();
        let (manifest, _) = load_manifest(&manifest_path).unwrap();
        let cs = manifest
            .imports
            .iter()
            .find(|import| import.id == "IMP-08-RANGE-PENDING-DRAIN")
            .unwrap();
        let member = &cs.member_selectors[0];
        assert!(check_member_capture(&roots, member).is_ok());
        let mut bad = member.clone();
        bad.proof.capture_section_sha256 = "0".repeat(64);
        assert!(check_member_capture(&roots, &bad).is_err());

        let data_layer = manifest
            .imports
            .iter()
            .find(|import| import.id == "IMP-01-DATALAYER-TS")
            .unwrap();
        let view = &data_layer.function_views[0];
        assert!(check_function_view(&roots, data_layer, view).is_ok());
        let mut neighbor = view.clone();
        neighbor.header_token = "static async loadDataLayers(".to_string();
        assert!(check_function_view(&roots, data_layer, &neighbor).is_err());
        assert!(find_function_view(&manifest, &view.function_id).is_some());
        assert!(find_function_view(&manifest, "FUNC-0000").is_none());
    }

    #[test]
    fn accepted_classes_are_the_preserved_246_280_6_1() {
        let roots = actual_roots();
        let (_, _, manifest_path) = actual_paths();
        let (manifest, _) = load_manifest(&manifest_path).unwrap();
        let (by_id, counts) = accepted_classes(&roots, &manifest).unwrap();
        assert_eq!(by_id.len(), 533);
        assert_eq!(counts.get("DIRECT_CIL_FULL"), Some(&246));
        assert_eq!(counts.get("SOURCE_ONLY"), Some(&280));
        assert_eq!(counts.get("UNKNOWN"), Some(&6));
        assert_eq!(counts.get("METADATA_ONLY"), Some(&1));
        let supplemented = supplemented_class_counts(&counts);
        assert_eq!(supplemented.get("DIRECT_CIL_FULL"), Some(&250));
        assert_eq!(supplemented.get("SOURCE_ONLY"), Some(&280));
        assert_eq!(supplemented.get("UNKNOWN"), Some(&6));
        assert_eq!(supplemented.get("METADATA_ONLY"), Some(&1));
    }

    #[test]
    fn unknown_family_row_is_rejected() {
        let mut manifest = manifest_value();
        manifest.family_rules = vec![FamilyRule {
            family_id: "ONLY".to_string(),
            origin_class: "O".to_string(),
            disposition_class: "D".to_string(),
            match_kind: "PREFIX".to_string(),
            path_rule: "does/not/match/".to_string(),
            extensions: Vec::new(),
            snapshot_kind: None,
            snapshot_count: 41456,
            evidence_scope: "E".to_string(),
            notes: "N".to_string(),
        }];
        let row = SnapshotRow {
            artifact_id: "a".repeat(64),
            root_id: "RESEARCH_ROOT".to_string(),
            path: "other/a.cs".to_string(),
            size: Some(1),
            sha256: Some("b".repeat(64)),
            status: "OK".to_string(),
            kind: "source".to_string(),
            backup_coverage: "UNKNOWN".to_string(),
            error_code: None,
        };
        assert!(family_partition(&manifest, &[row]).is_err());
    }

    #[test]
    fn missing_history_remains_a_row_and_is_reported() {
        let manifest = manifest_value();
        let row = SnapshotRow {
            artifact_id: "a".repeat(64),
            root_id: "RESEARCH_ROOT".to_string(),
            path: "src/reconstructed/missing.ts".to_string(),
            size: Some(1),
            sha256: Some("b".repeat(64)),
            status: "OK".to_string(),
            kind: "source".to_string(),
            backup_coverage: "UNKNOWN".to_string(),
            error_code: None,
        };
        let mut survey = SurveyReport {
            research_current: BTreeMap::new(),
            curated_current: BTreeMap::new(),
            missing: Vec::new(),
            changed: Vec::new(),
            new_research: Vec::new(),
            new_curated: Vec::new(),
            errors: Vec::new(),
        };
        let rows = vec![row];
        compare_survey(&rows, &manifest, &mut survey);
        assert_eq!(rows.len(), 1);
        assert!(
            survey
                .missing
                .iter()
                .any(|item| item["path"] == "src/reconstructed/missing.ts")
        );
    }

    #[test]
    fn payload_tamper_is_detected_without_writing() {
        let path = Path::new("Cargo.toml");
        let size = fs::metadata(path).unwrap().len();
        assert!(verify_file_hash(path, size, &"0".repeat(64)).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn windows_ancestor_junction_is_rejected() {
        let base = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!("source-closure-junction-{}", std::process::id()));
        let target = base.join("actual");
        let link = base.join("link");
        fs::create_dir_all(&target).unwrap();
        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                link.to_str().unwrap(),
                target.to_str().unwrap(),
            ])
            .status();
        let mklink_ok = status.map(|status| status.success()).unwrap_or(false);
        if !mklink_ok {
            let _ = fs::remove_dir_all(&base);
        }
        assert!(mklink_ok, "mklink /J failed; junction fixture unavailable");
        let result = inspect_relative_entry(&link, Path::new("x.txt"));
        assert!(matches!(result, Err("UNSAFE_REPARSE_OR_SYMLINK")));
        let _ = fs::remove_dir(&link);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn windows_reparse_attribute_is_rejected() {
        assert!(crate::reparse_attributes_rejected(0x400));
        assert!(!crate::reparse_attributes_rejected(0));
    }

    #[test]
    fn old_key_hash_is_order_independent_and_disjoint_from_new() {
        let mut left = BTreeSet::new();
        left.insert("a\0b\0c\0d".to_string());
        let mut right = BTreeSet::new();
        right.insert("a\0b\0c\0d".to_string());
        assert_eq!(old_key_hash(&left), old_key_hash(&right));
        right.insert("x\0y\0z\0w".to_string());
        assert_ne!(old_key_hash(&left), old_key_hash(&right));
    }
}
