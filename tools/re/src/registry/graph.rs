use super::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

const GRAPH_SCHEMA: &str = "wuwa-reconstruction-corpus-graph-v1";
const GRAPH_TASK: &str = "RE-CORPUS-PRESERVE01";
const GRAPH_PHASE: &str = "B";
const AUTHORITY_PLAN_PATH: &str = "evidence/r5b2b_full_shard_plan.json";
const AUTHORITY_PRESEED_PATH: &str = "evidence/r5b2b_full_preseed_map.jsonl";
const AUTHORITY_COMPLETION_PATH: &str = "evidence/r5b2b_p1_completion_records.jsonl";
const AUTHORITY_REVIEWS_PATH: &str = "evidence/reviews.jsonl";
const AUTHORITY_PARENT_P1_MANIFEST_PATH: &str = "protocol/R5B2B_P1_POST_INTEGRATION_SHA256SUMS.txt";
const ACCEPTED_ASSEMBLY_SHA256: &str =
    "e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db";
const ACCEPTED_PLAN_SHA256: &str =
    "868ad22d4e3b5ffcddacd9d8febf54dd61284424febadaf293c3d7958ce5c66f";
const ACCEPTED_PRESEED_SHA256: &str =
    "5eae46e9a30a9ca9a310587496274871e9b368fd9a1faaa84dc56934b4b9c68a";
const ACCEPTED_COMPLETION_SHA256: &str =
    "5b4a9f8f09f5a27561653f43ad30c35005af86e1d4b1251bdc3a38b272f78661";
const ACCEPTED_PARENT_MANIFEST_SHA256: &str =
    "e79b06dada7fbafe0d3d356553519e57486009a34c938a96e030b713ec513f5a";
const PARENT_HISTORICAL_REVIEWS_SHA256: &str =
    "2cabf416d7fc349ce6a318e1cbfa231295ad5fb82d5f93755822de84ca7b4ebe";
const ACCEPTED_P1_REVIEW_ID: &str = "REV-R5B2B-P1-0001";
const ACCEPTED_FULL_REVIEW_ID: &str = "REV-R5B2B-FULL-D0-0001";
const ACCEPTED_CANONICAL_SHARDS: u64 = 165;
const ACCEPTED_P1_SHARDS: u64 = 5;
const ACCEPTED_ELIGIBLE_ROOTS: u64 = 29025;
const ACCEPTED_FULL_ROOTS: u64 = 28937;
const ACCEPTED_PRESEED_ROOTS: u64 = 88;
const ACCEPTED_CANONICAL_METHODS: u64 = 303456;
const ACCEPTED_P1_METHODS: u64 = 2399;
const ACCEPTED_ALL5_METHODS: u64 = 305855;
const ACCEPTED_KNOWN_METHODS: u64 = 305856;
const ACCEPTED_INITIAL_PARTIAL: u64 = 303579;
const ACCEPTED_OMITTED_OTHER4: u64 = 2276;
const CANONICAL_ROOT: &str = "RESEARCH_ROOT";
const CURATED_ROOT: &str = "CURATED_ROOT";
const CANONICAL_PREFIX: &str = "src/reconstructed/managed/full/CSharpScript/shards/";
const P1_PREFIX: &str = "src/reconstructed/managed/b2b_pilot/CSharpScript/shards/";
const GAMEPLAY_PREFIX: &str = "src/reconstructed/managed/semantic/gameplay/";
const PERFORMANCE_PREFIX: &str = "src/reconstructed/performance/";
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
const PAYLOAD_FILES: [&str; 5] = [
    "methods.jsonl",
    "artifacts.links.jsonl",
    "relationships.jsonl",
    "cohorts.jsonl",
    "missing.jsonl",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactRef {
    root_id: String,
    path: String,
    size: u64,
    sha256: String,
    kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRange {
    start_line: u64,
    end_line: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Representation {
    representation_id: String,
    representation_kind: String,
    evidence: Vec<String>,
    coverage: Option<String>,
    source_status: Option<String>,
    source_ref: Option<ArtifactRef>,
    raw_ref: Option<ArtifactRef>,
    semantic_ref: Option<ArtifactRef>,
    semantic_range: Option<SourceRange>,
    proof_ref: Option<ArtifactRef>,
    proof_row: Option<u64>,
    raw_row: Option<u64>,
    source_range: Option<SourceRange>,
    body_sha256: Option<String>,
    body_hash_verified: Option<bool>,
    table_heading: Option<String>,
    table_header: Option<String>,
    note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MethodRow {
    method_id: String,
    assembly_sha256: String,
    mvid: String,
    type_token: String,
    method_token: String,
    root_owners: Vec<String>,
    representations: Vec<Representation>,
    evidence_levels: BTreeMap<String, u64>,
    source_refs: Vec<ArtifactRef>,
    raw_refs: Vec<ArtifactRef>,
    cohorts: Vec<String>,
    limitations: Vec<String>,
}

impl MethodRow {
    fn new(key: &MethodKey) -> Self {
        Self {
            method_id: key.id(),
            assembly_sha256: key.assembly_sha256.clone(),
            mvid: key.mvid.clone(),
            type_token: key.type_token.clone(),
            method_token: key.method_token.clone(),
            root_owners: Vec::new(),
            representations: Vec::new(),
            evidence_levels: BTreeMap::new(),
            source_refs: Vec::new(),
            raw_refs: Vec::new(),
            cohorts: Vec::new(),
            limitations: Vec::new(),
        }
    }

    fn add_representation(&mut self, representation: Representation) -> RegistryResult<()> {
        if self
            .representations
            .iter()
            .any(|existing| existing.representation_id == representation.representation_id)
        {
            return Err(RegistryError::new(
                "GRAPH_DUPLICATE_REPRESENTATION",
                "representation_id is duplicated within a method",
            ));
        }
        for evidence in &representation.evidence {
            if !evidence.is_empty() {
                *self.evidence_levels.entry(evidence.clone()).or_insert(0) += 1;
            }
        }
        if let Some(source) = &representation.source_ref {
            push_unique_artifact(&mut self.source_refs, source);
        }
        if let Some(raw) = &representation.raw_ref {
            push_unique_artifact(&mut self.raw_refs, raw);
        }
        if let Some(semantic) = &representation.semantic_ref {
            push_unique_artifact(&mut self.source_refs, semantic);
        }
        self.representations.push(representation);
        Ok(())
    }

    fn finalize(&mut self) {
        self.representations
            .sort_by(|left, right| left.representation_id.cmp(&right.representation_id));
        self.root_owners.sort();
        self.root_owners.dedup();
        self.source_refs
            .sort_by(|left, right| artifact_key(left).cmp(&artifact_key(right)));
        self.source_refs.dedup();
        self.raw_refs
            .sort_by(|left, right| artifact_key(left).cmp(&artifact_key(right)));
        self.raw_refs.dedup();
        self.cohorts.sort();
        self.cohorts.dedup();
        self.limitations.sort();
        self.limitations.dedup();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MethodKey {
    assembly_sha256: String,
    mvid: String,
    type_token: String,
    method_token: String,
}

impl MethodKey {
    fn id(&self) -> String {
        crate::sha256_hex(
            format!(
                "{}\0{}\0{}\0{}",
                self.assembly_sha256, self.mvid, self.type_token, self.method_token
            )
            .as_bytes(),
        )
    }

    fn sort_key(&self) -> String {
        format!(
            "{}\0{}\0{}\0{}",
            self.assembly_sha256, self.mvid, self.type_token, self.method_token
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactLinkRow {
    artifact: ArtifactRef,
    roles: Vec<String>,
    method_ids: Vec<String>,
    cohorts: Vec<String>,
    notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RelationshipRow {
    relationship_id: String,
    kind: String,
    status: String,
    subjects: Vec<String>,
    details: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CohortRow {
    cohort_id: String,
    kind: String,
    file_count: u64,
    method_count: u64,
    members: Vec<String>,
    notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MissingRow {
    missing_id: String,
    kind: String,
    root_id: Option<String>,
    path: Option<String>,
    expected: Option<String>,
    reason: String,
    refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PayloadFile {
    name: String,
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OverlayEntry {
    path: String,
    baseline_sha256: Option<String>,
    current_sha256: Option<String>,
    status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphSummary {
    schema: String,
    task: String,
    phase: String,
    mode: String,
    status: String,
    not_final: bool,
    graph_id: String,
    payload_files: Vec<PayloadFile>,
    input_mode: String,
    baseline_artifacts_sha256: String,
    baseline_summary_sha256: Option<String>,
    config_sha256: String,
    source_authority: Value,
    tooling_overlay: Vec<OverlayEntry>,
    counts: Value,
    expected_vs_actual: Value,
    evidence_counts: BTreeMap<String, u64>,
    body_anchor_verification: Value,
    cohorts: Vec<String>,
    limits: Vec<String>,
    publication: String,
    self_validation: String,
}

#[derive(Debug, Serialize)]
struct GraphCounts {
    baseline_artifacts: u64,
    canonical_files: u64,
    canonical_rows: u64,
    canonical_unique: u64,
    p1_files: u64,
    p1_rows: u64,
    p1_unique: u64,
    p1_p0_unique: u64,
    p1_p0_overlap: u64,
    p1_overlap_actual: u64,
    p1_all5_new: u64,
    canonical_plus_p0_partial: u64,
    all5_union: u64,
    mapped_methods_all5_actual: u64,
    known_methods_all5_incl_abstract: u64,
    p1_scope: String,
    accepted_partition: Value,
    curated_files: u64,
    curated_methods: u64,
    curated_abstract: u64,
    curated_associations: Value,
    gameplay_rows: u64,
    gameplay_methods_matched: u64,
    gameplay_source_files: u64,
    gameplay_linked_source_artifacts: u64,
    gameplay_curated_files: u64,
    gameplay_curated_rows: u64,
    performance_files: u64,
    performance_curated: u64,
    performance_queue: u64,
    performance_tiers: BTreeMap<String, u64>,
    performance_method_links: u64,
    performance_unmapped: u64,
    qualification_queue: u64,
    duplicate_groups: u64,
    duplicate_paths: u64,
    same_symbol_groups: u64,
    r38_rows: u64,
    r54_staged: u64,
    missing_triad: u64,
    method_rows: u64,
    link_rows: u64,
    relationship_rows: u64,
    cohort_rows: u64,
    body_anchor_classes: BTreeMap<String, u64>,
    missing_rows: u64,
    non_managed: BTreeMap<String, u64>,
}

#[derive(Debug)]
struct GraphData {
    methods: BTreeMap<String, MethodRow>,
    links: BTreeMap<String, ArtifactLinkRow>,
    relationships: BTreeMap<String, RelationshipRow>,
    cohorts: BTreeMap<String, CohortRow>,
    missing: BTreeMap<String, MissingRow>,
    summary: GraphSummary,
}

#[derive(Debug)]
struct LoadedInput {
    artifacts: BTreeMap<String, ArtifactRow>,
    mode: String,
    artifacts_sha256: String,
    summary_sha256: Option<String>,
}

#[derive(Debug, Clone)]
struct GameplayRecord {
    number: u64,
    key: MethodKey,
    source_path: Option<String>,
    range: Option<SourceRange>,
    body_sha256: Option<String>,
    coverage: String,
    body_anchor_class: String,
    physical_line: u64,
}

#[derive(Debug, Clone)]
struct GameplayTableRow {
    heading: String,
    header: Vec<String>,
    physical_line: usize,
    cells: Vec<String>,
}

#[derive(Debug)]
struct GraphBuild {
    data: GraphData,
    canonical_rows: u64,
    canonical_unique: u64,
    canonical_files: u64,
    canonical_shards: BTreeSet<String>,
    canonical_roots: BTreeSet<String>,
    p1_rows: u64,
    p1_unique: u64,
    p1_overlap_actual: u64,
    p1_new: u64,
    p1_p0_unique: u64,
    p1_p0_overlap: u64,
    p1_shards: BTreeSet<String>,
    p1_roots: BTreeSet<String>,
    authority_binding: Value,
    curated_files: u64,
    curated_methods: u64,
    curated_status_counts: BTreeMap<String, u64>,
    curated_abstract: u64,
    association_summary: Value,
    gameplay_rows: u64,
    gameplay_matched: u64,
    gameplay_source_files: BTreeSet<String>,
    gameplay_primary_sources: BTreeSet<String>,
    gameplay_curated_files: BTreeSet<String>,
    gameplay_curated_rows: u64,
    performance_files: u64,
    performance_curated: u64,
    performance_queue: u64,
    performance_tiers: BTreeMap<String, u64>,
    performance_method_links: u64,
    performance_unmapped: u64,
    performance_rep_ids: BTreeSet<String>,
    body_expected: u64,
    body_verified: u64,
    body_failed: u64,
    body_anchor_classes: BTreeMap<String, u64>,
    duplicate_groups: u64,
    duplicate_paths: u64,
    same_symbol_groups: u64,
    r38_rows: u64,
    r54_staged: u64,
    missing_triad: u64,
    qualification_queue: u64,
    non_managed_counts: BTreeMap<String, u64>,
    abstract_method_key: Option<MethodKey>,
    config_sha256: String,
    source_authority: Value,
    tooling_overlay: Vec<OverlayEntry>,
    authority_assembly: String,
    authority_mvid: String,
    curated_original_paths: BTreeSet<String>,
    curated_status_by_method: BTreeMap<String, String>,
    r38_gameplay: Vec<GameplayRecord>,
}

pub(super) fn run_build(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    let config_path = flags
        .get("--config")
        .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--config is required"))?;
    let snapshot = flags
        .get("--snapshot")
        .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--snapshot is required"))?;
    let output = flags
        .get("--output")
        .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--output is required"))?;
    graph_build(config_path, snapshot, output)
}

pub(super) fn run_verify(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    let config_path = flags
        .get("--config")
        .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--config is required"))?;
    let snapshot = flags
        .get("--snapshot")
        .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--snapshot is required"))?;
    let graph = flags
        .get("--graph")
        .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--graph is required"))?;
    let scratch = flags
        .get("--scratch")
        .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--scratch is required"))?;
    graph_verify(config_path, snapshot, graph, scratch)
}

pub(super) fn run_lookup(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    graph_lookup(flags)
}

fn artifact_key(artifact: &ArtifactRef) -> String {
    format!("{}\0{}", artifact.root_id, artifact.path)
}

fn row_artifact(row: &ArtifactRow) -> RegistryResult<ArtifactRef> {
    if row.status != "OK" {
        return Err(RegistryError::new(
            "GRAPH_INPUT_ARTIFACT_NOT_OK",
            "graph input artifact row is not OK",
        ));
    }
    let sha256 = row.sha256.clone().ok_or_else(|| {
        RegistryError::new(
            "GRAPH_INPUT_ARTIFACT_NOT_HASHED",
            "graph input artifact row has no sha256",
        )
    })?;
    Ok(ArtifactRef {
        root_id: row.root_id.clone(),
        path: row.path.clone(),
        size: row.size,
        sha256,
        kind: row.kind.clone(),
    })
}

fn push_unique_artifact(target: &mut Vec<ArtifactRef>, artifact: &ArtifactRef) {
    if !target
        .iter()
        .any(|existing| artifact_key(existing) == artifact_key(artifact))
    {
        target.push(artifact.clone());
    }
}

fn push_unique_string(target: &mut Vec<String>, value: &str) {
    if !target.iter().any(|existing| existing == value) {
        target.push(value.to_string());
    }
}

fn artifact_lookup_key(root_id: &str, path: &str) -> String {
    format!(
        "{}\0{}",
        root_id.to_ascii_lowercase(),
        path.to_ascii_lowercase()
    )
}

fn lookup_artifact<'a>(
    artifacts: &'a BTreeMap<String, ArtifactRow>,
    root_id: &str,
    path: &str,
) -> Option<&'a ArtifactRow> {
    artifacts.get(&artifact_lookup_key(root_id, path))
}

fn require_artifact(
    artifacts: &BTreeMap<String, ArtifactRow>,
    root_id: &str,
    path: &str,
) -> RegistryResult<ArtifactRef> {
    let row = lookup_artifact(artifacts, root_id, path).ok_or_else(|| {
        RegistryError::new(
            "GRAPH_ARTIFACT_MISSING",
            "a required artifact is not present in the input artifact registry",
        )
    })?;
    row_artifact(row)
}

fn require_artifact_with_hash(
    artifacts: &BTreeMap<String, ArtifactRow>,
    root_id: &str,
    path: &str,
    size: u64,
    sha256: &str,
) -> RegistryResult<ArtifactRef> {
    let artifact = require_artifact(artifacts, root_id, path)?;
    if artifact.size != size || artifact.sha256 != sha256 {
        return Err(RegistryError::new(
            "GRAPH_ARTIFACT_HASH_MISMATCH",
            "artifact size or sha256 does not match the proof row",
        ));
    }
    Ok(artifact)
}

fn root_path(config: &Config, root_id: &str) -> RegistryResult<PathBuf> {
    config
        .roots
        .iter()
        .find(|root| root.id == root_id)
        .map(|root| root.path.clone())
        .ok_or_else(|| {
            RegistryError::new(
                "GRAPH_ROOT_MISSING",
                "the required root id is not configured",
            )
        })
}

fn read_artifact_bytes(
    config: &Config,
    artifacts: &BTreeMap<String, ArtifactRow>,
    root_id: &str,
    path: &str,
) -> RegistryResult<(ArtifactRef, Vec<u8>)> {
    if !validate_snapshot_path(path) {
        return Err(RegistryError::new(
            "GRAPH_PATH_UNSAFE",
            "artifact path is not a safe portable relative path",
        ));
    }
    let root = root_path(config, root_id)?;
    let absolute = root.join(path);
    inspect_existing_file(&absolute)?;
    let bytes = fs::read(&absolute).map_err(|_| {
        RegistryError::new("GRAPH_ARTIFACT_READ_ERROR", "artifact could not be read")
    })?;
    let artifact = require_artifact_with_hash(
        artifacts,
        root_id,
        path,
        bytes.len() as u64,
        &crate::sha256_hex(&bytes),
    )?;
    Ok((artifact, bytes))
}

fn load_graph_input(_config: &Config, snapshot: &Path) -> RegistryResult<LoadedInput> {
    if !snapshot.is_absolute() {
        return Err(RegistryError::new(
            "GRAPH_SNAPSHOT_NOT_ABSOLUTE",
            "snapshot path must be absolute",
        ));
    }
    reject_parent_components(snapshot, "GRAPH_SNAPSHOT_PATH_UNSAFE")?;
    inspect_existing_directory(snapshot)?;
    let snapshot_artifacts = snapshot.join("artifacts.jsonl");
    let snapshot_manifest = snapshot.join("snapshot.json");
    if fs::symlink_metadata(&snapshot_artifacts).is_ok()
        && fs::symlink_metadata(&snapshot_manifest).is_ok()
    {
        let (manifest, rows, computed, _) = super::load_snapshot(snapshot)?;
        if manifest.status != "PASS" {
            return Err(RegistryError::new(
                "GRAPH_INPUT_SNAPSHOT_NOT_PASS",
                "input snapshot is not a complete PASS snapshot",
            ));
        }
        let artifacts = {
            let mut map = BTreeMap::new();
            for row in rows {
                let key = artifact_lookup_key(&row.root_id, &row.path);
                if map.insert(key, row).is_some() {
                    return Err(RegistryError::new(
                        "GRAPH_CASEFOLD_COLLISION",
                        "input snapshot contains casefold-equal artifact paths",
                    ));
                }
            }
            map
        };
        return Ok(LoadedInput {
            artifacts,
            mode: "RUST_SNAPSHOT".to_string(),
            artifacts_sha256: computed,
            summary_sha256: None,
        });
    }
    let baseline_artifacts = snapshot.join("baseline.artifacts.jsonl");
    let baseline_summary = snapshot.join("baseline.summary.json");
    inspect_existing_file(&baseline_artifacts)?;
    inspect_existing_file(&baseline_summary)?;
    let artifacts_bytes = fs::read(&baseline_artifacts).map_err(|_| {
        RegistryError::new(
            "GRAPH_BASELINE_READ_ERROR",
            "baseline artifacts could not be read",
        )
    })?;
    let computed = crate::sha256_hex(&artifacts_bytes);
    let summary_bytes = fs::read(&baseline_summary).map_err(|_| {
        RegistryError::new(
            "GRAPH_BASELINE_READ_ERROR",
            "baseline summary could not be read",
        )
    })?;
    let summary_value = crate::parse_strict_value(&summary_bytes).map_err(|_| {
        RegistryError::new(
            "GRAPH_BASELINE_INVALID",
            "baseline summary JSON is invalid or contains duplicate keys",
        )
    })?;
    if value_str(&summary_value, "status")? != "PASS" {
        return Err(RegistryError::new(
            "GRAPH_BASELINE_NOT_PASS",
            "frozen baseline summary is not PASS",
        ));
    }
    let expected_sha = summary_value
        .pointer("/snapshot/artifacts_sha256_candidate")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            RegistryError::new(
                "GRAPH_BASELINE_INVALID",
                "baseline summary has no artifacts hash candidate",
            )
        })?;
    if expected_sha != computed {
        return Err(RegistryError::new(
            "GRAPH_BASELINE_HASH_MISMATCH",
            "baseline artifacts file hash does not match its summary",
        ));
    }
    let text = std::str::from_utf8(&artifacts_bytes).map_err(|_| {
        RegistryError::new("GRAPH_BASELINE_INVALID", "baseline artifacts are not UTF-8")
    })?;
    let mut artifacts = BTreeMap::new();
    let mut count = 0u64;
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let value = crate::parse_strict_value(line.as_bytes()).map_err(|_| {
            RegistryError::new(
                "GRAPH_BASELINE_INVALID",
                "baseline artifact row is invalid or contains duplicate keys",
            )
        })?;
        let row: ArtifactRow = serde_json::from_value(value).map_err(|_| {
            RegistryError::new(
                "GRAPH_BASELINE_INVALID",
                "baseline artifact row shape is invalid",
            )
        })?;
        validate_artifact_row(&row)?;
        let key = artifact_lookup_key(&row.root_id, &row.path);
        if artifacts.insert(key, row).is_some() {
            return Err(RegistryError::new(
                "GRAPH_CASEFOLD_COLLISION",
                "baseline contains casefold-equal artifact paths",
            ));
        }
        count += 1;
    }
    let expected_count = summary_value
        .pointer("/observed/entries")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            RegistryError::new(
                "GRAPH_BASELINE_INVALID",
                "baseline summary has no entry count",
            )
        })?;
    if count != expected_count {
        return Err(RegistryError::new(
            "GRAPH_BASELINE_COUNT_MISMATCH",
            "baseline artifact row count does not match its summary",
        ));
    }
    Ok(LoadedInput {
        artifacts,
        mode: "FROZEN_BASELINE_DEV".to_string(),
        artifacts_sha256: computed,
        summary_sha256: Some(crate::sha256_hex(&summary_bytes)),
    })
}

fn value_str<'a>(value: &'a Value, key: &str) -> RegistryResult<&'a str> {
    value.get(key).and_then(Value::as_str).ok_or_else(|| {
        RegistryError::new(
            "GRAPH_JSON_FIELD_INVALID",
            format!("missing string field {key}"),
        )
    })
}

fn value_u64(value: &Value, key: &str) -> RegistryResult<u64> {
    value.get(key).and_then(Value::as_u64).ok_or_else(|| {
        RegistryError::new(
            "GRAPH_JSON_FIELD_INVALID",
            format!("missing integer field {key}"),
        )
    })
}

fn value_bool(value: &Value, key: &str) -> RegistryResult<bool> {
    value.get(key).and_then(Value::as_bool).ok_or_else(|| {
        RegistryError::new(
            "GRAPH_JSON_FIELD_INVALID",
            format!("missing boolean field {key}"),
        )
    })
}

fn value_array<'a>(value: &'a Value, key: &str) -> RegistryResult<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array).ok_or_else(|| {
        RegistryError::new(
            "GRAPH_JSON_FIELD_INVALID",
            format!("missing array field {key}"),
        )
    })
}

fn read_json_artifact(
    config: &Config,
    artifacts: &BTreeMap<String, ArtifactRow>,
    root_id: &str,
    path: &str,
) -> RegistryResult<(ArtifactRef, Value)> {
    let (artifact, bytes) = read_artifact_bytes(config, artifacts, root_id, path)?;
    let value = crate::parse_strict_value(&bytes).map_err(|_| {
        RegistryError::new(
            "GRAPH_JSON_INVALID",
            "graph JSON input is invalid or contains duplicate keys",
        )
    })?;
    Ok((artifact, value))
}

fn parse_json_file(
    config: &Config,
    artifacts: &BTreeMap<String, ArtifactRow>,
    root_id: &str,
    path: &str,
) -> RegistryResult<Value> {
    read_json_artifact(config, artifacts, root_id, path).map(|(_, value)| value)
}

fn method_index_by_md(methods: &BTreeMap<String, MethodRow>) -> BTreeMap<String, Vec<MethodKey>> {
    let mut index: BTreeMap<String, Vec<MethodKey>> = BTreeMap::new();
    for row in methods.values() {
        let key = MethodKey {
            assembly_sha256: row.assembly_sha256.clone(),
            mvid: row.mvid.clone(),
            type_token: row.type_token.clone(),
            method_token: row.method_token.clone(),
        };
        index
            .entry(row.method_token.to_lowercase())
            .or_default()
            .push(key);
    }
    index
}

fn method_key_for_token(
    index: &BTreeMap<String, Vec<MethodKey>>,
    token: &str,
) -> Option<MethodKey> {
    let keys = index.get(&norm_token(token))?;
    if keys.len() == 1 {
        Some(keys[0].clone())
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
fn add_performance_rep(
    build: &mut GraphBuild,
    key: &MethodKey,
    file: &ArtifactRef,
    proof: &ArtifactRef,
    tier: &str,
    evidence: Vec<String>,
    coverage: Option<String>,
    note: &str,
) -> RegistryResult<()> {
    let representation_id = format!("PERF:{tier}:{}:{}", file.path, key.method_token);
    if !build.performance_rep_ids.insert(representation_id.clone()) {
        return Ok(());
    }
    if !build.data.methods.contains_key(&key.id()) {
        build.performance_unmapped += 1;
        add_missing(
            &mut build.data.missing,
            format!("PERFORMANCE_METHOD_MISSING:{}:{tier}", key.method_token),
            "PERFORMANCE_METHOD_LINK_UNMAPPED",
            Some(&file.root_id),
            Some(&file.path),
            Some(key.method_token.clone()),
            "structured performance method token does not resolve to a unique qualified method key",
            vec![proof.path.clone()],
        );
        return Ok(());
    }
    let mut labels = vec![tier.to_string()];
    for label in evidence {
        push_unique_string(&mut labels, &label);
    }
    let rep = Representation {
        representation_id,
        representation_kind: "PERFORMANCE_METHOD_LINK".to_string(),
        evidence: labels,
        coverage,
        source_status: None,
        source_ref: Some(file.clone()),
        raw_ref: None,
        semantic_ref: None,
        semantic_range: None,
        proof_ref: Some(proof.clone()),
        proof_row: None,
        raw_row: None,
        source_range: None,
        body_sha256: None,
        body_hash_verified: None,
        table_heading: None,
        table_header: None,
        note: Some(note.to_string()),
    };
    add_method_rep(
        &mut build.data.methods,
        key,
        rep,
        &format!("PERFORMANCE_{tier}"),
        None,
    )?;
    add_link(
        &mut build.data.links,
        file,
        "PERFORMANCE_METHOD",
        Some(&key.id()),
        &format!("PERFORMANCE_{tier}"),
        Some(note),
    );
    build.performance_method_links += 1;
    Ok(())
}

fn performance_path_for_basename(
    performance_paths: &BTreeSet<String>,
    basename: &str,
    prefer_root: bool,
) -> Option<String> {
    let mut candidates = performance_paths
        .iter()
        .filter(|path| short_name(path).as_deref() == Some(basename))
        .cloned()
        .collect::<Vec<_>>();
    candidates.sort();
    if prefer_root {
        if let Some(root) = candidates
            .iter()
            .find(|path| path.as_str() == format!("{PERFORMANCE_PREFIX}{basename}"))
        {
            return Some(root.clone());
        }
    }
    if candidates.len() == 1 {
        Some(candidates.remove(0))
    } else {
        None
    }
}

fn is_canonical_provenance(path: &str) -> bool {
    let Some(rest) = path.strip_prefix(CANONICAL_PREFIX) else {
        return false;
    };
    let Some((shard, tail)) = rest.split_once('/') else {
        return false;
    };
    shard.len() == 64
        && shard.bytes().all(|byte| byte.is_ascii_hexdigit())
        && tail == "evidence/METHOD_PROVENANCE.jsonl"
}

fn is_p1_methods(path: &str) -> bool {
    let Some(rest) = path.strip_prefix(P1_PREFIX) else {
        return false;
    };
    let Some((_, tail)) = rest.split_once('/') else {
        return false;
    };
    tail == "evidence/methods.jsonl"
}

fn is_performance_source(path: &str) -> bool {
    path.starts_with(PERFORMANCE_PREFIX) && path.ends_with(".cs")
}

fn is_canonical_source(path: &str) -> bool {
    path.starts_with(CANONICAL_PREFIX) && path.ends_with(".cs")
}

fn is_semantic_source(path: &str) -> bool {
    path.starts_with(GAMEPLAY_PREFIX) && path.ends_with(".cs")
}

fn short_name(path: &str) -> Option<String> {
    path.rsplit('/').next().map(str::to_string)
}

fn add_link(
    links: &mut BTreeMap<String, ArtifactLinkRow>,
    artifact: &ArtifactRef,
    role: &str,
    method_id: Option<&str>,
    cohort: &str,
    note: Option<&str>,
) {
    let key = artifact_key(artifact);
    let row = links.entry(key).or_insert_with(|| ArtifactLinkRow {
        artifact: artifact.clone(),
        roles: Vec::new(),
        method_ids: Vec::new(),
        cohorts: Vec::new(),
        notes: Vec::new(),
    });
    push_unique_string(&mut row.roles, role);
    if let Some(method_id) = method_id {
        push_unique_string(&mut row.method_ids, method_id);
    }
    push_unique_string(&mut row.cohorts, cohort);
    if let Some(note) = note {
        push_unique_string(&mut row.notes, note);
    }
}

fn add_missing(
    missing: &mut BTreeMap<String, MissingRow>,
    id: String,
    kind: &str,
    root_id: Option<&str>,
    path: Option<&str>,
    expected: Option<String>,
    reason: &str,
    refs: Vec<String>,
) {
    missing.entry(id.clone()).or_insert(MissingRow {
        missing_id: id,
        kind: kind.to_string(),
        root_id: root_id.map(str::to_string),
        path: path.map(str::to_string),
        expected,
        reason: reason.to_string(),
        refs,
    });
}

fn add_relationship(
    relationships: &mut BTreeMap<String, RelationshipRow>,
    id: String,
    kind: &str,
    status: &str,
    subjects: Vec<String>,
    details: Value,
) {
    relationships.entry(id.clone()).or_insert(RelationshipRow {
        relationship_id: id,
        kind: kind.to_string(),
        status: status.to_string(),
        subjects,
        details,
    });
}

fn add_cohort(
    cohorts: &mut BTreeMap<String, CohortRow>,
    id: &str,
    kind: &str,
    file_count: u64,
    method_count: u64,
    members: Vec<String>,
    notes: Vec<String>,
) {
    cohorts.entry(id.to_string()).or_insert(CohortRow {
        cohort_id: id.to_string(),
        kind: kind.to_string(),
        file_count,
        method_count,
        members,
        notes,
    });
}

fn validate_token(value: &str, prefix: &str) -> bool {
    value.len() == 10
        && value.starts_with(prefix)
        && value[4..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn norm_token(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn split_md_cells(line: &str) -> Vec<String> {
    let line = line.trim();
    let line = line.strip_prefix('|').unwrap_or(line);
    let line = line.strip_suffix('|').unwrap_or(line);
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut in_backtick = false;
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            current.push(character);
            continue;
        }
        if character == '`' {
            in_backtick = !in_backtick;
            current.push(character);
            continue;
        }
        if character == '|' && !in_backtick {
            cells.push(current.trim().to_string());
            current.clear();
            continue;
        }
        current.push(character);
    }
    cells.push(current.trim().to_string());
    cells
}

fn is_separator_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let trimmed = cell.trim();
            !trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|character| character == '-' || character == ':' || character == ' ')
                && trimmed.chars().any(|character| character == '-')
        })
}

fn parse_decimal_cell(value: &str) -> Option<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn parse_range(value: &str) -> Option<SourceRange> {
    let value = value.trim().trim_matches('`').trim();
    let (start, end) = value.split_once('-')?;
    let start = start.parse::<u64>().ok()?;
    let end = end.parse::<u64>().ok()?;
    if start == 0 || end < start {
        return None;
    }
    Some(SourceRange {
        start_line: start,
        end_line: end,
    })
}

fn first_range_in_text(value: &str) -> Option<SourceRange> {
    let bytes = value.as_bytes();
    for index in 0..bytes.len() {
        if !bytes[index].is_ascii_digit() {
            continue;
        }
        if index > 0 && bytes[index - 1].is_ascii_digit() {
            continue;
        }
        let mut end = index;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end >= bytes.len() || bytes[end] != b'-' {
            continue;
        }
        let mut range_end = end + 1;
        while range_end < bytes.len() && bytes[range_end].is_ascii_digit() {
            range_end += 1;
        }
        if range_end == end + 1 {
            continue;
        }
        if let Some(range) = parse_range(&value[index..range_end]) {
            return Some(range);
        }
    }
    None
}

fn overlay_range(cell: &str) -> Option<SourceRange> {
    let lower = cell.to_lowercase();
    let index = lower.find("overlay")?;
    first_range_in_text(&cell[index + "overlay".len()..])
}

fn path_and_range(cell: &str) -> Option<(String, Option<SourceRange>)> {
    let trimmed = cell.trim().trim_matches('`').trim();
    for prefix in [CANONICAL_PREFIX, GAMEPLAY_PREFIX] {
        if let Some(index) = trimmed.find(prefix) {
            let tail = &trimmed[index..];
            let end = tail
                .find(|character: char| {
                    character.is_whitespace() || character == ';' || character == ')'
                })
                .unwrap_or(tail.len());
            let candidate = &tail[..end];
            let (path_text, suffix) = if let Some((before, after)) = candidate.split_once('`') {
                (before, after)
            } else {
                (candidate, "")
            };
            let (path, mut range) = if let Some((path, suffix)) = path_text.rsplit_once(':') {
                if let Some(range) = parse_range(suffix) {
                    (path, Some(range))
                } else {
                    (path, None)
                }
            } else {
                (path_text, None)
            };
            if range.is_none() {
                range = first_range_in_text(suffix);
            }
            if range.is_none() {
                range = first_range_in_text(tail);
            }
            return Some((path.trim().to_string(), range));
        }
    }
    None
}

fn normalized_slice_hash(bytes: &[u8], range: &SourceRange) -> RegistryResult<String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| RegistryError::new("GRAPH_SOURCE_NOT_UTF8", "source artifact is not UTF-8"))?;
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<&str> = normalized.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let start = (range.start_line as usize)
        .checked_sub(1)
        .ok_or_else(|| RegistryError::new("GRAPH_RANGE_INVALID", "source range is invalid"))?;
    let end = range.end_line as usize;
    if end > lines.len() || start >= end {
        return Err(RegistryError::new(
            "GRAPH_RANGE_INVALID",
            "source range is outside the UTF-8 LF-normalized source",
        ));
    }
    let mut slice = lines[start..end].join("\n");
    slice.push('\n');
    Ok(crate::sha256_hex(slice.as_bytes()))
}

fn parse_gameplay_tables(text: &str) -> Vec<GameplayTableRow> {
    let lines: Vec<&str> = text.lines().collect();
    let mut rows = Vec::new();
    let mut heading = String::new();
    let mut header: Vec<String> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if let Some(stripped) = line.strip_prefix('#') {
            heading = stripped.trim_start_matches('#').trim().to_string();
            header.clear();
            continue;
        }
        if !line.trim_start().starts_with('|') {
            header.clear();
            continue;
        }
        let cells = split_md_cells(line);
        if index + 1 < lines.len() && lines[index + 1].trim_start().starts_with('|') {
            let next = split_md_cells(lines[index + 1]);
            if is_separator_row(&next) {
                header = cells;
                continue;
            }
        }
        if is_separator_row(&cells) || header.is_empty() {
            continue;
        }
        rows.push(GameplayTableRow {
            heading: heading.clone(),
            header: header.clone(),
            physical_line: index + 1,
            cells,
        });
    }
    rows
}

fn header_has_body_sha(header: &str) -> bool {
    let normalized = header
        .to_lowercase()
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>();
    normalized.contains("bodysha256")
        || normalized.contains("bodysourcesha")
        || normalized.contains("sourceslicesha")
        || normalized.contains("sourceslicsha")
        || normalized == "bodysha"
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn coverage_from_cells(cells: &[String]) -> String {
    const LABELS: [&str; 9] = [
        "ASYNC_LIFTED_FULL_SEMANTIC_BODY",
        "FULL_SELECTED_BODY",
        "SOURCE_ONLY_PRIMARY_ILSPY",
        "SOURCE_AND_RAW_SUMMARY_ONLY",
        "SOURCE_AND_METADATA_ONLY",
        "PRIMARY_FRESH_CIL",
        "FULL_CIL",
        "SOURCE_ONLY",
        "UNKNOWN",
    ];
    let mut found = Vec::new();
    for cell in cells {
        for label in LABELS {
            if cell.contains(label) {
                push_unique_string(&mut found, label);
            }
        }
    }
    if found.is_empty() {
        "UNKNOWN".to_string()
    } else {
        found.join("; ")
    }
}

fn gameplay_anchor_class(
    has_semantic_anchor: bool,
    range: Option<&SourceRange>,
    body_hash: Option<&str>,
) -> &'static str {
    if body_hash.is_some() {
        if has_semantic_anchor && range.is_some() {
            "EXPLICIT_RANGE_WITH_BODY_SHA"
        } else {
            "BODY_HASH_WITHOUT_SEMANTIC_RANGE"
        }
    } else if !has_semantic_anchor {
        "UNRESOLVED_ANCHOR"
    } else if range.is_some() {
        "EXPLICIT_RANGE_NO_BODY_SHA"
    } else {
        "FILE_OR_METHOD_ANCHOR_ONLY"
    }
}

fn token_positions(value: &str, prefix: &str) -> Vec<(usize, String)> {
    let bytes = value.as_bytes();
    let mut positions = Vec::new();
    let mut index = 0;
    while index + 10 <= bytes.len() {
        if bytes[index] == b'0' && bytes[index + 1] == b'x' {
            if let Some(candidate) = value.get(index..index + 10) {
                if candidate.starts_with(prefix)
                    && candidate
                        .bytes()
                        .skip(2)
                        .all(|byte| byte.is_ascii_hexdigit())
                {
                    positions.push((index, candidate.to_ascii_lowercase()));
                    index += 10;
                    continue;
                }
            }
        }
        index += 1;
    }
    positions
}

fn identity_pair(cell: &str) -> Option<(String, String)> {
    let td = token_positions(cell, "0x02");
    let md = token_positions(cell, "0x06");
    if td.len() != 1 || md.len() != 1 {
        return None;
    }
    let td_end = td[0].0 + 10;
    if td_end > md[0].0 {
        return None;
    }
    let between = cell[td_end..md[0].0].trim().trim_matches('`').trim();
    if between != "/" {
        return None;
    }
    Some((td[0].1.clone(), md[0].1.clone()))
}

fn short_filename_with_range(cell: &str) -> Option<(String, Option<SourceRange>)> {
    let cleaned = cell.trim().trim_matches('`').trim();
    if cleaned.contains('/') {
        return None;
    }
    let mut search = 0;
    while let Some(relative) = cleaned[search..].find(".cs") {
        let end = search + relative + 3;
        let start = cleaned[..end - 3]
            .rfind(|character: char| {
                !(character.is_ascii_alphanumeric()
                    || character == '_'
                    || character == '.'
                    || character == '-')
            })
            .map(|index| index + 1)
            .unwrap_or(0);
        let name = &cleaned[start..end];
        if !name.contains('/') && !name.is_empty() {
            let after = cleaned[end..].trim_start();
            let range = after.strip_prefix(':').and_then(|suffix| {
                let end = suffix
                    .find(|character: char| {
                        character.is_whitespace()
                            || character == '`'
                            || character == ';'
                            || character == ')'
                            || character == ','
                    })
                    .unwrap_or(suffix.len());
                parse_range(&suffix[..end])
            });
            return Some((name.to_string(), range));
        }
        search = end;
    }
    None
}

fn basename_map(artifacts: &BTreeMap<String, ArtifactRow>) -> BTreeMap<String, Vec<ArtifactRef>> {
    let mut map: BTreeMap<String, Vec<ArtifactRef>> = BTreeMap::new();
    for row in artifacts.values() {
        if row.status != "OK" {
            continue;
        }
        if !is_canonical_source(&row.path) && !is_semantic_source(&row.path) {
            continue;
        }
        if let Some(artifact) = row_artifact(row).ok() {
            if let Some(name) = short_name(&row.path) {
                map.entry(name).or_default().push(artifact);
            }
        }
    }
    map
}

fn resolve_gameplay_source(
    row: &GameplayTableRow,
    artifacts: &BTreeMap<String, ArtifactRow>,
    basenames: &BTreeMap<String, Vec<ArtifactRef>>,
) -> RegistryResult<(
    Option<ArtifactRef>,
    Option<SourceRange>,
    Option<ArtifactRef>,
    Option<SourceRange>,
    Option<String>,
)> {
    let mut canonical = None;
    let mut canonical_range = None;
    let mut semantic = None;
    let mut semantic_range = None;
    for cell in &row.cells {
        for part in cell.split(';') {
            if let Some((path, range)) = path_and_range(part) {
                if let Some(artifact) = lookup_artifact(artifacts, CANONICAL_ROOT, &path) {
                    let artifact = row_artifact(artifact)?;
                    if path.starts_with(GAMEPLAY_PREFIX) {
                        if semantic.is_none() {
                            semantic = Some(artifact);
                            semantic_range = range;
                        }
                    } else if canonical.is_none() {
                        canonical = Some(artifact);
                        canonical_range = range;
                    }
                }
            }
        }
    }
    let mut short_names: Vec<(String, Option<SourceRange>)> = Vec::new();
    for cell in &row.cells {
        for part in cell.split(';') {
            if let Some((name, range)) = short_filename_with_range(part) {
                if !short_names
                    .iter()
                    .any(|(existing, existing_range)| existing == &name && existing_range == &range)
                {
                    short_names.push((name, range));
                }
            }
        }
    }
    let mut resolution = if canonical.is_some() || semantic.is_some() {
        Some("FULL_PATH".to_string())
    } else {
        Some("UNRESOLVED".to_string())
    };
    for (name, range) in short_names {
        if let Some(candidates) = basenames.get(&name) {
            let semantic_candidates = candidates
                .iter()
                .filter(|candidate| candidate.path.starts_with(GAMEPLAY_PREFIX))
                .cloned()
                .collect::<Vec<_>>();
            if semantic.is_none() && semantic_candidates.len() == 1 {
                semantic = Some(semantic_candidates[0].clone());
                semantic_range = range.clone();
                resolution = Some("SHORT_NAME_SEMANTIC".to_string());
            }
            if canonical.is_none() && candidates.len() == 1 {
                canonical = Some(candidates[0].clone());
                canonical_range = range.clone();
                resolution = Some("SHORT_NAME_CANONICAL".to_string());
            }
            if semantic.is_none() && semantic_candidates.len() > 1 {
                resolution = Some("AMBIGUOUS_SHORT_NAME".to_string());
            }
        }
    }
    if semantic.is_some() && semantic_range.is_none() {
        for cell in &row.cells {
            if let Some(range) = overlay_range(cell) {
                semantic_range = Some(range);
                break;
            }
        }
    }
    Ok((
        canonical,
        canonical_range,
        semantic,
        semantic_range,
        resolution,
    ))
}

fn canonical_shard(path: &str) -> Option<&str> {
    let rest = path.strip_prefix(CANONICAL_PREFIX)?;
    let (shard, tail) = rest.split_once('/')?;
    if shard.len() == 64
        && shard.bytes().all(|byte| byte.is_ascii_hexdigit())
        && tail == "evidence/METHOD_PROVENANCE.jsonl"
    {
        Some(shard)
    } else {
        None
    }
}

fn p1_shard(path: &str) -> Option<&str> {
    let rest = path.strip_prefix(P1_PREFIX)?;
    let (shard, tail) = rest.split_once('/')?;
    if tail == "evidence/methods.jsonl" {
        Some(shard)
    } else {
        None
    }
}

fn add_method_rep(
    methods: &mut BTreeMap<String, MethodRow>,
    key: &MethodKey,
    representation: Representation,
    cohort: &str,
    root_owner: Option<&str>,
) -> RegistryResult<()> {
    let id = key.id();
    let row = methods.entry(id).or_insert_with(|| MethodRow::new(key));
    if let Some(root_owner) = root_owner {
        push_unique_string(&mut row.root_owners, &norm_token(root_owner));
    }
    push_unique_string(&mut row.cohorts, cohort);
    row.add_representation(representation)
}

fn build_canonical(
    build: &mut GraphBuild,
    config: &Config,
    input: &LoadedInput,
) -> RegistryResult<()> {
    let mut provenance = input
        .artifacts
        .values()
        .filter(|row| is_canonical_provenance(&row.path))
        .cloned()
        .collect::<Vec<_>>();
    provenance.sort_by(|left, right| left.path.cmp(&right.path));
    build.canonical_files = provenance.len() as u64;
    for row in provenance {
        let (proof, bytes) =
            read_artifact_bytes(config, &input.artifacts, CANONICAL_ROOT, &row.path)?;
        let text = std::str::from_utf8(&bytes).map_err(|_| {
            RegistryError::new(
                "GRAPH_CANONICAL_NOT_UTF8",
                "canonical provenance is not UTF-8",
            )
        })?;
        let shard = canonical_shard(&row.path).ok_or_else(|| {
            RegistryError::new(
                "GRAPH_CANONICAL_PATH_INVALID",
                "canonical shard path is invalid",
            )
        })?;
        build.canonical_shards.insert(shard.to_string());
        for (line_index, line) in text.lines().enumerate() {
            if line.is_empty() {
                continue;
            }
            let value = crate::parse_strict_value(line.as_bytes()).map_err(|_| {
                RegistryError::new(
                    "GRAPH_CANONICAL_ROW_INVALID",
                    "canonical provenance row is invalid or contains duplicate keys",
                )
            })?;
            let assembly = value_str(&value, "assembly_sha256")?;
            let type_token = value_str(&value, "type_token")?;
            let method_token = value_str(&value, "method_token")?;
            let root_owner = value_str(&value, "root_owner")?;
            build.canonical_roots.insert(norm_token(root_owner));
            let source_path = value_str(&value, "source_path")?;
            let source_size = value_u64(&value, "source_byte_size")?;
            let source_sha = value_str(&value, "source_sha256")?;
            let raw_file = value_str(&value, "raw_file")?;
            let raw_size = value_u64(&value, "raw_byte_size")?;
            let raw_sha = value_str(&value, "raw_sha256")?;
            let raw_row = value_u64(&value, "raw_row_number")?;
            let static_only = value_bool(&value, "static_only")?;
            if norm_token(assembly) != norm_token(&build.authority_assembly) {
                return Err(RegistryError::new(
                    "GRAPH_CANONICAL_AUTHORITY_MISMATCH",
                    "canonical provenance assembly does not match the pinned authority",
                ));
            }
            if !validate_token(type_token, "0x02") || !validate_token(method_token, "0x06") {
                return Err(RegistryError::new(
                    "GRAPH_CANONICAL_TOKEN_INVALID",
                    "canonical provenance method identity token is invalid",
                ));
            }
            if !static_only {
                return Err(RegistryError::new(
                    "GRAPH_CANONICAL_NOT_STATIC_ONLY",
                    "canonical provenance row is not static_only",
                ));
            }
            let source_rel = format!("{CANONICAL_PREFIX}{shard}/{source_path}");
            let raw_rel = format!("{CANONICAL_PREFIX}{shard}/evidence/raw/{raw_file}");
            let source = match require_artifact_with_hash(
                &input.artifacts,
                CANONICAL_ROOT,
                &source_rel,
                source_size,
                source_sha,
            ) {
                Ok(source) => source,
                Err(_) => {
                    add_missing(
                        &mut build.data.missing,
                        format!("CANONICAL_SOURCE:{shard}:{}", line_index + 1),
                        "GRAPH_CANONICAL_SOURCE_MISMATCH",
                        Some(CANONICAL_ROOT),
                        Some(&source_rel),
                        Some(format!("{source_size}/{source_sha}")),
                        "canonical source artifact is absent or fails its size/hash locator",
                        vec![proof.path.clone()],
                    );
                    continue;
                }
            };
            let raw = match require_artifact_with_hash(
                &input.artifacts,
                CANONICAL_ROOT,
                &raw_rel,
                raw_size,
                raw_sha,
            ) {
                Ok(raw) => raw,
                Err(_) => {
                    add_missing(
                        &mut build.data.missing,
                        format!("CANONICAL_RAW:{shard}:{}", line_index + 1),
                        "GRAPH_CANONICAL_RAW_MISMATCH",
                        Some(CANONICAL_ROOT),
                        Some(&raw_rel),
                        Some(format!("{raw_size}/{raw_sha}")),
                        "canonical raw artifact is absent or fails its size/hash locator",
                        vec![proof.path.clone()],
                    );
                    continue;
                }
            };
            let key = MethodKey {
                assembly_sha256: norm_token(assembly),
                mvid: norm_token(&build.authority_mvid),
                type_token: norm_token(type_token),
                method_token: norm_token(method_token),
            };
            let rep = Representation {
                representation_id: format!("CANON:{shard}:{}", line_index + 1),
                representation_kind: "CANONICAL_METHOD_PROVENANCE".to_string(),
                evidence: vec!["METADATA_RAW_PROVENANCE".to_string()],
                coverage: Some("METHOD_PROVENANCE_METADATA".to_string()),
                source_status: None,
                source_ref: Some(source.clone()),
                raw_ref: Some(raw.clone()),
                semantic_ref: None,
                semantic_range: None,
                proof_ref: Some(proof.clone()),
                proof_row: Some((line_index + 1) as u64),
                raw_row: Some(raw_row),
                source_range: None,
                body_sha256: None,
                body_hash_verified: None,
                table_heading: None,
                table_header: None,
                note: Some(
                    "Canonical metadata/raw provenance is not DIRECT_CIL_FULL evidence."
                        .to_string(),
                ),
            };
            add_method_rep(
                &mut build.data.methods,
                &key,
                rep,
                "CANONICAL_PROVENANCE_165",
                Some(root_owner),
            )?;
            let method_id = key.id();
            add_link(
                &mut build.data.links,
                &source,
                "CANONICAL_SOURCE",
                Some(&method_id),
                "CANONICAL_PROVENANCE_165",
                None,
            );
            add_link(
                &mut build.data.links,
                &raw,
                "CANONICAL_RAW",
                Some(&method_id),
                "CANONICAL_PROVENANCE_165",
                Some(&format!("raw row {raw_row}")),
            );
            build.canonical_rows += 1;
        }
    }
    build.canonical_unique = build.data.methods.len() as u64;
    Ok(())
}

fn method_row_sort_key(row: &MethodRow) -> String {
    format!(
        "{}\0{}\0{}\0{}",
        row.assembly_sha256, row.mvid, row.type_token, row.method_token
    )
}

fn key_set_overlap(left: &BTreeSet<String>, right: &BTreeSet<String>) -> u64 {
    left.intersection(right).count() as u64
}

fn build_p1(build: &mut GraphBuild, config: &Config, input: &LoadedInput) -> RegistryResult<()> {
    let mut files = input
        .artifacts
        .values()
        .filter(|row| is_p1_methods(&row.path))
        .cloned()
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let baseline_keys = build
        .data
        .methods
        .values()
        .map(method_row_sort_key)
        .collect::<BTreeSet<_>>();
    let mut p1_keys = BTreeSet::new();
    let mut p1_p0_keys = BTreeSet::new();
    for row in files {
        let shard = p1_shard(&row.path).ok_or_else(|| {
            RegistryError::new("GRAPH_P1_PATH_INVALID", "P1 shard path is invalid")
        })?;
        build.p1_shards.insert(shard.to_string());
        let completion_path = format!("{P1_PREFIX}{shard}/COMPLETION.json");
        let completion =
            parse_json_file(config, &input.artifacts, CANONICAL_ROOT, &completion_path)?;
        let assembly = value_str(&completion, "assembly_sha256")?;
        if norm_token(assembly) != norm_token(&build.authority_assembly) {
            return Err(RegistryError::new(
                "GRAPH_P1_AUTHORITY_MISMATCH",
                "P1 completion assembly does not match the pinned authority",
            ));
        }
        let (proof, bytes) =
            read_artifact_bytes(config, &input.artifacts, CANONICAL_ROOT, &row.path)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| RegistryError::new("GRAPH_P1_NOT_UTF8", "P1 method list is not UTF-8"))?;
        for (line_index, line) in text.lines().enumerate() {
            if line.is_empty() {
                continue;
            }
            let value = crate::parse_strict_value(line.as_bytes()).map_err(|_| {
                RegistryError::new(
                    "GRAPH_P1_ROW_INVALID",
                    "P1 method row is invalid or contains duplicate keys",
                )
            })?;
            let type_token = value_str(&value, "declaring_type_token")?;
            let method_token = value_str(&value, "method_def_token")?;
            let root_type = value_str(&value, "root_type_token")?;
            build.p1_roots.insert(norm_token(root_type));
            let source_path = value_str(&value, "source_path")?;
            let source_sha = value_str(&value, "source_file_sha256")?;
            let metadata_name = value_str(&value, "metadata_name")?;
            let full_type = value_str(&value, "full_declaring_type")?;
            let raw_body_sha = value_str(&value, "raw_body_sha256")?;
            let decompilation = value_str(&value, "decompilation_status")?;
            let dependency = value_str(&value, "dependency_status")?;
            if !validate_token(type_token, "0x02") || !validate_token(method_token, "0x06") {
                return Err(RegistryError::new(
                    "GRAPH_P1_TOKEN_INVALID",
                    "P1 method identity token is invalid",
                ));
            }
            let source = lookup_artifact(&input.artifacts, CANONICAL_ROOT, source_path)
                .ok_or_else(|| {
                    RegistryError::new(
                        "GRAPH_P1_SOURCE_MISSING",
                        "P1 source artifact is not in the baseline registry",
                    )
                })?;
            let source_artifact = row_artifact(source)?;
            if source_artifact.sha256 != norm_token(source_sha) {
                return Err(RegistryError::new(
                    "GRAPH_P1_SOURCE_HASH_MISMATCH",
                    "P1 source hash does not match the baseline artifact",
                ));
            }
            let key = MethodKey {
                assembly_sha256: norm_token(assembly),
                mvid: norm_token(&build.authority_mvid),
                type_token: norm_token(type_token),
                method_token: norm_token(method_token),
            };
            p1_keys.insert(key.sort_key());
            if shard.starts_with("R5B2B-P1-P0-") {
                p1_p0_keys.insert(key.sort_key());
            }
            let rep = Representation {
                representation_id: format!("P1:{shard}:{}", line_index + 1),
                representation_kind: "P1_METHOD_PROVENANCE".to_string(),
                evidence: vec![
                    "P1_METHOD_PROVENANCE".to_string(),
                    "RAW_BODY_HASH_ONLY".to_string(),
                ],
                coverage: Some(format!("{decompilation}/{dependency}")),
                source_status: None,
                source_ref: Some(source_artifact.clone()),
                raw_ref: None,
                semantic_ref: None,
                semantic_range: None,
                proof_ref: Some(proof.clone()),
                proof_row: Some((line_index + 1) as u64),
                raw_row: None,
                source_range: None,
                body_sha256: Some(raw_body_sha.to_string()),
                body_hash_verified: Some(false),
                table_heading: None,
                table_header: None,
                note: Some(format!(
                    "P1 metadata; full_declaring_type={full_type}; metadata_name={metadata_name}; raw_body_sha256 is retained but is not a source-slice verification"
                )),
            };
            add_method_rep(
                &mut build.data.methods,
                &key,
                rep,
                "P1_5_FILES",
                Some(root_type),
            )?;
            add_link(
                &mut build.data.links,
                &source_artifact,
                "P1_SOURCE",
                Some(&key.id()),
                "P1_5_FILES",
                Some(&format!("{shard} row {}", line_index + 1)),
            );
            build.p1_rows += 1;
        }
    }
    build.p1_unique = p1_keys.len() as u64;
    build.p1_overlap_actual = key_set_overlap(&baseline_keys, &p1_keys);
    build.p1_new = build.p1_unique.saturating_sub(build.p1_overlap_actual);
    build.p1_p0_unique = p1_p0_keys.len() as u64;
    build.p1_p0_overlap = key_set_overlap(&baseline_keys, &p1_p0_keys);
    Ok(())
}

fn string_set_sha256(values: &BTreeSet<String>) -> String {
    let mut bytes = Vec::new();
    for value in values {
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(b'\n');
    }
    crate::sha256_hex(&bytes)
}

fn require_authority_count(label: &str, expected: u64, actual: u64) -> RegistryResult<()> {
    if expected != actual {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_COUNT_MISMATCH",
            "accepted authority count does not match the derived partition",
        ));
    }
    let _ = label;
    Ok(())
}

fn require_authority_set(
    label: &str,
    expected: &BTreeSet<String>,
    actual: &BTreeSet<String>,
) -> RegistryResult<()> {
    if expected != actual {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_SET_MISMATCH",
            "accepted authority set does not match the derived partition",
        ));
    }
    let _ = label;
    Ok(())
}

fn require_authority_subset(
    label: &str,
    subset: &BTreeSet<String>,
    superset: &BTreeSet<String>,
) -> RegistryResult<()> {
    if !subset.is_subset(superset) {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_SUBSET_MISMATCH",
            "derived root set is not a subset of the accepted authority set",
        ));
    }
    let _ = label;
    Ok(())
}

fn require_authority_disjoint(
    label: &str,
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
) -> RegistryResult<()> {
    if !left.is_disjoint(right) {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_ROOT_OVERLAP",
            "accepted FULL and preseed root sets overlap",
        ));
    }
    let _ = label;
    Ok(())
}

fn read_authority_jsonl(
    config: &Config,
    input: &LoadedInput,
    path: &str,
) -> RegistryResult<(ArtifactRef, Vec<Value>)> {
    let (artifact, bytes) = read_artifact_bytes(config, &input.artifacts, CANONICAL_ROOT, path)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        RegistryError::new(
            "GRAPH_AUTHORITY_NOT_UTF8",
            "accepted authority JSONL is not UTF-8",
        )
    })?;
    let mut values = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        values.push(crate::parse_strict_value(line.as_bytes()).map_err(|_| {
            RegistryError::new(
                "GRAPH_AUTHORITY_ROW_INVALID",
                "accepted authority JSONL row is invalid or has duplicate keys",
            )
        })?);
    }
    Ok((artifact, values))
}

fn build_authority_partition(
    build: &mut GraphBuild,
    config: &Config,
    input: &LoadedInput,
) -> RegistryResult<()> {
    let (plan_artifact, plan) = read_json_artifact(
        config,
        &input.artifacts,
        CANONICAL_ROOT,
        AUTHORITY_PLAN_PATH,
    )?;
    if plan_artifact.sha256 != ACCEPTED_PLAN_SHA256 {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_HASH_MISMATCH",
            "accepted shard plan hash does not match the pinned authority",
        ));
    }
    let plan_assembly = norm_token(value_str(&plan, "assembly_sha256")?);
    if plan_assembly != ACCEPTED_ASSEMBLY_SHA256
        || plan_assembly != norm_token(&build.authority_assembly)
    {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_ASSEMBLY_MISMATCH",
            "accepted shard plan assembly does not match the graph authority",
        ));
    }
    if value_str(&plan, "parent_p1_post_integration_manifest")? != AUTHORITY_PARENT_P1_MANIFEST_PATH
        || value_str(&plan, "parent_p1_post_integration_manifest_sha256")?
            != ACCEPTED_PARENT_MANIFEST_SHA256
    {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_PARENT_BINDING_MISMATCH",
            "shard plan parent P1 manifest binding does not match the pinned authority",
        ));
    }
    if value_str(&plan, "preseed_map_path")? != AUTHORITY_PRESEED_PATH
        || value_str(&plan, "preseed_set_identity")? != ACCEPTED_PRESEED_SHA256
    {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_PRESEED_BINDING_MISMATCH",
            "shard plan preseed binding does not match the pinned authority",
        ));
    }
    require_authority_count(
        "eligible roots",
        ACCEPTED_ELIGIBLE_ROOTS,
        value_u64(&plan, "eligible_root_count")?,
    )?;
    require_authority_count(
        "full roots",
        ACCEPTED_FULL_ROOTS,
        value_u64(&plan, "full_recovery_root_count")?,
    )?;
    require_authority_count(
        "preseed roots",
        ACCEPTED_PRESEED_ROOTS,
        value_u64(&plan, "preseed_root_count")?,
    )?;
    require_authority_count(
        "production shards",
        ACCEPTED_CANONICAL_SHARDS,
        value_u64(&plan, "production_shard_count")?,
    )?;
    if plan
        .pointer("/totals/census_totals/validated_il_bodied_methods")
        .and_then(Value::as_u64)
        != Some(ACCEPTED_ALL5_METHODS)
    {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_METHOD_COUNT_MISMATCH",
            "accepted plan census IL-bodied method total is not the accepted all-five total",
        ));
    }

    let mut plan_shards = BTreeSet::new();
    let mut plan_roots = BTreeSet::new();
    let mut plan_root_count = 0u64;
    let mut plan_method_sum = 0u64;
    for shard in value_array(&plan, "shards")? {
        let shard_id = value_str(shard, "shard_id")?;
        if !is_lower_hex_64(shard_id) || !plan_shards.insert(shard_id.to_string()) {
            return Err(RegistryError::new(
                "GRAPH_AUTHORITY_SHARD_INVALID",
                "accepted plan has an invalid or duplicate shard id",
            ));
        }
        let roots = value_array(shard, "root_tokens")?;
        let root_count = value_u64(shard, "root_count")?;
        if root_count != roots.len() as u64 {
            return Err(RegistryError::new(
                "GRAPH_AUTHORITY_ROOT_COUNT_MISMATCH",
                "accepted plan shard root count does not match its root token list",
            ));
        }
        plan_root_count += root_count;
        for root in roots {
            let root = root.as_str().ok_or_else(|| {
                RegistryError::new(
                    "GRAPH_AUTHORITY_ROOT_INVALID",
                    "accepted plan root token is not a string",
                )
            })?;
            if !validate_token(root, "0x02") || !plan_roots.insert(norm_token(root)) {
                return Err(RegistryError::new(
                    "GRAPH_AUTHORITY_ROOT_INVALID",
                    "accepted plan has an invalid or duplicate root token",
                ));
            }
        }
        plan_method_sum += value_u64(shard, "represented_il_bodied_methods")?;
    }
    require_authority_count(
        "plan shard count",
        ACCEPTED_CANONICAL_SHARDS,
        plan_shards.len() as u64,
    )?;
    require_authority_count("plan root count", ACCEPTED_FULL_ROOTS, plan_root_count)?;
    require_authority_count(
        "plan root set count",
        ACCEPTED_FULL_ROOTS,
        plan_roots.len() as u64,
    )?;
    require_authority_count(
        "plan method sum",
        ACCEPTED_CANONICAL_METHODS,
        plan_method_sum,
    )?;

    let (preseed_artifact, preseed_rows) =
        read_authority_jsonl(config, input, AUTHORITY_PRESEED_PATH)?;
    if preseed_artifact.sha256 != ACCEPTED_PRESEED_SHA256 {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_HASH_MISMATCH",
            "accepted preseed map hash does not match the pinned authority",
        ));
    }
    require_authority_count(
        "preseed row count",
        ACCEPTED_PRESEED_ROOTS,
        preseed_rows.len() as u64,
    )?;
    let mut preseed_roots = BTreeSet::new();
    let mut preseed_shards = BTreeSet::new();
    for row in &preseed_rows {
        if value_str(row, "root_mode")? != "PRESEEDED_ACCEPTED"
            || !value_bool(row, "re_decompile_forbidden")?
            || !value_bool(row, "source_copy_forbidden")?
        {
            return Err(RegistryError::new(
                "GRAPH_AUTHORITY_PRESEED_POLICY_MISMATCH",
                "accepted preseed row is not a forbidden PRESEEDED_ACCEPTED row",
            ));
        }
        let root = norm_token(value_str(row, "root_type_token")?);
        let shard = value_str(row, "p1_shard_id")?;
        if !validate_token(&root, "0x02") || !preseed_roots.insert(root) {
            return Err(RegistryError::new(
                "GRAPH_AUTHORITY_PRESEED_DUPLICATE",
                "accepted preseed map has an invalid or duplicate root identity",
            ));
        }
        preseed_shards.insert(shard.to_string());
        let source_path = value_str(row, "p1_source_path")?;
        let source_sha = norm_token(value_str(row, "p1_source_sha256")?);
        if !source_path.starts_with(P1_PREFIX)
            || !source_path.contains(&format!("/{shard}/source/"))
        {
            return Err(RegistryError::new(
                "GRAPH_AUTHORITY_PRESEED_SOURCE_PATH",
                "accepted preseed source path does not match its P1 shard",
            ));
        }
        let source = require_artifact(&input.artifacts, CANONICAL_ROOT, source_path)?;
        if source.sha256 != source_sha {
            return Err(RegistryError::new(
                "GRAPH_AUTHORITY_PRESEED_SOURCE_HASH",
                "accepted preseed source hash does not match the baseline artifact",
            ));
        }
    }
    require_authority_count(
        "preseed root set count",
        ACCEPTED_PRESEED_ROOTS,
        preseed_roots.len() as u64,
    )?;
    require_authority_count(
        "preseed shard count",
        ACCEPTED_P1_SHARDS,
        preseed_shards.len() as u64,
    )?;
    require_authority_disjoint("FULL/preseed roots", &plan_roots, &preseed_roots)?;

    let (completion_artifact, completion_rows) =
        read_authority_jsonl(config, input, AUTHORITY_COMPLETION_PATH)?;
    if completion_artifact.sha256 != ACCEPTED_COMPLETION_SHA256 {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_HASH_MISMATCH",
            "accepted P1 completion hash does not match the pinned authority",
        ));
    }
    require_authority_count(
        "P1 completion shard count",
        ACCEPTED_P1_SHARDS,
        completion_rows.len() as u64,
    )?;
    let mut completion_shards = BTreeSet::new();
    let mut completion_roots = 0u64;
    let mut completion_method_sum = 0u64;
    for row in &completion_rows {
        if value_str(row, "status")? != "COMPLETE"
            || norm_token(value_str(row, "assembly_sha256")?) != ACCEPTED_ASSEMBLY_SHA256
        {
            return Err(RegistryError::new(
                "GRAPH_AUTHORITY_COMPLETION_STATUS",
                "accepted P1 completion row is not COMPLETE for the accepted assembly",
            ));
        }
        let shard = value_str(row, "shard_id")?;
        if !shard.starts_with("R5B2B-P1-") || !completion_shards.insert(shard.to_string()) {
            return Err(RegistryError::new(
                "GRAPH_AUTHORITY_COMPLETION_SHARD",
                "accepted P1 completion has an invalid or duplicate shard id",
            ));
        }
        completion_roots += value_u64(row, "root_count")?;
        completion_method_sum += value_u64(row, "il_bodied_method_count")?;
    }
    require_authority_count(
        "P1 completion root sum",
        ACCEPTED_PRESEED_ROOTS,
        completion_roots,
    )?;
    require_authority_count(
        "P1 completion method sum",
        ACCEPTED_P1_METHODS,
        completion_method_sum,
    )?;
    require_authority_set(
        "preseed/completion shards",
        &completion_shards,
        &preseed_shards,
    )?;

    let (parent_artifact, parent_bytes) = read_artifact_bytes(
        config,
        &input.artifacts,
        CANONICAL_ROOT,
        AUTHORITY_PARENT_P1_MANIFEST_PATH,
    )?;
    if parent_artifact.sha256 != ACCEPTED_PARENT_MANIFEST_SHA256 {
        return Err(RegistryError::new(
            "GRAPH_AUTHORITY_HASH_MISMATCH",
            "accepted parent P1 manifest hash does not match the pinned authority",
        ));
    }
    let parent_text = std::str::from_utf8(&parent_bytes).map_err(|_| {
        RegistryError::new(
            "GRAPH_AUTHORITY_NOT_UTF8",
            "accepted parent P1 manifest is not UTF-8",
        )
    })?;
    for required in [
        format!("{ACCEPTED_ASSEMBLY_SHA256}  private/r5b1/materialized/CSharpScript.dll"),
        format!("{ACCEPTED_COMPLETION_SHA256}  evidence/r5b2b_p1_completion_records.jsonl"),
        format!("{PARENT_HISTORICAL_REVIEWS_SHA256}  evidence/reviews.jsonl"),
    ] {
        if !parent_text.lines().any(|line| line == required) {
            return Err(RegistryError::new(
                "GRAPH_AUTHORITY_PARENT_LINE_MISMATCH",
                "accepted parent P1 manifest is missing a pinned authority line",
            ));
        }
    }

    let (reviews_artifact, review_rows) =
        read_authority_jsonl(config, input, AUTHORITY_REVIEWS_PATH)?;
    let mut review_dispositions = BTreeMap::new();
    for row in &review_rows {
        let Some(id) = row.get("id").and_then(Value::as_str) else {
            continue;
        };
        if id != ACCEPTED_P1_REVIEW_ID && id != ACCEPTED_FULL_REVIEW_ID {
            continue;
        }
        if review_dispositions.contains_key(id)
            || value_str(row, "record_kind")? != "review"
            || value_str(row, "disposition")? != "ACCEPT"
        {
            return Err(RegistryError::new(
                "GRAPH_AUTHORITY_REVIEW_MISMATCH",
                "accepted review id is absent, duplicated, or not ACCEPT",
            ));
        }
        review_dispositions.insert(id.to_string(), "ACCEPT".to_string());
    }
    for id in [ACCEPTED_P1_REVIEW_ID, ACCEPTED_FULL_REVIEW_ID] {
        if !review_dispositions.contains_key(id) {
            return Err(RegistryError::new(
                "GRAPH_AUTHORITY_REVIEW_MISSING",
                "accepted review id is missing from the current reviews artifact",
            ));
        }
    }

    require_authority_set("canonical shards", &plan_shards, &build.canonical_shards)?;
    require_authority_set("P1 shards", &completion_shards, &build.p1_shards)?;
    require_authority_subset("canonical roots", &build.canonical_roots, &plan_roots)?;
    require_authority_subset("P1 roots", &build.p1_roots, &preseed_roots)?;
    require_authority_count(
        "canonical method rows",
        ACCEPTED_CANONICAL_METHODS,
        build.canonical_rows,
    )?;
    require_authority_count("P1 method rows", ACCEPTED_P1_METHODS, build.p1_rows)?;
    require_authority_count(
        "canonical unique methods",
        ACCEPTED_CANONICAL_METHODS,
        build.canonical_unique,
    )?;
    require_authority_count("P1 unique methods", ACCEPTED_P1_METHODS, build.p1_unique)?;
    require_authority_count("P1 overlap", 0, build.p1_overlap_actual)?;
    require_authority_count("P0 overlap", 0, build.p1_p0_overlap)?;
    require_authority_count("curated abstract", 1, build.curated_abstract)?;
    let known_methods = build.data.methods.len() as u64;
    let mapped_all5 = known_methods.saturating_sub(build.curated_abstract);
    require_authority_count(
        "all-five mapped methods",
        ACCEPTED_ALL5_METHODS,
        mapped_all5,
    )?;
    require_authority_count(
        "known methods incl abstract",
        ACCEPTED_KNOWN_METHODS,
        known_methods,
    )?;
    let canonical_plus_p0 = build.canonical_unique + build.p1_p0_unique - build.p1_p0_overlap;
    require_authority_count(
        "initial partial P0 union",
        ACCEPTED_INITIAL_PARTIAL,
        canonical_plus_p0,
    )?;
    let omitted_other4 = mapped_all5.saturating_sub(canonical_plus_p0);
    require_authority_count(
        "omitted other four P1 shards",
        ACCEPTED_OMITTED_OTHER4,
        omitted_other4,
    )?;

    let plan_shard_sha = string_set_sha256(&plan_shards);
    let p1_shard_sha = string_set_sha256(&completion_shards);
    let plan_root_sha = string_set_sha256(&plan_roots);
    let preseed_root_sha = string_set_sha256(&preseed_roots);
    let canonical_root_sha = string_set_sha256(&build.canonical_roots);
    let p1_root_sha = string_set_sha256(&build.p1_roots);
    let mut partition_members = plan_shards
        .iter()
        .chain(completion_shards.iter())
        .cloned()
        .collect::<Vec<_>>();
    partition_members.sort();

    let details = json!({
        "authority_schema": "r5b2b-accepted-canonical-p1-partition-v1",
        "assembly_sha256": ACCEPTED_ASSEMBLY_SHA256,
        "accepted_inputs": {
            "shard_plan": {"path": AUTHORITY_PLAN_PATH, "sha256": plan_artifact.sha256},
            "preseed_map": {"path": AUTHORITY_PRESEED_PATH, "sha256": preseed_artifact.sha256},
            "p1_completion_records": {"path": AUTHORITY_COMPLETION_PATH, "sha256": completion_artifact.sha256},
            "parent_p1_manifest": {"path": AUTHORITY_PARENT_P1_MANIFEST_PATH, "sha256": parent_artifact.sha256},
            "reviews": {"path": AUTHORITY_REVIEWS_PATH, "sha256": reviews_artifact.sha256, "parent_manifest_historical_sha256": PARENT_HISTORICAL_REVIEWS_SHA256}
        },
        "reviews": {
            "ids": [ACCEPTED_P1_REVIEW_ID, ACCEPTED_FULL_REVIEW_ID],
            "dispositions": review_dispositions,
            "scope_note": "Review IDs and ACCEPT dispositions only; actor/session metadata is not embedded."
        },
        "sets": {
            "canonical_shards": {"expected_count": ACCEPTED_CANONICAL_SHARDS, "observed_count": build.canonical_shards.len(), "expected_sha256": plan_shard_sha, "observed_sha256": string_set_sha256(&build.canonical_shards)},
            "p1_shards": {"expected_count": ACCEPTED_P1_SHARDS, "observed_count": build.p1_shards.len(), "expected_sha256": p1_shard_sha, "observed_sha256": string_set_sha256(&build.p1_shards)},
            "plan_full_roots": {"count": ACCEPTED_FULL_ROOTS, "sha256": plan_root_sha},
            "preseed_roots": {"count": ACCEPTED_PRESEED_ROOTS, "sha256": preseed_root_sha},
            "observed_canonical_roots": {"count": build.canonical_roots.len(), "sha256": canonical_root_sha, "subset_of_plan": true},
            "observed_p1_roots": {"count": build.p1_roots.len(), "sha256": p1_root_sha, "subset_of_preseed": true},
            "full_preseed_disjoint": true
        },
        "counts": {
            "eligible_roots": ACCEPTED_ELIGIBLE_ROOTS,
            "full_recovery_roots": ACCEPTED_FULL_ROOTS,
            "preseed_roots": ACCEPTED_PRESEED_ROOTS,
            "canonical_shards": ACCEPTED_CANONICAL_SHARDS,
            "p1_shards": ACCEPTED_P1_SHARDS,
            "canonical_methods": ACCEPTED_CANONICAL_METHODS,
            "p1_methods": ACCEPTED_P1_METHODS,
            "all5_mapped_methods": ACCEPTED_ALL5_METHODS,
            "known_methods_incl_abstract": ACCEPTED_KNOWN_METHODS,
            "canonical_plus_p0_partial": ACCEPTED_INITIAL_PARTIAL,
            "omitted_other_four_p1": ACCEPTED_OMITTED_OTHER4
        },
        "scope": "UNKNOWN_PARTIAL_RECONSTRUCTION_NOT_FULL_SEMANTIC_PROOF",
        "initial_primary_census_correction": {
            "initial_primary_count": ACCEPTED_INITIAL_PARTIAL,
            "initial_primary_scope": "canonical+P0-only partial scope; NOT the accepted historical baseline",
            "canonical_plus_p0_partial": ACCEPTED_INITIAL_PARTIAL,
            "accepted_all5_mapped": ACCEPTED_ALL5_METHODS,
            "omitted_other_four_p1": ACCEPTED_OMITTED_OTHER4,
            "scope_resolution_refs": [AUTHORITY_PLAN_PATH, AUTHORITY_PRESEED_PATH, AUTHORITY_COMPLETION_PATH, AUTHORITY_PARENT_P1_MANIFEST_PATH]
        }
    });
    add_relationship(
        &mut build.data.relationships,
        "ACCEPTED_CANONICAL_P1_PARTITION".to_string(),
        "ACCEPTED_AUTHORITY_PARTITION",
        "ACCEPTED_WITH_UNKNOWN_PARTIAL_SCOPE",
        vec!["canonical_165".to_string(), "p1_5".to_string()],
        details.clone(),
    );
    add_relationship(
        &mut build.data.relationships,
        "INITIAL_PRIMARY_CENSUS_CORRECTION".to_string(),
        "INITIAL_PRIMARY_CENSUS_CORRECTION",
        "RESOLVED_PARTIAL_P0_SCOPE",
        vec!["canonical_303456".to_string(), "p1_p0_123".to_string()],
        json!({
            "initial_primary_count": ACCEPTED_INITIAL_PARTIAL,
            "initial_primary_scope": "canonical+P0-only partial scope; NOT the accepted historical baseline",
            "canonical_plus_p0_partial": ACCEPTED_INITIAL_PARTIAL,
            "accepted_all5_mapped": ACCEPTED_ALL5_METHODS,
            "omitted_other_four_p1": ACCEPTED_OMITTED_OTHER4,
            "scope_resolution_refs": [AUTHORITY_PLAN_PATH, AUTHORITY_PRESEED_PATH, AUTHORITY_COMPLETION_PATH, AUTHORITY_PARENT_P1_MANIFEST_PATH]
        }),
    );
    add_cohort(
        &mut build.data.cohorts,
        "ACCEPTED_CANONICAL_P1_PARTITION",
        "ACCEPTED_AUTHORITY_PARTITION",
        partition_members.len() as u64,
        ACCEPTED_ALL5_METHODS,
        partition_members,
        vec![
            "accepted shard plan, preseed map, P1 completion records and review dispositions bound to exact hashes".to_string(),
            "scope remains UNKNOWN/partial reconstruction; this is not FULL semantic proof".to_string(),
            format!("review_ids={ACCEPTED_P1_REVIEW_ID},{ACCEPTED_FULL_REVIEW_ID}"),
        ],
    );
    build.authority_binding = details;
    Ok(())
}

fn association_summary(v5: &Value, catalogs: &[Value]) -> RegistryResult<Value> {
    let mut method_count_by_type: BTreeMap<String, u64> = BTreeMap::new();
    for entry in value_array(v5, "imports")? {
        if let Some(methods) = entry.get("methods").and_then(Value::as_array) {
            for method in methods {
                let identity = method.get("method_identity").ok_or_else(|| {
                    RegistryError::new(
                        "GRAPH_CURATED_INVALID",
                        "curated method identity is missing",
                    )
                })?;
                let td = norm_token(value_str(identity, "typeDef")?);
                *method_count_by_type.entry(td).or_insert(0) += 1;
            }
        } else if let Some(identity) = entry.get("method_identity") {
            let td = norm_token(value_str(identity, "typeDef")?);
            *method_count_by_type.entry(td).or_insert(0) += 1;
        }
    }
    let mut associated_type_count = 0u64;
    let mut nested_method_count = 0u64;
    let mut generated_method_association_count = 0u64;
    let mut explicit_nested_member_count = 0u64;
    let mut mapping = Vec::new();
    for catalog in catalogs {
        for record in value_array(catalog, "records")? {
            let record_object = record.get("record").unwrap_or(record);
            let record_id = value_str(record_object, "id")?;
            let selected_root = record_object
                .get("typeDef")
                .and_then(Value::as_str)
                .unwrap_or("");
            let associated = record
                .get("associated_types")
                .or_else(|| record_object.get("associated_types"))
                .and_then(Value::as_array);
            let Some(associated) = associated else {
                continue;
            };
            for entry in associated {
                let actual_td = norm_token(value_str(entry, "typeDef")?);
                let parent_td = entry
                    .get("parent_typeDef")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let count = method_count_by_type.get(&actual_td).copied().unwrap_or(0);
                let class = if record_id == "PEXEC_QUEUE" {
                    explicit_nested_member_count += count;
                    "EXPLICIT_NESTED_MEMBER"
                } else {
                    generated_method_association_count += count;
                    "GENERATED_ASSOCIATION"
                };
                associated_type_count += 1;
                nested_method_count += count;
                mapping.push(json!({
                    "record_id": record_id,
                    "selected_root_type": selected_root,
                    "actual_associated_type": actual_td,
                    "parent_type_def": parent_td,
                    "nested_method_count": count,
                    "association_class": class
                }));
            }
        }
    }
    Ok(json!({
        "associated_type_count": associated_type_count,
        "nested_method_count": nested_method_count,
        "generated_method_association_count": generated_method_association_count,
        "explicit_nested_member_count": explicit_nested_member_count,
        "queue_generated_count": 0,
        "additional_csharp_declarations": 0,
        "mapping": mapping
    }))
}

fn build_curated(
    build: &mut GraphBuild,
    config: &Config,
    input: &LoadedInput,
) -> RegistryResult<()> {
    let curated_root = root_path(config, CURATED_ROOT)?;
    let active = match crate::load_active_receipt(&curated_root) {
        Ok(Some(active)) => active,
        Ok(None) => {
            add_missing(
                &mut build.data.missing,
                "CURATED_ACTIVE_RECEIPT_ABSENT".to_string(),
                "CURATED_INPUT_ABSENT",
                Some(CURATED_ROOT),
                Some("metadata/imports.v5.json"),
                None,
                "active v5 receipt was not loaded",
                vec![],
            );
            return Ok(());
        }
        Err(error) => {
            return Err(RegistryError::new(
                "GRAPH_CURATED_INPUT_INVALID",
                format!("active curated receipt did not validate: {}", error.code),
            ));
        }
    };
    let v5 = parse_json_file(
        config,
        &input.artifacts,
        CURATED_ROOT,
        "metadata/imports.v5.json",
    )?;
    let imports = value_array(&v5, "imports")?;
    let mut entries = BTreeMap::new();
    for entry in imports {
        entries.insert(value_str(entry, "id")?.to_string(), entry.clone());
    }
    build.curated_files = active.files.len() as u64;
    for file in &active.files {
        let Some(entry) = entries.get(&file.id) else {
            add_missing(
                &mut build.data.missing,
                format!("CURATED_FILE:{}", file.id),
                "CURATED_LEDGER_ENTRY_MISSING",
                Some(CURATED_ROOT),
                Some(&file.imported.path),
                None,
                "active receipt file id is not present in v5 JSON",
                vec![],
            );
            continue;
        };
        let original_path = value_str(entry, "original_path")?;
        let original_size = value_u64(entry, "original_size")?;
        let original_sha = value_str(entry, "original_sha256")?;
        let imported_path = value_str(entry, "imported_path")?;
        let imported_size = value_u64(entry, "imported_size")?;
        let imported_sha = value_str(entry, "imported_sha256")?;
        let original = require_artifact_with_hash(
            &input.artifacts,
            CANONICAL_ROOT,
            original_path,
            original_size,
            original_sha,
        )?;
        let imported = require_artifact_with_hash(
            &input.artifacts,
            CURATED_ROOT,
            imported_path,
            imported_size,
            imported_sha,
        )?;
        build
            .curated_original_paths
            .insert(original_path.to_string());
        let mut status_by_method = BTreeMap::new();
        if let Some(statuses) = entry.get("methods").and_then(Value::as_array) {
            for status in statuses {
                let identity = status.get("method_identity").ok_or_else(|| {
                    RegistryError::new(
                        "GRAPH_CURATED_INVALID",
                        "curated method identity is missing",
                    )
                })?;
                let td = value_str(identity, "typeDef")?;
                let md = value_str(identity, "methodDef")?;
                let evidence = value_str(&status, "evidence_status")?;
                let source_status = value_str(&status, "source_status")?;
                status_by_method.insert(
                    format!("{}/{}", norm_token(td), norm_token(md)),
                    (evidence.to_string(), source_status.to_string()),
                );
                *build
                    .curated_status_counts
                    .entry(evidence.to_string())
                    .or_insert(0) += 1;
                build
                    .curated_status_by_method
                    .insert(md.to_lowercase(), evidence.to_string());
            }
        } else if let Some(identity) = entry.get("method_identity") {
            let td = value_str(identity, "typeDef")?;
            let md = value_str(identity, "methodDef")?;
            // Singular Event entries do not carry a per-method status field.
            // The accepted source03 receipt records 45 DIRECT_CIL_FULL methods
            // across the 13 files; these three singular direct-CIL bodies supply
            // the three that are not present in the mapped arrays.
            let evidence = "DIRECT_CIL_FULL".to_string();
            let source_status = "RECONSTRUCTED_FROM_DIRECT_CIL".to_string();
            status_by_method.insert(
                format!("{}/{}", norm_token(td), norm_token(md)),
                (evidence.clone(), source_status),
            );
            *build
                .curated_status_counts
                .entry(evidence.clone())
                .or_insert(0) += 1;
            build
                .curated_status_by_method
                .insert(md.to_lowercase(), evidence);
        }
        for method in &file.methods {
            let td = &method.type_def;
            let md = &method.method_def;
            let (evidence, source_status) = status_by_method
                .get(&format!("{}/{}", norm_token(td), norm_token(md)))
                .cloned()
                .unwrap_or_else(|| ("UNKNOWN".to_string(), "UNKNOWN".to_string()));
            let key = MethodKey {
                assembly_sha256: norm_token(&method.full_authority_sha256),
                mvid: norm_token(&method.mvid),
                type_token: norm_token(td),
                method_token: norm_token(md),
            };
            for (kind, root_id, artifact, kind_label) in [
                (
                    "CURATED_ORIGINAL",
                    CANONICAL_ROOT,
                    original.clone(),
                    "CURATED_ORIGINAL",
                ),
                (
                    "CURATED_IMPORTED",
                    CURATED_ROOT,
                    imported.clone(),
                    "CURATED_IMPORTED",
                ),
            ] {
                let rep = Representation {
                    representation_id: format!(
                        "{kind}:{}:{}:{}",
                        file.id,
                        norm_token(td),
                        norm_token(md)
                    ),
                    representation_kind: kind_label.to_string(),
                    evidence: vec![evidence.clone()],
                    coverage: Some("CURATED_IMPORTED_METHOD".to_string()),
                    source_status: Some(source_status.clone()),
                    source_ref: Some(artifact.clone()),
                    raw_ref: None,
                    semantic_ref: None,
                    semantic_range: None,
                    proof_ref: Some(artifact.clone()),
                    proof_row: None,
                    raw_row: None,
                    source_range: None,
                    body_sha256: None,
                    body_hash_verified: None,
                    table_heading: None,
                    table_header: None,
                    note: Some(format!(
                        "frozen v5 import id {}; root_id={root_id}",
                        file.id
                    )),
                };
                add_method_rep(&mut build.data.methods, &key, rep, "CURATED_13_FILES", None)?;
                add_link(
                    &mut build.data.links,
                    &artifact,
                    kind,
                    Some(&key.id()),
                    "CURATED_13_FILES",
                    Some(&file.id),
                );
            }
            build.curated_methods += 1;
            if evidence == "METADATA_ONLY" && md.eq_ignore_ascii_case("0x0601C913") {
                build.curated_abstract += 1;
                build.abstract_method_key = Some(key.clone());
            }
        }
    }
    let mut catalogs = Vec::new();
    for path in [
        "metadata/performance-source.v1.json",
        "metadata/scheduling-source.v1.json",
        "metadata/task-execution-source.v1.json",
    ] {
        if let Ok(value) = parse_json_file(config, &input.artifacts, CURATED_ROOT, path) {
            catalogs.push(value);
        }
    }
    let association_summary = association_summary(&v5, &catalogs)?;
    build.data.relationships.insert(
        "CURATED_GENERATED_ASSOCIATIONS".to_string(),
        RelationshipRow {
            relationship_id: "CURATED_GENERATED_ASSOCIATIONS".to_string(),
            kind: "CURATED_GENERATED_ASSOCIATIONS_NOT_DECLARATIONS".to_string(),
            status: "RETAINED_AS_ASSOCIATIONS".to_string(),
            subjects: vec!["curated_catalogs".to_string()],
            details: association_summary.clone(),
        },
    );
    add_cohort(
        &mut build.data.cohorts,
        "CURATED_ASSOCIATIONS_9_17",
        "CURATED_ASSOCIATED_TYPES",
        9,
        17,
        Vec::new(),
        vec![
            "9 associated TypeDefs / 17 nested Method identities; 11 generated associations + 6 explicit Queue.Enumerator members; not additional independent C# declarations"
                .to_string(),
        ],
    );
    build.association_summary = association_summary;
    Ok(())
}

fn build_gameplay(
    build: &mut GraphBuild,
    config: &Config,
    input: &LoadedInput,
) -> RegistryResult<()> {
    let map_path = format!("{GAMEPLAY_PREFIX}METHOD_EVIDENCE_MAP.md");
    let (map_ref, bytes) =
        read_artifact_bytes(config, &input.artifacts, CANONICAL_ROOT, &map_path)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| RegistryError::new("GRAPH_GAMEPLAY_NOT_UTF8", "gameplay map is not UTF-8"))?;
    let basenames = basename_map(&input.artifacts);
    let table_rows = parse_gameplay_tables(text);
    let mut seen = BTreeMap::<String, GameplayRecord>::new();
    let mut numbers: BTreeMap<u64, Vec<String>> = BTreeMap::new();
    for row in &table_rows {
        let Some(number) = row.cells.first().and_then(|cell| parse_decimal_cell(cell)) else {
            continue;
        };
        if !(1..=301).contains(&number) {
            continue;
        }
        let Some((td, md)) = row.cells.iter().find_map(|cell| identity_pair(cell)) else {
            continue;
        };
        let identity = format!("{td}/{md}");
        numbers.entry(number).or_default().push(identity.clone());
        if seen.contains_key(&identity) {
            continue;
        }
        let key = MethodKey {
            assembly_sha256: norm_token(&build.authority_assembly),
            mvid: norm_token(&build.authority_mvid),
            type_token: td.clone(),
            method_token: md.clone(),
        };
        let (canonical_ref, canonical_range, semantic_ref, semantic_range, resolution) =
            resolve_gameplay_source(row, &input.artifacts, &basenames)?;
        let source_ref = canonical_ref.clone().or(semantic_ref.clone());
        let body_source = semantic_ref.clone();
        let body_range = semantic_range.clone();
        let coverage = coverage_from_cells(&row.cells);
        let header = row.header.join(" | ");
        let body_hash = row
            .header
            .iter()
            .position(|header| header_has_body_sha(header))
            .and_then(|index| row.cells.get(index))
            .map(|cell| cell.trim().trim_matches('`').to_string())
            .filter(|cell| is_lower_hex_64(cell));
        let anchor_class = gameplay_anchor_class(
            semantic_ref.is_some(),
            semantic_range.as_ref(),
            body_hash.as_deref(),
        );
        if anchor_class == "UNRESOLVED_ANCHOR" {
            add_missing(
                &mut build.data.missing,
                format!("GAMEPLAY_ANCHOR:{number}"),
                "GAMEPLAY_SEMANTIC_ANCHOR_UNRESOLVED",
                Some(CANONICAL_ROOT),
                Some(&map_path),
                Some(number.to_string()),
                "ordinary row has no resolved semantic source anchor",
                vec![format!("physical_row={}", row.physical_line)],
            );
        }
        if anchor_class == "BODY_HASH_WITHOUT_SEMANTIC_RANGE" {
            build.body_failed += 1;
            add_missing(
                &mut build.data.missing,
                format!("GAMEPLAY_BODY_RANGE:{number}"),
                "GAMEPLAY_BODY_HASH_WITHOUT_SEMANTIC_RANGE",
                Some(CANONICAL_ROOT),
                body_source.as_ref().map(|artifact| artifact.path.as_str()),
                Some(body_hash.clone().unwrap_or_default()),
                "dedicated body hash without an explicit semantic source anchor and two-end range",
                vec![format!("physical_row={}", row.physical_line)],
            );
        }
        let record = GameplayRecord {
            number,
            key: key.clone(),
            source_path: semantic_ref
                .as_ref()
                .or(canonical_ref.as_ref())
                .map(|artifact| artifact.path.clone()),
            range: body_range.clone(),
            body_sha256: body_hash.clone(),
            coverage: coverage.clone(),
            body_anchor_class: anchor_class.to_string(),
            physical_line: row.physical_line as u64,
        };
        *build
            .body_anchor_classes
            .entry(record.body_anchor_class.clone())
            .or_insert(0) += 1;
        let mut body_verified = None;
        if let Some(body_hash) = &body_hash {
            build.body_expected += 1;
            if anchor_class == "EXPLICIT_RANGE_WITH_BODY_SHA" {
                if let (Some(source), Some(range)) = (&body_source, &body_range) {
                    let (_, source_bytes) = read_artifact_bytes(
                        config,
                        &input.artifacts,
                        CANONICAL_ROOT,
                        &source.path,
                    )?;
                    let computed = normalized_slice_hash(&source_bytes, range)?;
                    if &computed == body_hash {
                        build.body_verified += 1;
                        body_verified = Some(true);
                    } else {
                        build.body_failed += 1;
                        body_verified = Some(false);
                        add_missing(
                            &mut build.data.missing,
                            format!("GAMEPLAY_BODY_HASH:{number}"),
                            "GAMEPLAY_BODY_HASH_MISMATCH",
                            Some(CANONICAL_ROOT),
                            Some(&source.path),
                            Some(body_hash.clone()),
                            "accepted map body hash does not match the UTF-8 LF-normalized source slice",
                            vec![format!("computed={computed}")],
                        );
                    }
                }
            } else if anchor_class != "BODY_HASH_WITHOUT_SEMANTIC_RANGE" {
                build.body_failed += 1;
                add_missing(
                    &mut build.data.missing,
                    format!("GAMEPLAY_BODY_RANGE:{number}"),
                    "GAMEPLAY_BODY_RANGE_UNRESOLVED",
                    Some(CANONICAL_ROOT),
                    body_source.as_ref().map(|artifact| artifact.path.as_str()),
                    Some(body_hash.clone()),
                    "body hash has no explicit semantic source range",
                    vec![map_path.clone()],
                );
            }
        }
        let rep = Representation {
            representation_id: format!("GAMEPLAY_MAP:{number}"),
            representation_kind: "GAMEPLAY_SEMANTIC_OVERLAY".to_string(),
            evidence: vec!["SEMANTIC_OVERLAY_ROW".to_string(), coverage.clone()],
            coverage: Some(coverage),
            source_status: None,
            source_ref: source_ref.clone(),
            raw_ref: None,
            semantic_ref: semantic_ref.clone(),
            semantic_range: semantic_range.clone(),
            proof_ref: Some(map_ref.clone()),
            proof_row: Some(row.physical_line as u64),
            raw_row: None,
            source_range: canonical_range.clone(),
            body_sha256: body_hash.clone(),
            body_hash_verified: body_verified,
            table_heading: Some(row.heading.clone()),
            table_header: Some(header),
            note: Some(format!(
                "row={number}; body_anchor_class={anchor_class}; source_resolution={}",
                resolution.unwrap_or_else(|| "UNKNOWN".to_string())
            )),
        };
        let matched = build.data.methods.contains_key(&key.id());
        add_method_rep(
            &mut build.data.methods,
            &key,
            rep,
            "GAMEPLAY_ACCEPTED_301",
            None,
        )?;
        if matched {
            build.gameplay_matched += 1;
        }
        let mut curated_source = false;
        for (source, role) in [
            (canonical_ref.as_ref(), "GAMEPLAY_CANONICAL_SOURCE"),
            (semantic_ref.as_ref(), "GAMEPLAY_SEMANTIC_SOURCE"),
        ] {
            if let Some(source) = source {
                add_link(
                    &mut build.data.links,
                    source,
                    role,
                    Some(&key.id()),
                    "GAMEPLAY_ACCEPTED_301",
                    Some(&format!("row {number}")),
                );
                build.gameplay_source_files.insert(source.path.clone());
                if build.curated_original_paths.contains(&source.path) {
                    curated_source = true;
                }
            }
        }
        if curated_source {
            if let Some(source) = semantic_ref.as_ref().or(canonical_ref.as_ref()) {
                build.gameplay_curated_files.insert(source.path.clone());
            }
            build.gameplay_curated_rows += 1;
        }
        if let Some(primary) = semantic_ref.as_ref().or(canonical_ref.as_ref()) {
            build.gameplay_primary_sources.insert(primary.path.clone());
        }
        build.gameplay_rows += 1;
        seen.insert(identity, record.clone());
        if (288..=295).contains(&number) {
            build.r38_gameplay.push(record);
        }
    }
    for number in 1..=301u64 {
        let Some(identities) = numbers.get(&number) else {
            add_missing(
                &mut build.data.missing,
                format!("GAMEPLAY_ROW_MISSING:{number}"),
                "GAMEPLAY_ROW_MISSING",
                Some(CANONICAL_ROOT),
                Some(&map_path),
                Some(number.to_string()),
                "ordinary row number is absent after pair filtering",
                vec![],
            );
            continue;
        };
        if identities.len() != 1 {
            add_missing(
                &mut build.data.missing,
                format!("GAMEPLAY_ROW_AMBIGUOUS:{number}"),
                "GAMEPLAY_ROW_AMBIGUOUS",
                Some(CANONICAL_ROOT),
                Some(&map_path),
                Some(number.to_string()),
                "ordinary row number has more than one accepted TypeDef/MethodDef pair",
                identities.clone(),
            );
        }
    }
    let mut distinct_numbers = BTreeSet::new();
    for record in seen.values() {
        distinct_numbers.insert(record.number);
    }
    if distinct_numbers.len() != 301 {
        add_missing(
            &mut build.data.missing,
            "GAMEPLAY_301_COUNT_GAP".to_string(),
            "GAMEPLAY_ROW_COUNT_GAP",
            Some(CANONICAL_ROOT),
            Some(&map_path),
            Some("301".to_string()),
            "pair-filtered unique ordinary rows do not equal 301",
            vec![format!("actual={}", distinct_numbers.len())],
        );
    }
    add_cohort(
        &mut build.data.cohorts,
        "GAMEPLAY_ACCEPTED_68_FILES_301_ROWS",
        "GAMEPLAY_ACCEPTED",
        build.gameplay_primary_sources.len() as u64,
        build.gameplay_rows,
        build.gameplay_primary_sources.iter().cloned().collect(),
        vec![format!(
            "curated_files={} curated_rows={} noncurated_files={} noncurated_rows={} linked_source_artifacts={}",
            build.gameplay_curated_files.len(),
            build.gameplay_curated_rows,
            build
                .gameplay_primary_sources
                .len()
                .saturating_sub(build.gameplay_curated_files.len()),
            build
                .gameplay_rows
                .saturating_sub(build.gameplay_curated_rows),
            build.gameplay_source_files.len()
        )],
    );
    Ok(())
}

fn build_r38(build: &mut GraphBuild) -> RegistryResult<()> {
    let mut entries = Vec::new();
    for record in &build.r38_gameplay {
        let md = record.key.method_token.to_lowercase();
        let frozen = build
            .curated_status_by_method
            .get(&md)
            .cloned()
            .unwrap_or_else(|| "NOT_IN_FROZEN_V5".to_string());
        entries.push(json!({
            "row": record.number,
            "type_token": record.key.type_token,
            "method_token": record.key.method_token,
            "curated_v5_evidence_status": frozen,
            "gameplay_map_coverage": record.coverage,
            "gameplay_map_body_sha256": record.body_sha256,
            "gameplay_map_source": record.source_path,
            "gameplay_map_range": record.range,
            "gameplay_map_anchor_class": record.body_anchor_class,
            "map_physical_line": record.physical_line,
            "proof_refs": [
                format!("METHOD_EVIDENCE_MAP.md line {}", record.physical_line),
                "metadata/imports.v5.json".to_string()
            ]
        }));
    }
    build.r38_rows = entries.len() as u64;
    add_relationship(
        &mut build.data.relationships,
        "R38_COVERAGE_DIFF".to_string(),
        "ACCEPTED_RESEARCH_COVERAGE_DIFFERS_FROM_FROZEN_IMPORT_VIEW",
        "EXPLICIT_RELATIONSHIP_RECORDED",
        entries
            .iter()
            .filter_map(|entry| entry.get("method_token").and_then(Value::as_str))
            .map(str::to_string)
            .collect(),
        json!({
            "entries": entries,
            "promotion": "NONE",
            "limits": "accepted research coverage differs from the frozen import view; v5 and migration were not rewritten"
        }),
    );
    Ok(())
}

fn build_performance(
    build: &mut GraphBuild,
    config: &Config,
    input: &LoadedInput,
) -> RegistryResult<()> {
    let mut performance = input
        .artifacts
        .values()
        .filter(|row| is_performance_source(&row.path))
        .cloned()
        .collect::<Vec<_>>();
    performance.sort_by(|left, right| left.path.cmp(&right.path));
    build.performance_files = performance.len() as u64;
    let performance_paths = performance
        .iter()
        .map(|row| row.path.clone())
        .collect::<BTreeSet<_>>();
    let method_index = method_index_by_md(&build.data.methods);
    let mut exact = BTreeSet::new();
    let mut inferred = BTreeSet::new();
    let mut expected_exact = BTreeSet::new();
    let mut curated_paths = BTreeSet::new();
    for path in &build.curated_original_paths {
        if path.starts_with(PERFORMANCE_PREFIX) && path.ends_with(".cs") {
            curated_paths.insert(path.clone());
            exact.insert(path.clone());
            expected_exact.insert(path.clone());
            build.performance_curated += 1;
        }
    }

    let (migration_ref, migration) = read_json_artifact(
        config,
        &input.artifacts,
        CURATED_ROOT,
        "metadata/migration.json",
    )?;
    if let Some(records) = migration.get("records").and_then(Value::as_array) {
        for record in records {
            let Some(path) = record.get("original_path").and_then(Value::as_str) else {
                continue;
            };
            if !performance_paths.contains(path) {
                continue;
            }
            exact.insert(path.to_string());
            expected_exact.insert(path.to_string());
            if curated_paths.contains(path) {
                continue;
            }
            let Some(methods) = record.pointer("/methods/current").and_then(Value::as_array) else {
                continue;
            };
            for method in methods {
                let Some(identity) = method.get("symbol_key") else {
                    continue;
                };
                let key = MethodKey {
                    assembly_sha256: norm_token(value_str(identity, "full_authority_sha256")?),
                    mvid: norm_token(value_str(identity, "mvid")?),
                    type_token: norm_token(value_str(identity, "typeDef")?),
                    method_token: norm_token(value_str(identity, "methodDef")?),
                };
                add_performance_rep(
                    build,
                    &key,
                    &require_artifact(&input.artifacts, CANONICAL_ROOT, path)?,
                    &migration_ref,
                    "EXACT_METHOD_MAP",
                    vec![
                        "MIGRATION_METHOD_RECORD".to_string(),
                        value_str(method, "evidence_status")
                            .unwrap_or("UNKNOWN")
                            .to_string(),
                    ],
                    Some(format!(
                        "coverage={}",
                        method
                            .get("coverage")
                            .and_then(Value::as_str)
                            .unwrap_or("UNKNOWN")
                    )),
                    &format!("metadata/migration.json record method; file={path}"),
                )?;
            }
        }
    }

    let (p02_ref, p02) = read_json_artifact(
        config,
        &input.artifacts,
        CANONICAL_ROOT,
        "evidence/performance/PERF02_EVIDENCE.json",
    )?;
    let mut p02_exact = BTreeSet::new();
    if let Some(members) = p02
        .pointer("/projection_method_mapping/members")
        .and_then(Value::as_array)
    {
        for member in members {
            let Some(projection) = member.get("projection").and_then(Value::as_str) else {
                continue;
            };
            let target = if projection == "SubLevelController.cs" {
                Some(format!("{PERFORMANCE_PREFIX}SubLevelController.cs"))
            } else {
                performance_path_for_basename(&performance_paths, projection, true)
            };
            let Some(target) = target else {
                continue;
            };
            if !performance_paths.contains(&target) {
                continue;
            }
            if matches!(
                projection,
                "AsyncTask.cs" | "Completion.cs" | "SubLevelController.cs"
            ) {
                p02_exact.insert(target.clone());
            }
            if curated_paths.contains(&target) {
                exact.insert(target);
                continue;
            }
            exact.insert(target.clone());
            let Some(token) = member.get("method_token").and_then(Value::as_str) else {
                continue;
            };
            let Some(key) = method_key_for_token(&method_index, token) else {
                build.performance_unmapped += 1;
                add_missing(
                    &mut build.data.missing,
                    format!("PERFORMANCE_PERF02_UNMAPPED:{projection}:{token}"),
                    "PERFORMANCE_METHOD_LINK_UNMAPPED",
                    Some(CANONICAL_ROOT),
                    Some(&target),
                    Some(token.to_string()),
                    "PERF02 method token does not resolve to a unique qualified key under the accepted authority",
                    vec!["evidence/performance/PERF02_EVIDENCE.json".to_string()],
                );
                continue;
            };
            add_performance_rep(
                build,
                &key,
                &require_artifact(&input.artifacts, CANONICAL_ROOT, &target)?,
                &p02_ref,
                "EXACT_METHOD_MAP",
                vec![
                    "PERF02_PROJECTION_METHOD".to_string(),
                    member
                        .get("source")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                        .to_string(),
                    member
                        .get("direct_il")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                        .to_string(),
                ],
                Some(format!(
                    "source={}; direct_il={}",
                    member
                        .get("source")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN"),
                    member
                        .get("direct_il")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                )),
                &format!(
                    "PERF02 projection {projection}; file={target}; proof member={}; raw_reference={}",
                    member
                        .get("member")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN"),
                    member
                        .get("raw_reference")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                ),
            )?;
        }
    }
    expected_exact.extend(p02_exact);

    let (p04_ref, p04) = read_json_artifact(
        config,
        &input.artifacts,
        CANONICAL_ROOT,
        "evidence/performance/PERF04_EVIDENCE.json",
    )?;
    let mut p04_exact = BTreeSet::new();
    if let Some(paths) = p04.get("source_files_created").and_then(Value::as_array) {
        for path in paths.iter().filter_map(Value::as_str) {
            if performance_paths.contains(path)
                && matches!(
                    short_name(path).as_deref(),
                    Some("LoadMapController.cs")
                        | Some("SubLevelController.cs")
                        | Some("WorldGlobal.cs")
                        | Some("SubLevel.cs")
                )
            {
                p04_exact.insert(path.to_string());
                exact.insert(path.to_string());
            }
        }
    }
    if let Some(methods) = p04.get("method_inventory").and_then(Value::as_array) {
        for method in methods {
            let Some(projection) = method.get("projection").and_then(Value::as_str) else {
                continue;
            };
            if !performance_paths.contains(projection)
                || !matches!(
                    short_name(projection).as_deref(),
                    Some("LoadMapController.cs")
                        | Some("SubLevelController.cs")
                        | Some("WorldGlobal.cs")
                        | Some("SubLevel.cs")
                )
            {
                continue;
            }
            p04_exact.insert(projection.to_string());
            exact.insert(projection.to_string());
            let Some(token) = method.get("method_token").and_then(Value::as_str) else {
                continue;
            };
            let Some(key) = method_key_for_token(&method_index, token) else {
                build.performance_unmapped += 1;
                add_missing(
                    &mut build.data.missing,
                    format!("PERFORMANCE_PERF04_UNMAPPED:{projection}:{token}"),
                    "PERFORMANCE_METHOD_LINK_UNMAPPED",
                    Some(CANONICAL_ROOT),
                    Some(projection),
                    Some(token.to_string()),
                    "PERF04 method token does not resolve to a unique qualified key under the accepted authority",
                    vec!["evidence/performance/PERF04_EVIDENCE.json".to_string()],
                );
                continue;
            };
            add_performance_rep(
                build,
                &key,
                &require_artifact(&input.artifacts, CANONICAL_ROOT, projection)?,
                &p04_ref,
                "EXACT_METHOD_MAP",
                vec![
                    "PERF04_METHOD_INVENTORY".to_string(),
                    method
                        .get("capture")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                        .to_string(),
                    method
                        .get("body_status")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                        .to_string(),
                ],
                Some(format!(
                    "capture={}; body_status={}",
                    method
                        .get("capture")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN"),
                    method
                        .get("body_status")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                )),
                &format!(
                    "PERF04 method_inventory; file={projection}; proof id={}; raw_reference={}",
                    method
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN"),
                    method
                        .get("raw_reference")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                ),
            )?;
        }
    }
    expected_exact.extend(p04_exact);

    let (p04b_ref, p04b) = read_json_artifact(
        config,
        &input.artifacts,
        CANONICAL_ROOT,
        "evidence/performance/PERF04B_EVIDENCE.json",
    )?;
    if let Some(path) = p04b
        .pointer("/source_reconstruction/path")
        .and_then(Value::as_str)
    {
        if performance_paths.contains(path) {
            expected_exact.insert(path.to_string());
            exact.insert(path.to_string());
            if let Some(tokens) = p04b
                .pointer("/source_reconstruction/source_body_tokens")
                .and_then(Value::as_array)
            {
                for token in tokens.iter().filter_map(Value::as_str) {
                    let Some(key) = method_key_for_token(&method_index, token) else {
                        build.performance_unmapped += 1;
                        add_missing(
                            &mut build.data.missing,
                            format!("PERFORMANCE_PERF04B_UNMAPPED:{path}:{token}"),
                            "PERFORMANCE_METHOD_LINK_UNMAPPED",
                            Some(CANONICAL_ROOT),
                            Some(path),
                            Some(token.to_string()),
                            "PERF04B source_body_token does not resolve to a unique qualified key",
                            vec!["evidence/performance/PERF04B_EVIDENCE.json".to_string()],
                        );
                        continue;
                    };
                    add_performance_rep(
                        build,
                        &key,
                        &require_artifact(&input.artifacts, CANONICAL_ROOT, path)?,
                        &p04b_ref,
                        "EXACT_METHOD_MAP",
                        vec![
                            "PERF04B_SOURCE_BODY_TOKEN".to_string(),
                            "COMPLETE_DIRECT_CIL_METHOD_COUNT_9".to_string(),
                        ],
                        Some("source_reconstruction; 9 source_body_tokens".to_string()),
                        &format!(
                            "PERF04B source_reconstruction; file={path}; method_token={token}"
                        ),
                    )?;
                }
            }
        }
    }

    let (p09_ref, p09) = read_json_artifact(
        config,
        &input.artifacts,
        CANONICAL_ROOT,
        "evidence/performance/PERF09_EVIDENCE.json",
    )?;
    if let Some(path) = p09
        .pointer("/source_checks/candidate_source/path")
        .and_then(Value::as_str)
    {
        if performance_paths.contains(path) {
            expected_exact.insert(path.to_string());
            exact.insert(path.to_string());
            if let Some(checks) = p09
                .pointer("/source_checks/method_checks")
                .and_then(Value::as_array)
            {
                for check in checks {
                    let Some(token) = check.get("method_token").and_then(Value::as_str) else {
                        continue;
                    };
                    let Some(key) = method_key_for_token(&method_index, token) else {
                        build.performance_unmapped += 1;
                        add_missing(
                            &mut build.data.missing,
                            format!("PERFORMANCE_PERF09_UNMAPPED:{path}:{token}"),
                            "PERFORMANCE_METHOD_LINK_UNMAPPED",
                            Some(CANONICAL_ROOT),
                            Some(path),
                            Some(token.to_string()),
                            "PERF09 method_check token does not resolve to a unique qualified key",
                            vec!["evidence/performance/PERF09_EVIDENCE.json".to_string()],
                        );
                        continue;
                    };
                    add_performance_rep(
                        build,
                        &key,
                        &require_artifact(&input.artifacts, CANONICAL_ROOT, path)?,
                        &p09_ref,
                        "EXACT_METHOD_MAP",
                        vec![
                            "PERF09_METHOD_CHECK".to_string(),
                            format!(
                                "DIRECT_CIL_COMPLETE={}",
                                check
                                    .get("direct_cil_complete")
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false)
                            ),
                        ],
                        Some("13 selected partial-file method checks".to_string()),
                        &format!(
                            "PERF09 source_checks; file={path}; direct_cil_section={}",
                            check
                                .get("direct_cil_section")
                                .and_then(Value::as_str)
                                .unwrap_or("UNKNOWN")
                        ),
                    )?;
                }
            }
        }
    }

    let (p05_ref, p05) = read_json_artifact(
        config,
        &input.artifacts,
        CANONICAL_ROOT,
        "evidence/performance/PERF05_EVIDENCE.json",
    )?;
    if let Some(fragments) = p05.get("source_fragments").and_then(Value::as_object) {
        for path in fragments.keys() {
            if performance_paths.contains(path) {
                inferred.insert(path.clone());
            }
        }
    }
    if let Some(methods) = p05.get("method_inventory").and_then(Value::as_array) {
        for method in methods {
            let Some(method_type) = method.get("type").and_then(Value::as_str) else {
                continue;
            };
            let Some(path) = performance_path_for_basename(
                &performance_paths,
                &format!("{method_type}.cs"),
                false,
            ) else {
                continue;
            };
            if curated_paths.contains(&path) {
                continue;
            }
            let Some(token) = method.get("method_token").and_then(Value::as_str) else {
                continue;
            };
            let Some(key) = method_key_for_token(&method_index, token) else {
                build.performance_unmapped += 1;
                add_missing(
                    &mut build.data.missing,
                    format!("PERFORMANCE_PERF05_UNMAPPED:{path}:{token}"),
                    "PERFORMANCE_METHOD_LINK_UNMAPPED",
                    Some(CANONICAL_ROOT),
                    Some(&path),
                    Some(token.to_string()),
                    "PERF05 inferred method token does not resolve to a unique qualified key",
                    vec!["evidence/performance/PERF05_EVIDENCE.json".to_string()],
                );
                continue;
            };
            add_performance_rep(
                build,
                &key,
                &require_artifact(&input.artifacts, CANONICAL_ROOT, &path)?,
                &p05_ref,
                "TYPE_NAME_INFERRED",
                vec![
                    "PERF05_TYPE_NAME_INFERRED".to_string(),
                    method
                        .get("source")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                        .to_string(),
                    method
                        .get("direct_il")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                        .to_string(),
                ],
                Some(
                    "type/name inferred; file association not an explicit per-file map".to_string(),
                ),
                &format!("PERF05 method_inventory type={method_type}; file={path}"),
            )?;
        }
    }

    let (p07_ref, p07) = read_json_artifact(
        config,
        &input.artifacts,
        CANONICAL_ROOT,
        "evidence/performance/PERF07_EVIDENCE.json",
    )?;
    if let Some(fragments) = p07.get("source_fragments").and_then(Value::as_object) {
        for path in fragments.keys() {
            if performance_paths.contains(path) {
                inferred.insert(path.clone());
            }
        }
    }
    if let Some(path) = performance_path_for_basename(&performance_paths, "Time.cs", false) {
        if let Some(methods) = p07
            .get("selected_method_inventory")
            .and_then(Value::as_array)
        {
            for method in methods {
                let Some(token) = method.get("method_token").and_then(Value::as_str) else {
                    continue;
                };
                let Some(key) = method_key_for_token(&method_index, token) else {
                    build.performance_unmapped += 1;
                    add_missing(
                        &mut build.data.missing,
                        format!("PERFORMANCE_PERF07_UNMAPPED:{path}:{token}"),
                        "PERFORMANCE_METHOD_LINK_UNMAPPED",
                        Some(CANONICAL_ROOT),
                        Some(&path),
                        Some(token.to_string()),
                        "PERF07 inferred method token does not resolve to a unique qualified key",
                        vec!["evidence/performance/PERF07_EVIDENCE.json".to_string()],
                    );
                    continue;
                };
                add_performance_rep(
                    build,
                    &key,
                    &require_artifact(&input.artifacts, CANONICAL_ROOT, &path)?,
                    &p07_ref,
                    "TYPE_NAME_INFERRED",
                    vec!["PERF07_TYPE_NAME_INFERRED".to_string()],
                    Some("single Time source fragment; selected method inventory".to_string()),
                    &format!("PERF07 Time.cs inferred association; method_token={token}"),
                )?;
            }
        }
    }

    let (p08_ref, p08) = read_json_artifact(
        config,
        &input.artifacts,
        CANONICAL_ROOT,
        "evidence/performance/PERF08_EVIDENCE.json",
    )?;
    if let Some(fragments) = p08.get("source_fragments").and_then(Value::as_object) {
        for path in fragments.keys() {
            if performance_paths.contains(path) {
                inferred.insert(path.clone());
            }
        }
    }
    if let Some(methods) = p08.get("method_inventory").and_then(Value::as_array) {
        for method in methods {
            let Some(method_type) = method.get("type").and_then(Value::as_str) else {
                continue;
            };
            let Some(path) = performance_path_for_basename(
                &performance_paths,
                &format!("{method_type}.cs"),
                false,
            ) else {
                continue;
            };
            let Some(token) = method.get("method_token").and_then(Value::as_str) else {
                continue;
            };
            let Some(key) = method_key_for_token(&method_index, token) else {
                build.performance_unmapped += 1;
                add_missing(
                    &mut build.data.missing,
                    format!("PERFORMANCE_PERF08_UNMAPPED:{path}:{token}"),
                    "PERFORMANCE_METHOD_LINK_UNMAPPED",
                    Some(CANONICAL_ROOT),
                    Some(&path),
                    Some(token.to_string()),
                    "PERF08 inferred method token does not resolve to a unique qualified key",
                    vec!["evidence/performance/PERF08_EVIDENCE.json".to_string()],
                );
                continue;
            };
            add_performance_rep(
                build,
                &key,
                &require_artifact(&input.artifacts, CANONICAL_ROOT, &path)?,
                &p08_ref,
                "TYPE_NAME_INFERRED",
                vec![
                    "PERF08_TYPE_NAME_INFERRED".to_string(),
                    method
                        .get("source")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                        .to_string(),
                ],
                Some(
                    "type/name inferred; file association not an explicit per-file map".to_string(),
                ),
                &format!("PERF08 method_inventory type={method_type}; file={path}"),
            )?;
        }
    }

    let (p11_ref, p11) = read_json_artifact(
        config,
        &input.artifacts,
        CANONICAL_ROOT,
        "evidence/performance/PERF11_EVIDENCE.json",
    )?;
    let pending_events =
        "src/reconstructed/performance/managed/entity/RangeComponent.PendingEvents.cs";
    let pending_boundaries =
        "src/reconstructed/performance/managed/entity/PendingEventBoundaries.cs";
    if performance_paths.contains(pending_events) {
        inferred.insert(pending_events.to_string());
    }
    if let Some(methods) = p11.get("selected_methods").and_then(Value::as_array) {
        for method in methods {
            let Some(name) = method.get("name").and_then(Value::as_str) else {
                continue;
            };
            if !name.starts_with("RangeComponent.") {
                add_missing(
                    &mut build.data.missing,
                    format!("PERFORMANCE_PERF11_NAME_UNMAPPED:{name}"),
                    "PERF11_SELECTED_NAME_WITHOUT_FILE_MAP",
                    Some(CANONICAL_ROOT),
                    Some(pending_events),
                    Some(name.to_string()),
                    "selected PERF11 name has no explicit token-to-file field; source comment/declaration cannot license a file association",
                    vec!["evidence/performance/PERF11_EVIDENCE.json".to_string()],
                );
                continue;
            }
            let Some(token) = method.get("methoddef").and_then(Value::as_str) else {
                continue;
            };
            let Some(key) = method_key_for_token(&method_index, token) else {
                build.performance_unmapped += 1;
                add_missing(
                    &mut build.data.missing,
                    format!("PERFORMANCE_PERF11_UNMAPPED:{pending_events}:{token}"),
                    "PERFORMANCE_METHOD_LINK_UNMAPPED",
                    Some(CANONICAL_ROOT),
                    Some(pending_events),
                    Some(token.to_string()),
                    "PERF11 RangeComponent method token does not resolve to a unique qualified key",
                    vec!["evidence/performance/PERF11_EVIDENCE.json".to_string()],
                );
                continue;
            };
            add_performance_rep(
                build,
                &key,
                &require_artifact(&input.artifacts, CANONICAL_ROOT, pending_events)?,
                &p11_ref,
                "TYPE_NAME_INFERRED",
                vec![
                    "PERF11_RANGE_SELECTED_METHOD".to_string(),
                    method
                        .get("classification")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                        .to_string(),
                    method
                        .get("source_status")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    method
                        .get("direct_cil_status")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                ],
                Some(format!(
                    "classification={}; 6 RangeComponent-prefixed selected methods only",
                    method
                        .get("classification")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                )),
                &format!(
                    "PERF11 selected method {name}; file={pending_events}; association inferred; direct_cil_section={}",
                    method
                        .get("direct_cil_section")
                        .and_then(Value::as_str)
                        .unwrap_or("UNKNOWN")
                ),
            )?;
        }
    }

    inferred.remove(pending_boundaries);
    exact.remove(pending_boundaries);
    if expected_exact != exact {
        let missing = expected_exact
            .difference(&exact)
            .cloned()
            .collect::<Vec<_>>();
        let extra = exact
            .difference(&expected_exact)
            .cloned()
            .collect::<Vec<_>>();
        add_missing(
            &mut build.data.missing,
            "PERFORMANCE_EXACT_MEMBERSHIP_GAP".to_string(),
            "PERFORMANCE_EXACT_MEMBERSHIP_GAP",
            None,
            None,
            Some("22 exact file members".to_string()),
            "derived exact performance membership differs from the accepted explicit 22-file set",
            vec![format!("missing={missing:?}"), format!("extra={extra:?}")],
        );
    }
    for row in performance {
        let artifact = row_artifact(&row)?;
        let tier = if exact.contains(&row.path) {
            "EXACT_METHOD_MAP"
        } else if inferred.contains(&row.path) {
            "TYPE_NAME_INFERRED"
        } else if row.path == pending_boundaries {
            "FILE_RECEIPT_ONLY"
        } else {
            "DOC_ONLY_NO_JSON"
        };
        *build.performance_tiers.entry(tier.to_string()).or_insert(0) += 1;
        let curated = curated_paths.contains(&row.path);
        let cohort = if curated {
            "PERFORMANCE_11_CURATED"
        } else {
            build.performance_queue += 1;
            "PERFORMANCE_27_QUEUE"
        };
        let mut method_ids = Vec::new();
        for method in build.data.methods.values() {
            if method.representations.iter().any(|representation| {
                representation
                    .source_ref
                    .as_ref()
                    .map(|source| source.path == row.path)
                    .unwrap_or(false)
            }) {
                method_ids.push(method.method_id.clone());
            }
        }
        let link = build
            .data
            .links
            .entry(artifact_key(&artifact))
            .or_insert_with(|| ArtifactLinkRow {
                artifact: artifact.clone(),
                roles: Vec::new(),
                method_ids: Vec::new(),
                cohorts: Vec::new(),
                notes: Vec::new(),
            });
        push_unique_string(&mut link.roles, "PERFORMANCE_SOURCE");
        push_unique_string(&mut link.cohorts, cohort);
        push_unique_string(&mut link.cohorts, &format!("PERFORMANCE_TIER_{tier}"));
        for id in method_ids {
            push_unique_string(&mut link.method_ids, &id);
        }
        push_unique_string(
            &mut link.notes,
            &format!(
                "mapping_tier={tier}; queue={}; method_links={}",
                !curated,
                link.method_ids.len()
            ),
        );
        if row.path == pending_boundaries {
            add_missing(
                &mut build.data.missing,
                "PERFORMANCE_PENDING_BOUNDARIES_MAPPING".to_string(),
                "PERFORMANCE_PER_METHOD_MAPPING_UNRESOLVED",
                Some(CANONICAL_ROOT),
                Some(&row.path),
                None,
                "file receipt only; no per-method token-to-file field; do not assign all selected methods to this file",
                vec!["evidence/performance/PERF11_EVIDENCE.json".to_string()],
            );
        }
    }
    for path in &exact {
        let count = if let Ok(artifact) = require_artifact(&input.artifacts, CANONICAL_ROOT, path) {
            build
                .data
                .links
                .get(&artifact_key(&artifact))
                .map(|link| link.method_ids.len() as u64)
                .unwrap_or(0)
        } else {
            0
        };
        if count == 0 {
            add_missing(
                &mut build.data.missing,
                format!("PERFORMANCE_EXACT_FILE_NO_METHODS:{path}"),
                "PERFORMANCE_EXACT_FILE_NO_METHODS",
                Some(CANONICAL_ROOT),
                Some(path),
                None,
                "exact-tier performance file has no qualified method association",
                vec![],
            );
        }
    }
    build.performance_method_links = 0;
    for path in &performance_paths {
        if let Ok(artifact) = require_artifact(&input.artifacts, CANONICAL_ROOT, path) {
            if let Some(link) = build.data.links.get(&artifact_key(&artifact)) {
                build.performance_method_links += link.method_ids.len() as u64;
            }
        }
    }
    add_cohort(
        &mut build.data.cohorts,
        "PERFORMANCE_38",
        "PERFORMANCE",
        build.performance_files,
        0,
        performance_paths.iter().cloned().collect(),
        vec![
            format!("curated={}", build.performance_curated),
            format!("queue={}", build.performance_queue),
            format!("tiers={:?}", build.performance_tiers),
            format!("method_links={}", build.performance_method_links),
            format!("unmapped={}", build.performance_unmapped),
        ],
    );
    Ok(())
}

fn build_missing_triad(build: &mut GraphBuild, input: &LoadedInput) -> RegistryResult<()> {
    let triad = [
        (
            "src/reconstructed/performance/managed/entity/EventArgs.Invoke.Enum2.cs",
            "0x020000A9/0x06000458",
            vec![
                "docs/performance/PERF_CIL15_CONCRETE_INVOKE.md lines 9,18-19,242",
                "evidence/performance/PERF_CIL15_DIRECT_CIL.txt lines 30-34",
                "docs/performance/PERF_EVENT_DISPATCH_STATIC_CLOSURE_V1.md line 698",
            ],
        ),
        (
            "src/reconstructed/performance/managed/entity/EventSystem.EmitWithTarget.Enum2.cs",
            "0x0200706E/0x06045B73",
            vec![
                "docs/performance/PERF_CIL16_WRAPPER_RECOVERY.md lines 17-18,26,460",
                "evidence/performance/PERF_CIL16_DIRECT_CIL.txt lines 7-9",
                "docs/performance/PERF_EVENT_DISPATCH_STATIC_CLOSURE_V1.md line 699",
            ],
        ),
        (
            "src/reconstructed/performance/managed/entity/RangeComponent.LifecycleRecovery.cs",
            "0x020048E7/0x06030B43,0x06030BA4",
            vec![
                "docs/performance/PERF_CIL12_FAILURE_LIFECYCLE.md lines 4,18,112-117,123-128,416,836",
                "evidence/performance/PERF_CIL12_DIRECT_CIL.txt lines 16,28,32-38",
                "docs/performance/PERF_EVENT_DISPATCH_STATIC_CLOSURE_V1.md lines 459-463",
            ],
        ),
    ];
    for (path, identities, refs) in triad {
        let Some(row) = lookup_artifact(&input.artifacts, CANONICAL_ROOT, path) else {
            add_missing(
                &mut build.data.missing,
                format!("MISSING_TRIAD_ARTIFACT:{path}"),
                "MISSING_STRUCTURED_MAP_TRIAD",
                Some(CANONICAL_ROOT),
                Some(path),
                Some(identities.to_string()),
                "documented triad file is absent from the artifact registry",
                refs.iter().map(|value| value.to_string()).collect(),
            );
            continue;
        };
        let artifact = row_artifact(row)?;
        add_link(
            &mut build.data.links,
            &artifact,
            "DOC_REPORTED_METHOD_IDENTITY",
            None,
            "MISSING_STRUCTURED_MAP_TRIAD",
            Some(&format!(
                "reported identities: {identities}; structured map absent"
            )),
        );
        add_missing(
            &mut build.data.missing,
            format!("MISSING_TRIAD:{path}"),
            "MISSING_STRUCTURED_MAP_TRIAD",
            Some(CANONICAL_ROOT),
            Some(path),
            Some(identities.to_string()),
            "structured method map is absent; file and document proof refs are retained as DOC_REPORTED_LINK, not a manufactured qualified import",
            refs.iter().map(|value| value.to_string()).collect(),
        );
        build.missing_triad += 1;
    }
    Ok(())
}

fn build_non_managed(build: &mut GraphBuild, input: &LoadedInput) {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for row in input.artifacts.values() {
        let ext = Path::new(&row.path)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_lowercase();
        let label = match ext.as_str() {
            "ts" => "TS15",
            "js" => "JS6",
            "cpp" => "CPP4",
            "hpp" => "HPP9",
            _ => continue,
        };
        *counts.entry(label.to_string()).or_insert(0) += 1;
        if let Ok(artifact) = row_artifact(row) {
            add_link(
                &mut build.data.links,
                &artifact,
                "NON_MANAGED_IDENTITY_NOT_APPLICABLE",
                None,
                &format!("NON_MANAGED_{label}"),
                Some("non-managed; no CSharpScript method identity"),
            );
        }
    }
    build.non_managed_counts = counts.clone();
    for (label, count) in counts {
        add_cohort(
            &mut build.data.cohorts,
            &format!("NON_MANAGED_{label}"),
            "NON_MANAGED_OPAQUE_INVENTORY",
            count,
            0,
            Vec::new(),
            vec![
                "identity is not a CSharpScript method identity; retain artifact hash only"
                    .to_string(),
            ],
        );
    }
}

fn build_r54(build: &mut GraphBuild, input: &LoadedInput) -> RegistryResult<()> {
    let path = "src/reconstructed/managed/semantic/gameplay/StateMachineCommon.Enter.cs";
    if let Some(row) = lookup_artifact(&input.artifacts, CANONICAL_ROOT, path) {
        let artifact = row_artifact(row)?;
        add_link(
            &mut build.data.links,
            &artifact,
            "STAGED_NOT_ACCEPTED",
            None,
            "R54_STAGED",
            Some("69th staged file; map unchanged; no row 302"),
        );
        add_cohort(
            &mut build.data.cohorts,
            "R54_STAGED",
            "STAGED_NOT_ACCEPTED",
            1,
            0,
            vec![path.to_string()],
            vec!["not added to accepted 301; no row 302".to_string()],
        );
        build.r54_staged = 1;
    }
    Ok(())
}

fn build_duplicates(build: &mut GraphBuild, input: &LoadedInput) {
    let mut groups: BTreeMap<(u64, String), Vec<ArtifactRef>> = BTreeMap::new();
    for row in input.artifacts.values() {
        if row.status != "OK" {
            continue;
        }
        if let Ok(artifact) = row_artifact(row) {
            groups
                .entry((artifact.size, artifact.sha256.clone()))
                .or_default()
                .push(artifact);
        }
    }
    for ((size, sha), artifacts) in groups {
        if artifacts.len() < 2 {
            continue;
        }
        build.duplicate_groups += 1;
        build.duplicate_paths += artifacts.len() as u64;
        let mut paths = artifacts.iter().map(artifact_key).collect::<Vec<_>>();
        paths.sort();
        let id = format!(
            "DUP_BYTES:{}",
            crate::sha256_hex(format!("{}\0{}", size, sha).as_bytes())
        );
        add_relationship(
            &mut build.data.relationships,
            id,
            "DUPLICATE_EXACT_BYTES",
            "PRESERVED_ALL_PATHS",
            paths,
            json!({"size": size, "sha256": sha, "group_count": artifacts.len(), "nonsemantic_equality_only": true}),
        );
    }
    let snapshot_methods = build
        .data
        .methods
        .values()
        .filter(|method| method.representations.len() > 1)
        .cloned()
        .collect::<Vec<_>>();
    for method in snapshot_methods {
        let mut reps = method
            .representations
            .iter()
            .map(|rep| rep.representation_id.clone())
            .collect::<Vec<_>>();
        reps.sort();
        build.same_symbol_groups += 1;
        add_relationship(
            &mut build.data.relationships,
            format!("SAME_SYMBOL:{}", method.method_id),
            "SAME_QUALIFIED_SYMBOL_MULTIPLE_REPRESENTATIONS",
            "PRESERVED_ALL_REPRESENTATIONS",
            reps,
            json!({
                "assembly_sha256": method.assembly_sha256,
                "mvid": method.mvid,
                "type_token": method.type_token,
                "method_token": method.method_token,
                "evidence_levels": method.evidence_levels
            }),
        );
    }
}

fn build_range_relationships(build: &mut GraphBuild) -> RegistryResult<()> {
    let partial = [
        "RangeComponent.OnEnd",
        "RangeComponent.ClearComponent",
        "CSharpScript.Game.Common.Event.EventSystem.EmitWithTarget<T1>",
        "CSharpScript.Game.Common.Event.EventSystem.EmitWithTarget<T1,T2>",
    ];
    let full = [
        "RangeComponent.EmitPendingEvents",
        "RangeComponent.EmitMyPlayerOverlapEvent",
        "RangeComponent.<EmitMyPlayerOverlapEvent>b__154_0",
        "RangeComponent.InitEmitEventData",
        "TimerSystemInstance.Has",
        "TimerSystemInstance.Remove",
    ];
    add_relationship(
        &mut build.data.relationships,
        "RANGE_COMPONENT_CLOSURE".to_string(),
        "RANGE_COMPONENT_FULL_PARTIAL_LIFECYCLE_CLOSURE",
        "RETAINED_SEPARATE",
        vec![
            "PERF09".to_string(),
            "PERF11".to_string(),
            "PERF_CIL12".to_string(),
            "PERF_CIL16/17/18".to_string(),
        ],
        json!({
            "full_names": full,
            "partial_names": partial,
            "blocked_direct_cil_count": 2,
            "lifecycle_overlay": "RangeComponent.LifecycleRecovery.cs is a separate lifecycle recovery overlay; OnEnd/ClearComponent are partial excerpts, not full-class proof",
            "closure": "PERF_EVENT_DISPATCH_STATIC_CLOSURE_V1.md lines 725-744: no 0x42E body gap, helper/interface/runtime conditional; NO_END_TO_END_GUARANTEE; CALLBACK_POPULATION_UNKNOWN",
        }),
    );
    Ok(())
}

fn derive_graph(config_path: &Path, config: &Config, snapshot: &Path) -> RegistryResult<GraphData> {
    inspect_existing_file(config_path)?;
    let config_bytes = fs::read(config_path).map_err(|_| {
        RegistryError::new("GRAPH_CONFIG_READ_ERROR", "graph config could not be read")
    })?;
    let input = load_graph_input(config, snapshot)?;
    let authority = load_source_authority(config, &input)?;
    let tooling_overlay = tooling_overlay(config, &input)?;
    let mut build = GraphBuild {
        data: GraphData {
            methods: BTreeMap::new(),
            links: BTreeMap::new(),
            relationships: BTreeMap::new(),
            cohorts: BTreeMap::new(),
            missing: BTreeMap::new(),
            summary: GraphSummary {
                schema: GRAPH_SCHEMA.to_string(),
                task: GRAPH_TASK.to_string(),
                phase: GRAPH_PHASE.to_string(),
                mode: "DEV_FROZEN_BASELINE_PLUS_TOOLING_OVERLAY".to_string(),
                status: "PENDING".to_string(),
                not_final: true,
                graph_id: String::new(),
                payload_files: Vec::new(),
                input_mode: input.mode.clone(),
                baseline_artifacts_sha256: input.artifacts_sha256.clone(),
                baseline_summary_sha256: input.summary_sha256.clone(),
                config_sha256: String::new(),
                source_authority: authority.clone(),
                tooling_overlay,
                counts: Value::Null,
                expected_vs_actual: Value::Null,
                evidence_counts: BTreeMap::new(),
                body_anchor_verification: Value::Null,
                cohorts: Vec::new(),
                limits: Vec::new(),
                publication: "DISABLED".to_string(),
                self_validation: "graph_id is a candidate digest of deterministic payload files; graph-verify rebuilds and compares bytes".to_string(),
            },
        },
        canonical_rows: 0,
        canonical_unique: 0,
        canonical_files: 0,
        canonical_shards: BTreeSet::new(),
        canonical_roots: BTreeSet::new(),
        p1_rows: 0,
        p1_unique: 0,
        p1_overlap_actual: 0,
        p1_new: 0,
        p1_p0_unique: 0,
        p1_p0_overlap: 0,
        p1_shards: BTreeSet::new(),
        p1_roots: BTreeSet::new(),
        authority_binding: Value::Null,
        curated_files: 0,
        curated_methods: 0,
        curated_status_counts: BTreeMap::new(),
        curated_abstract: 0,
        association_summary: Value::Null,
        gameplay_rows: 0,
        gameplay_matched: 0,
        gameplay_source_files: BTreeSet::new(),
        gameplay_primary_sources: BTreeSet::new(),
        gameplay_curated_files: BTreeSet::new(),
        gameplay_curated_rows: 0,
        performance_files: 0,
        performance_curated: 0,
        performance_queue: 0,
        performance_tiers: BTreeMap::new(),
        performance_method_links: 0,
        performance_unmapped: 0,
        performance_rep_ids: BTreeSet::new(),
        body_expected: 0,
        body_verified: 0,
        body_failed: 0,
        body_anchor_classes: BTreeMap::new(),
        duplicate_groups: 0,
        duplicate_paths: 0,
        same_symbol_groups: 0,
        r38_rows: 0,
        r54_staged: 0,
        missing_triad: 0,
        qualification_queue: 0,
        non_managed_counts: BTreeMap::new(),
        abstract_method_key: None,
        config_sha256: String::new(),
        source_authority: authority,
        tooling_overlay: Vec::new(),
        authority_assembly: String::new(),
        authority_mvid: String::new(),
        curated_original_paths: BTreeSet::new(),
        curated_status_by_method: BTreeMap::new(),
        r38_gameplay: Vec::new(),
    };
    // The source authority is loaded before the builder; copy the fixed identity into the builder.
    build.authority_assembly = build
        .source_authority
        .pointer("/assembly_sha256")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    build.authority_mvid = build
        .source_authority
        .pointer("/mvid")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    build.tooling_overlay = build.data.summary.tooling_overlay.clone();
    build.config_sha256 = crate::sha256_hex(&config_bytes);
    build_canonical(&mut build, config, &input)?;
    build_p1(&mut build, config, &input)?;
    build_curated(&mut build, config, &input)?;
    build_authority_partition(&mut build, config, &input)?;
    build_gameplay(&mut build, config, &input)?;
    build_r38(&mut build)?;
    build_performance(&mut build, config, &input)?;
    build_missing_triad(&mut build, &input)?;
    build_non_managed(&mut build, &input);
    build_r54(&mut build, &input)?;
    build_duplicates(&mut build, &input);
    build_range_relationships(&mut build)?;
    build.qualification_queue = build.performance_queue
        + build
            .gameplay_primary_sources
            .len()
            .saturating_sub(build.gameplay_curated_files.len()) as u64;
    add_cohort(
        &mut build.data.cohorts,
        "CANONICAL_PROVENANCE_165",
        "CANONICAL_METHOD_PROVENANCE",
        build.canonical_files,
        build.canonical_unique,
        Vec::new(),
        vec![format!(
            "rows={} unique={}",
            build.canonical_rows, build.canonical_unique
        )],
    );
    add_cohort(
        &mut build.data.cohorts,
        "P1_5_FILES",
        "P1_METHOD_PROVENANCE",
        5,
        build.p1_unique,
        Vec::new(),
        vec![format!(
            "occurrences={} actual_canonical_overlap={} new={} expected_overlap=2276",
            build.p1_rows, build.p1_overlap_actual, build.p1_new
        )],
    );
    add_cohort(
        &mut build.data.cohorts,
        "CURATED_13_FILES",
        "CURATED_IMPORTED",
        build.curated_files,
        build.curated_methods,
        Vec::new(),
        vec![format!(
            "evidence_status_counts={:?}; original/imported pairs are 2 representations",
            build.curated_status_counts
        )],
    );
    add_cohort(
        &mut build.data.cohorts,
        "CURATED_ABSTRACT_1",
        "CURATED_ABSTRACT_METADATA_ONLY",
        1,
        build.curated_abstract,
        Vec::new(),
        vec![
            "TaskBase.OnRun abstract signature; no canonical METHOD_PROVENANCE; metadata-only"
                .to_string(),
        ],
    );
    add_cohort(
        &mut build.data.cohorts,
        "QUALIFICATION_QUEUE_93",
        "QUALIFICATION_QUEUE_NOT_ALLOWLIST",
        build.qualification_queue,
        0,
        Vec::new(),
        vec![format!(
            "performance_queue={} gameplay_noncurated_files={} total={}; not an allowlist and not future import authority",
            build.performance_queue,
            build
                .gameplay_primary_sources
                .len()
                .saturating_sub(build.gameplay_curated_files.len()),
            build.qualification_queue
        )],
    );
    add_cohort(
        &mut build.data.cohorts,
        "MISSING_STRUCTURED_MAP_TRIAD_3",
        "MISSING_STRUCTURED_MAP_TRIAD",
        build.missing_triad,
        0,
        Vec::new(),
        vec!["EventArgs.Invoke.Enum2.cs, EventSystem.EmitWithTarget.Enum2.cs, RangeComponent.LifecycleRecovery.cs; DOC_REPORTED_LINK only".to_string()],
    );
    add_cohort(
        &mut build.data.cohorts,
        "R38_COVERAGE_DIFF",
        "ACCEPTED_VS_FROZEN_COVERAGE",
        8,
        build.r38_rows,
        Vec::new(),
        vec![
            "frozen v5 and accepted gameplay research coverage differ; no promotion or v5 rewrite"
                .to_string(),
        ],
    );
    add_cohort(
        &mut build.data.cohorts,
        "DUPLICATE_EXACT_BYTES",
        "DUPLICATE_GROUPS",
        build.duplicate_groups,
        0,
        Vec::new(),
        vec![format!(
            "duplicate_paths={}; exact size+sha only; no semantic equivalence",
            build.duplicate_paths
        )],
    );
    add_cohort(
        &mut build.data.cohorts,
        "RANGE_PARTIAL_FULL",
        "RANGE_COMPONENT_CLOSURE",
        4,
        10,
        Vec::new(),
        vec!["6 FULL + 4 PARTIAL selected names; lifecycle overlay separate; no end-to-end guarantee".to_string()],
    );
    let mut methods = build.data.methods.into_values().collect::<Vec<_>>();
    for method in &mut methods {
        method.finalize();
    }
    methods.sort_by(|left, right| {
        left.assembly_sha256
            .cmp(&right.assembly_sha256)
            .then_with(|| left.mvid.cmp(&right.mvid))
            .then_with(|| left.type_token.cmp(&right.type_token))
            .then_with(|| left.method_token.cmp(&right.method_token))
            .then_with(|| left.method_id.cmp(&right.method_id))
    });
    let mut links = build.data.links.into_values().collect::<Vec<_>>();
    for link in &mut links {
        link.roles.sort();
        link.roles.dedup();
        link.method_ids.sort();
        link.method_ids.dedup();
        link.cohorts.sort();
        link.cohorts.dedup();
        link.notes.sort();
        link.notes.dedup();
    }
    links.sort_by(|left, right| artifact_key(&left.artifact).cmp(&artifact_key(&right.artifact)));
    let relationships = build.data.relationships.into_values().collect::<Vec<_>>();
    let cohorts = build.data.cohorts.into_values().collect::<Vec<_>>();
    let mut evidence_counts = BTreeMap::new();
    for method in &methods {
        for (level, count) in &method.evidence_levels {
            *evidence_counts.entry(level.clone()).or_insert(0) += count;
        }
    }
    let method_count = methods.len() as u64;
    let mapped_all5 = method_count.saturating_sub(build.curated_abstract);
    let canonical_plus_p0 = build.canonical_unique + build.p1_p0_unique - build.p1_p0_overlap;
    let counts = serde_json::to_value(GraphCounts {
        baseline_artifacts: input.artifacts.len() as u64,
        canonical_files: build.canonical_files,
        canonical_rows: build.canonical_rows,
        canonical_unique: build.canonical_unique,
        p1_files: 5,
        p1_rows: build.p1_rows,
        p1_unique: build.p1_unique,
        p1_p0_unique: build.p1_p0_unique,
        p1_p0_overlap: build.p1_p0_overlap,
        p1_overlap_actual: build.p1_overlap_actual,
        p1_all5_new: build.p1_new,
        canonical_plus_p0_partial: canonical_plus_p0,
        all5_union: mapped_all5,
        mapped_methods_all5_actual: mapped_all5,
        known_methods_all5_incl_abstract: method_count,
        p1_scope: "ACCEPTED_ALL_FIVE_PRESEEDED_PARTITION".to_string(),
        accepted_partition: build.authority_binding.clone(),
        curated_files: build.curated_files,
        curated_methods: build.curated_methods,
        curated_abstract: build.curated_abstract,
        curated_associations: build.association_summary.clone(),
        gameplay_rows: build.gameplay_rows,
        gameplay_methods_matched: build.gameplay_matched,
        gameplay_source_files: build.gameplay_primary_sources.len() as u64,
        gameplay_linked_source_artifacts: build.gameplay_source_files.len() as u64,
        gameplay_curated_files: build.gameplay_curated_files.len() as u64,
        gameplay_curated_rows: build.gameplay_curated_rows,
        performance_files: build.performance_files,
        performance_curated: build.performance_curated,
        performance_queue: build.performance_queue,
        performance_tiers: build.performance_tiers.clone(),
        performance_method_links: build.performance_method_links,
        performance_unmapped: build.performance_unmapped,
        qualification_queue: build.qualification_queue,
        duplicate_groups: build.duplicate_groups,
        duplicate_paths: build.duplicate_paths,
        same_symbol_groups: build.same_symbol_groups,
        r38_rows: build.r38_rows,
        r54_staged: build.r54_staged,
        missing_triad: build.missing_triad,
        method_rows: methods.len() as u64,
        link_rows: links.len() as u64,
        relationship_rows: relationships.len() as u64,
        cohort_rows: cohorts.len() as u64,
        body_anchor_classes: build.body_anchor_classes.clone(),
        missing_rows: build.data.missing.len() as u64,
        non_managed: build.non_managed_counts.clone(),
    })
    .map_err(|_| {
        RegistryError::new(
            "GRAPH_SERIALIZE_ERROR",
            "graph counts could not be serialized",
        )
    })?;
    let expected_vs_actual = json!({
        "canonical": {"expected": ACCEPTED_CANONICAL_METHODS, "actual": build.canonical_unique},
        "p1": {"expected": ACCEPTED_P1_METHODS, "actual": build.p1_unique},
        "p1_overlap": {"expected": 0, "actual": build.p1_overlap_actual},
        "all5_union": {"expected": ACCEPTED_ALL5_METHODS, "actual": mapped_all5},
        "known_incl_abstract": {"expected": ACCEPTED_KNOWN_METHODS, "actual": method_count},
        "initial_primary_partial_p0": {
            "value": ACCEPTED_INITIAL_PARTIAL,
            "scope": "canonical+P0 only",
            "status": "INITIAL_PRIMARY_PARTIAL_NOT_ACCEPTED_UNION"
        },
        "omitted_other_four_p1": ACCEPTED_OMITTED_OTHER4,
        "p1_scope": "ACCEPTED_ALL_FIVE_PRESEEDED_PARTITION",
        "status": "ACCEPTED_AUTHORITY_MATCH"
    });
    let mut missing = build.data.missing.into_values().collect::<Vec<_>>();
    missing.sort_by(|left, right| left.missing_id.cmp(&right.missing_id));
    let mut relationships = relationships;
    relationships.sort_by(|left, right| left.relationship_id.cmp(&right.relationship_id));
    let mut cohorts = cohorts;
    cohorts.sort_by(|left, right| left.cohort_id.cmp(&right.cohort_id));
    let body_anchor_verification = json!({
        "expected_body_hashes": build.body_expected,
        "verified": build.body_verified,
        "failed": build.body_failed,
        "scope": "accepted gameplay map rows with a dedicated Body SHA-256 header and source range; UTF-8 LF-normalized plus one trailing LF",
        "r38_row288_expected": "104494b32b927284efc860883c49a233ff5ea3bc13efc3060873bb11873dd32d",
        "r38_row295_expected": "529466cc1e43d45854c03bb8a024e4f682163064023ed2572464432043506571"
    });
    let mut summary = build.data.summary;
    summary.config_sha256 = build.config_sha256.clone();
    summary.source_authority = build.source_authority.clone();
    summary.tooling_overlay = build.tooling_overlay.clone();
    summary.counts = counts;
    summary.expected_vs_actual = expected_vs_actual;
    summary.evidence_counts = evidence_counts;
    summary.body_anchor_verification = body_anchor_verification;
    summary.cohorts = cohorts
        .iter()
        .map(|cohort| cohort.cohort_id.clone())
        .collect();
    summary.input_mode = input.mode.clone();
    summary.baseline_artifacts_sha256 = input.artifacts_sha256.clone();
    summary.baseline_summary_sha256 = input.summary_sha256.clone();
    let source_snapshot = input.mode == "RUST_SNAPSHOT";
    summary.mode = if source_snapshot {
        "SOURCE_SNAPSHOT_DERIVED".to_string()
    } else {
        "DEV_FROZEN_BASELINE_PLUS_TOOLING_OVERLAY".to_string()
    };
    summary.status = if source_snapshot {
        if missing.is_empty() && build.body_failed == 0 {
            "SOURCE_SNAPSHOT_DERIVED".to_string()
        } else {
            "SOURCE_SNAPSHOT_DERIVED_WITH_DECLARED_GAPS".to_string()
        }
    } else if missing.is_empty() && build.body_failed == 0 {
        "DEV_COMPLETE".to_string()
    } else {
        "DEV_COMPLETE_WITH_DECLARED_GAPS".to_string()
    };
    summary.limits = vec![
        if source_snapshot {
            "Input is a Rust source snapshot; this is a derived graph, not final acceptance, an atomic freeze, or a restorable-backup claim.".to_string()
        } else {
            "DEV input is the frozen Phase A baseline plus an explicit current-tooling overlay; it is not a final current-corpus snapshot.".to_string()
        },
        "Canonical and P1 provenance are metadata/raw provenance, not DIRECT_CIL_FULL evidence.".to_string(),
        "Accepted canonical+P1 partition is bound by exact shard-plan, preseed-map, completion, parent-manifest and review-ID hashes: 303456 canonical methods, 2399 P1 methods, zero token overlap, 305855 mapped, 305856 known incl. the retained TaskBase.OnRun abstract metadata identity.".to_string(),
        "The initial 303579 count was a canonical+P0-only partial scope, not the accepted canonical+P1 baseline; the other four P1 shards contribute 2276 methods.".to_string(),
        "TaskBase.OnRun abstract is retained metadata-only/no-IL and is not a missing body error.".to_string(),
        "Gameplay anchor classes are derived from actual semantic anchor/range/dedicated-body-hash fields, not row ordinals.".to_string(),
        "Performance file associations are exact or explicitly inferred; method links are per representation and do not promote file scope.".to_string(),
        "Gameplay body hashes prove source-slice bytes, not IL semantics or runtime behavior.".to_string(),
        "Missing triad entries are DOC_REPORTED_LINK records, not manufactured structured maps or qualified imports.".to_string(),
        "No publication, runtime, native, or full current-corpus verification is claimed.".to_string(),
    ];
    let data = GraphData {
        methods: methods
            .into_iter()
            .map(|method| (method.method_id.clone(), method))
            .collect(),
        links: links
            .into_iter()
            .map(|link| (artifact_key(&link.artifact), link))
            .collect(),
        relationships: relationships
            .into_iter()
            .map(|row| (row.relationship_id.clone(), row))
            .collect(),
        cohorts: cohorts
            .into_iter()
            .map(|row| (row.cohort_id.clone(), row))
            .collect(),
        missing: missing
            .into_iter()
            .map(|row| (row.missing_id.clone(), row))
            .collect(),
        summary,
    };
    Ok(data)
}

fn load_source_authority(config: &Config, input: &LoadedInput) -> RegistryResult<Value> {
    let (_, bytes) = read_artifact_bytes(
        config,
        &input.artifacts,
        CURATED_ROOT,
        "metadata/migration.json",
    )?;
    let value = crate::parse_strict_value(&bytes).map_err(|_| {
        RegistryError::new(
            "GRAPH_AUTHORITY_INVALID",
            "migration authority JSON is invalid",
        )
    })?;
    let managed = value.pointer("/authority_policy/managed").ok_or_else(|| {
        RegistryError::new(
            "GRAPH_AUTHORITY_INVALID",
            "managed authority policy is missing",
        )
    })?;
    let authority_id = value_str(managed, "authority_id")?;
    let assembly = value_str(managed, "authority_binary_sha256")?;
    let mvid = value_str(managed, "mvid")?;
    Ok(json!({
        "path": "metadata/migration.json",
        "sha256": crate::sha256_hex(&bytes),
        "authority_id": authority_id,
        "assembly_sha256": assembly,
        "mvid": mvid,
        "basis": "fixed pinned CURATED_ROOT metadata/migration.json authority; no fresh binary query"
    }))
}

fn tooling_overlay(config: &Config, input: &LoadedInput) -> RegistryResult<Vec<OverlayEntry>> {
    let paths = [
        "tools/re/Cargo.toml",
        "tools/re/Cargo.lock",
        "tools/re/src/lib.rs",
        "tools/re/src/main.rs",
        "tools/re/src/registry.rs",
        "tools/re/src/registry/graph.rs",
        "tools/re/src/bin/corpus-registry.rs",
        "tools/re/tests/validation.rs",
        "tools/re/tests/registry.rs",
    ];
    let mut overlay = Vec::new();
    for path in paths {
        let baseline = lookup_artifact(&input.artifacts, CURATED_ROOT, path)
            .and_then(|row| row.sha256.clone());
        let absolute = root_path(config, CURATED_ROOT)?.join(path);
        let current = if fs::symlink_metadata(&absolute).is_ok() {
            inspect_existing_file(&absolute)?;
            let bytes = fs::read(&absolute).map_err(|_| {
                RegistryError::new("GRAPH_TOOLING_READ_ERROR", "tooling file could not be read")
            })?;
            Some(crate::sha256_hex(&bytes))
        } else {
            None
        };
        let status = match (&baseline, &current) {
            (Some(base), Some(cur)) if base == cur => "MATCH",
            (Some(_), Some(_)) => "AUTHORIZED_PHASE_A_OR_B_DELTA",
            (None, Some(_)) => "NEW_AFTER_BASELINE",
            _ => "MISSING",
        };
        overlay.push(OverlayEntry {
            path: path.to_string(),
            baseline_sha256: baseline,
            current_sha256: current,
            status: status.to_string(),
        });
    }
    overlay.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(overlay)
}

fn hex_digest(digest: impl AsRef<[u8]>) -> String {
    let mut output = String::with_capacity(64);
    for byte in digest.as_ref() {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn write_jsonl<T: Serialize>(path: &Path, rows: &[T]) -> RegistryResult<PayloadFile> {
    let file = fs::File::create_new(path).map_err(|_| {
        RegistryError::new(
            "GRAPH_WRITE_ERROR",
            "graph payload file could not be created",
        )
    })?;
    let mut writer = BufWriter::new(file);
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    for row in rows {
        let line = serde_json::to_string(row).map_err(|_| {
            RegistryError::new("GRAPH_SERIALIZE_ERROR", "graph row could not be serialized")
        })?;
        writer.write_all(line.as_bytes()).map_err(|_| {
            RegistryError::new("GRAPH_WRITE_ERROR", "graph row could not be written")
        })?;
        writer.write_all(b"\n").map_err(|_| {
            RegistryError::new(
                "GRAPH_WRITE_ERROR",
                "graph row terminator could not be written",
            )
        })?;
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
        bytes += line.len() as u64 + 1;
    }
    writer.flush().map_err(|_| {
        RegistryError::new("GRAPH_WRITE_ERROR", "graph payload could not be flushed")
    })?;
    let file = writer.into_inner().map_err(|_| {
        RegistryError::new("GRAPH_WRITE_ERROR", "graph payload could not be finalized")
    })?;
    file.sync_all().map_err(|_| {
        RegistryError::new("GRAPH_WRITE_ERROR", "graph payload could not be synced")
    })?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("payload")
        .to_string();
    Ok(PayloadFile {
        name,
        bytes,
        sha256: hex_digest(hasher.finalize()),
    })
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> RegistryResult<PayloadFile> {
    let bytes = serde_json::to_vec(value).map_err(|_| {
        RegistryError::new(
            "GRAPH_SERIALIZE_ERROR",
            "graph JSON could not be serialized",
        )
    })?;
    let mut file = fs::File::create_new(path)
        .map_err(|_| RegistryError::new("GRAPH_WRITE_ERROR", "graph JSON could not be created"))?;
    file.write_all(&bytes)
        .map_err(|_| RegistryError::new("GRAPH_WRITE_ERROR", "graph JSON could not be written"))?;
    file.sync_all()
        .map_err(|_| RegistryError::new("GRAPH_WRITE_ERROR", "graph JSON could not be synced"))?;
    Ok(PayloadFile {
        name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("summary.json")
            .to_string(),
        bytes: bytes.len() as u64,
        sha256: crate::sha256_hex(&bytes),
    })
}

fn payload_graph_id(files: &[PayloadFile]) -> String {
    let mut hasher = Sha256::new();
    for file in files {
        hasher.update(file.name.as_bytes());
        hasher.update([0]);
        hasher.update(file.sha256.as_bytes());
        hasher.update(b"\n");
    }
    hex_digest(hasher.finalize())
}

fn write_graph(output: &Path, data: &GraphData) -> RegistryResult<Vec<PayloadFile>> {
    let mut methods = data.methods.values().cloned().collect::<Vec<_>>();
    methods.sort_by(|left, right| {
        left.assembly_sha256
            .cmp(&right.assembly_sha256)
            .then_with(|| left.mvid.cmp(&right.mvid))
            .then_with(|| left.type_token.cmp(&right.type_token))
            .then_with(|| left.method_token.cmp(&right.method_token))
            .then_with(|| left.method_id.cmp(&right.method_id))
    });
    let mut links = data.links.values().cloned().collect::<Vec<_>>();
    links.sort_by(|left, right| artifact_key(&left.artifact).cmp(&artifact_key(&right.artifact)));
    let mut relationships = data.relationships.values().cloned().collect::<Vec<_>>();
    relationships.sort_by(|left, right| left.relationship_id.cmp(&right.relationship_id));
    let mut cohorts = data.cohorts.values().cloned().collect::<Vec<_>>();
    cohorts.sort_by(|left, right| left.cohort_id.cmp(&right.cohort_id));
    let mut missing = data.missing.values().cloned().collect::<Vec<_>>();
    missing.sort_by(|left, right| left.missing_id.cmp(&right.missing_id));
    let mut payload = Vec::new();
    payload.push(write_jsonl(&output.join("methods.jsonl"), &methods)?);
    payload.push(write_jsonl(&output.join("artifacts.links.jsonl"), &links)?);
    payload.push(write_jsonl(
        &output.join("relationships.jsonl"),
        &relationships,
    )?);
    payload.push(write_jsonl(&output.join("cohorts.jsonl"), &cohorts)?);
    payload.push(write_jsonl(&output.join("missing.jsonl"), &missing)?);
    Ok(payload)
}

fn write_readme(output: &Path, summary: &GraphSummary) -> RegistryResult<()> {
    let text = format!(
        "# Private corpus graph ({mode})\n\n\
This is a private, local-only **{mode}** derivation from `{input_mode}` inputs. It is **not** final acceptance, an atomic freeze, or a restorable-backup claim, and it does not claim publication or runtime verification.\n\n\
## Commands\n\n\
```text\n\
corpus-registry scan --config <private config> --output <fresh snapshot dir>\n\
corpus-registry verify --config <private config> --snapshot <snapshot dir>\n\
corpus-registry graph-build --config <private config> --snapshot <baseline taskroot or Rust snapshot dir> --output <fresh graph dir>\n\
corpus-registry graph-verify --config <private config> --snapshot <same input> --graph <graph dir> --scratch <fresh dir>\n\
corpus-registry graph-lookup --graph <graph dir> --assembly-sha <sha256> --mvid <uuid> --type <0x02......> --method <0x06......>\n\
corpus-registry graph-lookup --graph <graph dir> --artifact-root-id <root> --artifact-path <relative/path>\n\
```\n\n\
## Status\n\n\
status={status}\ngraph_id={graph_id}\nmode={mode}\ninput_mode={input_mode}\npublication=DISABLED\n\n\
## Guards\n\n\
All graph/config/snapshot member reads use the private no-follow/reparse guard. Outputs are CreateNew and excluded from scans. Rows use root IDs plus relative paths; the private config is the only absolute-path input. Lookup does not re-hash the source corpus; it recomputes graph payload hashes and states that mode. Artifact hashes are preservation locators, not backups. Missing/ambiguous links are preserved explicitly rather than skipped. Full current-corpus rescan, full validation suite, and final acceptance remain Phase C only.\n",
        mode = summary.mode,
        input_mode = summary.input_mode,
        status = summary.status,
        graph_id = summary.graph_id,
    );
    let mut file = fs::File::create_new(output.join("README.md")).map_err(|_| {
        RegistryError::new("GRAPH_WRITE_ERROR", "graph README could not be created")
    })?;
    file.write_all(text.as_bytes()).map_err(|_| {
        RegistryError::new("GRAPH_WRITE_ERROR", "graph README could not be written")
    })?;
    file.sync_all()
        .map_err(|_| RegistryError::new("GRAPH_WRITE_ERROR", "graph README could not be synced"))?;
    Ok(())
}

fn graph_build(config_path: &Path, snapshot: &Path, output: &Path) -> RegistryResult<u8> {
    let started = now_unix_ms();
    let config = load_config(config_path)?;
    let output_dir = OutputDir::create(&config, output)?;
    let mut data = derive_graph(config_path, &config, snapshot)?;
    let payload = write_graph(&output_dir.path, &data)?;
    data.summary.graph_id = payload_graph_id(&payload);
    data.summary.payload_files = payload.clone();
    write_readme(&output_dir.path, &data.summary)?;
    let summary = write_json(&output_dir.path.join("summary.json"), &data.summary)?;
    let _ = summary;
    let execution = ExecutionReceipt {
        schema: "wuwa-reconstruction-corpus-graph-execution-v1".to_string(),
        command: "graph-build".to_string(),
        started_unix_ms: started,
        finished_unix_ms: now_unix_ms(),
        elapsed_ms: now_unix_ms().saturating_sub(started),
        status: data.summary.status.clone(),
        exit: if data.summary.status.contains("COMPLETE") || data.summary.status.contains("DERIVED")
        {
            0
        } else {
            1
        },
    };
    write_json(&output_dir.path.join("execution.json"), &execution)?;
    print_cli(json!({
        "command": "graph-build",
        "ok": execution.exit == 0,
        "exit": execution.exit,
        "status": data.summary.status,
        "graph_id": data.summary.graph_id,
        "counts": data.summary.counts,
        "expected_vs_actual": data.summary.expected_vs_actual,
        "body_anchor_verification": data.summary.body_anchor_verification,
        "findings": data.summary.limits
    }));
    Ok(execution.exit)
}

fn read_graph_summary(graph: &Path) -> RegistryResult<GraphSummary> {
    inspect_existing_directory(graph)?;
    inspect_existing_file(&graph.join("summary.json"))?;
    let bytes = fs::read(graph.join("summary.json")).map_err(|_| {
        RegistryError::new("GRAPH_SUMMARY_MISSING", "graph summary could not be read")
    })?;
    let value = crate::parse_strict_value(&bytes)
        .map_err(|_| RegistryError::new("GRAPH_SUMMARY_INVALID", "graph summary is invalid"))?;
    let summary: GraphSummary = serde_json::from_value(value).map_err(|_| {
        RegistryError::new("GRAPH_SUMMARY_INVALID", "graph summary shape is invalid")
    })?;
    if summary.schema != GRAPH_SCHEMA {
        return Err(RegistryError::new(
            "GRAPH_SUMMARY_INVALID",
            "graph summary schema is not recognized",
        ));
    }
    Ok(summary)
}

fn graph_file_set(graph: &Path) -> RegistryResult<BTreeSet<String>> {
    let mut files = BTreeSet::new();
    for entry in fs::read_dir(graph).map_err(|_| {
        RegistryError::new("GRAPH_DIRECTORY_ERROR", "graph directory could not be read")
    })? {
        let entry = entry.map_err(|_| {
            RegistryError::new(
                "GRAPH_DIRECTORY_ERROR",
                "graph directory entry could not be read",
            )
        })?;
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = fs::symlink_metadata(entry.path()).map_err(|_| {
            RegistryError::new(
                "GRAPH_DIRECTORY_ERROR",
                "graph directory entry metadata failed",
            )
        })?;
        if is_reparse(&metadata) {
            return Err(RegistryError::new(
                "UNSAFE_REPARSE_OR_SYMLINK",
                "graph directory contains a reparse object",
            ));
        }
        files.insert(name);
    }
    Ok(files)
}

fn compare_payload_dirs(left: &Path, right: &Path, names: &[&str]) -> RegistryResult<Vec<String>> {
    let mut mismatches = Vec::new();
    for name in names {
        let left_bytes = fs::read(left.join(name)).map_err(|_| {
            RegistryError::new("GRAPH_PAYLOAD_MISSING", "graph payload could not be read")
        })?;
        let right_bytes = fs::read(right.join(name)).map_err(|_| {
            RegistryError::new("GRAPH_PAYLOAD_MISSING", "scratch payload could not be read")
        })?;
        if left_bytes != right_bytes {
            mismatches.push((*name).to_string());
        }
    }
    Ok(mismatches)
}

fn graph_verify(
    config_path: &Path,
    snapshot: &Path,
    graph: &Path,
    scratch: &Path,
) -> RegistryResult<u8> {
    let config = load_config(config_path)?;
    let summary = read_graph_summary(graph)?;
    let expected = GRAPH_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<BTreeSet<_>>();
    let actual = graph_file_set(graph)?;
    let mut findings = Vec::new();
    if actual != expected {
        findings
            .push(json!({"code":"GRAPH_FILE_SET_MISMATCH","expected":expected,"actual":actual}));
    }
    let mut payload = Vec::new();
    for name in PAYLOAD_FILES {
        let path = graph.join(name);
        inspect_existing_file(&path)?;
        let bytes = fs::read(&path).map_err(|_| {
            RegistryError::new("GRAPH_PAYLOAD_MISSING", "graph payload could not be read")
        })?;
        payload.push(PayloadFile {
            name: name.to_string(),
            bytes: bytes.len() as u64,
            sha256: crate::sha256_hex(&bytes),
        });
    }
    let computed_graph_id = payload_graph_id(&payload);
    if computed_graph_id != summary.graph_id || payload != summary.payload_files {
        findings.push(json!({"code":"GRAPH_PAYLOAD_HASH_MISMATCH"}));
    }
    let scratch_dir = OutputDir::create(&config, scratch)?;
    let mut rebuilt = derive_graph(config_path, &config, snapshot)?;
    let rebuilt_payload = write_graph(&scratch_dir.path, &rebuilt)?;
    rebuilt.summary.graph_id = payload_graph_id(&rebuilt_payload);
    rebuilt.summary.payload_files = rebuilt_payload.clone();
    write_readme(&scratch_dir.path, &rebuilt.summary)?;
    write_json(&scratch_dir.path.join("summary.json"), &rebuilt.summary)?;
    let execution = ExecutionReceipt {
        schema: "wuwa-reconstruction-corpus-graph-execution-v1".to_string(),
        command: "graph-verify".to_string(),
        started_unix_ms: now_unix_ms(),
        finished_unix_ms: now_unix_ms(),
        elapsed_ms: 0,
        status: "SCRATCH".to_string(),
        exit: 0,
    };
    write_json(&scratch_dir.path.join("execution.json"), &execution)?;
    let mismatches = compare_payload_dirs(
        graph,
        &scratch_dir.path,
        &[
            "methods.jsonl",
            "artifacts.links.jsonl",
            "relationships.jsonl",
            "cohorts.jsonl",
            "missing.jsonl",
            "summary.json",
            "README.md",
        ],
    )?;
    let exact = findings.is_empty() && mismatches.is_empty();
    let exit = if exact { 0 } else { 1 };
    print_cli(json!({
        "command": "graph-verify",
        "ok": exact,
        "exit": exit,
        "status": if exact { "EXACT_MATCH" } else { "MISMATCH" },
        "graph_id": summary.graph_id,
        "graph_id_recomputed": computed_graph_id,
        "counts": summary.counts,
        "mismatched_files": mismatches,
        "findings": findings,
        "verification_mode": "REBUILT_FROM_VERIFIED_INPUTS_AND_COMPARED_BYTES"
    }));
    Ok(exit)
}

fn find_method_in_graph(graph: &Path, method_id: &str) -> RegistryResult<Option<MethodRow>> {
    let path = graph.join("methods.jsonl");
    inspect_existing_file(&path)?;
    let file = fs::File::open(&path).map_err(|_| {
        RegistryError::new(
            "GRAPH_METHODS_MISSING",
            "methods payload could not be opened",
        )
    })?;
    let reader = BufReader::new(file);
    let needle = format!("\"method_id\":\"{method_id}\"");
    for line in reader.lines() {
        let line = line.map_err(|_| {
            RegistryError::new("GRAPH_METHODS_INVALID", "methods payload could not be read")
        })?;
        if !line.contains(&needle) {
            continue;
        }
        let value = crate::parse_strict_value(line.as_bytes())
            .map_err(|_| RegistryError::new("GRAPH_METHODS_INVALID", "method row is invalid"))?;
        let row: MethodRow = serde_json::from_value(value).map_err(|_| {
            RegistryError::new("GRAPH_METHODS_INVALID", "method row shape is invalid")
        })?;
        if row.method_id == method_id {
            return Ok(Some(row));
        }
    }
    Ok(None)
}

fn graph_lookup(flags: &BTreeMap<String, PathBuf>) -> RegistryResult<u8> {
    let graph = flags
        .get("--graph")
        .ok_or_else(|| RegistryError::new("MISSING_ARGUMENT", "--graph is required"))?;
    let summary = read_graph_summary(graph)?;
    let expected = GRAPH_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<BTreeSet<_>>();
    let actual = graph_file_set(graph)?;
    if actual != expected {
        return Err(RegistryError::new(
            "GRAPH_FILE_SET_MISMATCH",
            "graph directory is not the exact expected closure",
        ));
    }
    let mut payload = Vec::new();
    for name in PAYLOAD_FILES {
        let path = graph.join(name);
        inspect_existing_file(&path)?;
        let bytes = fs::read(&path).map_err(|_| {
            RegistryError::new("GRAPH_PAYLOAD_MISSING", "graph payload could not be read")
        })?;
        payload.push(PayloadFile {
            name: name.to_string(),
            bytes: bytes.len() as u64,
            sha256: crate::sha256_hex(&bytes),
        });
    }
    if payload_graph_id(&payload) != summary.graph_id || payload != summary.payload_files {
        return Err(RegistryError::new(
            "GRAPH_PAYLOAD_HASH_MISMATCH",
            "graph payload does not match its summary hashes",
        ));
    }
    let assembly = flags
        .get("--assembly-sha")
        .map(|value| value.to_string_lossy().to_string());
    let mvid = flags
        .get("--mvid")
        .map(|value| value.to_string_lossy().to_string());
    let type_token = flags
        .get("--type")
        .map(|value| value.to_string_lossy().to_string());
    let method_token = flags
        .get("--method")
        .map(|value| value.to_string_lossy().to_string());
    if let (Some(assembly), Some(mvid), Some(type_token), Some(method_token)) =
        (assembly, mvid, type_token, method_token)
    {
        let key = MethodKey {
            assembly_sha256: norm_token(&assembly),
            mvid: norm_token(&mvid),
            type_token: norm_token(&type_token),
            method_token: norm_token(&method_token),
        };
        let method_id = key.id();
        let found = find_method_in_graph(graph, &method_id)?;
        let exit = if found.is_some() { 0 } else { 1 };
        print_cli(json!({
            "command": "graph-lookup",
            "ok": found.is_some(),
            "exit": exit,
            "status": if found.is_some() { "FOUND" } else { "NOT_FOUND" },
            "method": found,
            "graph_id": summary.graph_id,
            "verification_mode": "GRAPH_PAYLOAD_HASHES_RECOMPUTED; INPUT_CORPUS_NOT_RECHECKED",
            "limits": summary.limits
        }));
        return Ok(exit);
    }
    let artifact_root = flags
        .get("--artifact-root-id")
        .map(|value| value.to_string_lossy().to_string());
    let artifact_path = flags
        .get("--artifact-path")
        .map(|value| value.to_string_lossy().to_string());
    if let (Some(root_id), Some(path)) = (artifact_root, artifact_path) {
        let mut found = None;
        let root_needle = format!("\"root_id\":\"{root_id}\"");
        let path_needle = format!("\"path\":\"{path}\"");
        for line in fs::read_to_string(graph.join("artifacts.links.jsonl"))
            .map_err(|_| {
                RegistryError::new("GRAPH_LINKS_MISSING", "links payload could not be read")
            })?
            .lines()
        {
            if line.is_empty() || !line.contains(&root_needle) || !line.contains(&path_needle) {
                continue;
            }
            let value = crate::parse_strict_value(line.as_bytes())
                .map_err(|_| RegistryError::new("GRAPH_LINKS_INVALID", "links row is invalid"))?;
            let row: ArtifactLinkRow = serde_json::from_value(value).map_err(|_| {
                RegistryError::new("GRAPH_LINKS_INVALID", "links row shape is invalid")
            })?;
            if row.artifact.root_id == root_id && row.artifact.path == path {
                found = Some(row);
                break;
            }
        }
        let exit = if found.is_some() { 0 } else { 1 };
        print_cli(json!({
            "command": "graph-lookup",
            "ok": found.is_some(),
            "exit": exit,
            "status": if found.is_some() { "FOUND" } else { "NOT_FOUND" },
            "artifact": found,
            "graph_id": summary.graph_id,
            "verification_mode": "GRAPH_PAYLOAD_HASHES_RECOMPUTED; INPUT_CORPUS_NOT_RECHECKED",
            "limits": summary.limits
        }));
        return Ok(exit);
    }
    Err(RegistryError::new(
        "MISSING_ARGUMENT",
        "provide a full method key or artifact-root-id + artifact-path",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("repository root")
            .to_path_buf()
    }

    #[test]
    fn markdown_cell_split_keeps_pipes_inside_backticks_and_escapes() {
        let cells = split_md_cells("| 1 | `A|B` | `C\\|D` | E |");
        assert_eq!(cells, vec!["1", "`A|B`", "`C\\|D`", "E"]);
    }

    #[test]
    fn payload_comparison_rejects_resealed_evidence_promotion_and_omitted_rows() {
        let root = std::env::temp_dir().join(format!(
            "wuwa-graph-compare-{}-{}",
            std::process::id(),
            now_unix_ms()
        ));
        let left = root.join("left");
        let right = root.join("right");
        fs::create_dir_all(&left).unwrap();
        fs::create_dir_all(&right).unwrap();
        fs::write(
            left.join("methods.jsonl"),
            "{\"method_id\":\"m\",\"evidence\":[\"SOURCE_ONLY\"]}\n",
        )
        .unwrap();
        fs::write(
            right.join("methods.jsonl"),
            "{\"method_id\":\"m\",\"evidence\":[\"DIRECT_CIL_FULL\"]}\n",
        )
        .unwrap();
        fs::write(left.join("cohorts.jsonl"), "{\"cohort_id\":\"c\"}\n").unwrap();
        fs::write(right.join("cohorts.jsonl"), "").unwrap();
        let mismatches =
            compare_payload_dirs(&left, &right, &["methods.jsonl", "cohorts.jsonl"]).unwrap();
        assert_eq!(mismatches, vec!["methods.jsonl", "cohorts.jsonl"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn gameplay_anchor_class_uses_fields_not_row_ordinals() {
        let (name, range) =
            short_filename_with_range("`CharacterMoveComponent.JumpHandoff.cs:56-117`").unwrap();
        assert_eq!(name, "CharacterMoveComponent.JumpHandoff.cs");
        let range = range.unwrap();
        let body = "a".repeat(64);
        assert_eq!(
            gameplay_anchor_class(true, Some(&range), Some(&body)),
            "EXPLICIT_RANGE_WITH_BODY_SHA"
        );
        assert_eq!(
            gameplay_anchor_class(true, Some(&range), None),
            "EXPLICIT_RANGE_NO_BODY_SHA"
        );
        assert_eq!(
            gameplay_anchor_class(true, None, None),
            "FILE_OR_METHOD_ANCHOR_ONLY"
        );
        assert_eq!(
            gameplay_anchor_class(false, None, Some(&body)),
            "BODY_HASH_WITHOUT_SEMANTIC_RANGE"
        );
        assert_eq!(
            gameplay_anchor_class(false, None, None),
            "UNRESOLVED_ANCHOR"
        );
    }

    #[test]
    fn actual_gameplay_map_anchor_distribution_is_field_derived() {
        let path =
            repo_root().join("src/reconstructed/managed/semantic/gameplay/METHOD_EVIDENCE_MAP.md");
        let text = fs::read_to_string(path).unwrap();
        let rows = parse_gameplay_tables(&text);
        let mut classes = BTreeMap::new();
        let mut accepted = 0u64;
        for row in rows {
            let Some(number) = row.cells.first().and_then(|cell| parse_decimal_cell(cell)) else {
                continue;
            };
            if !(1..=301).contains(&number) {
                continue;
            }
            if row
                .cells
                .iter()
                .find_map(|cell| identity_pair(cell))
                .is_none()
            {
                continue;
            }
            accepted += 1;
            let mut has_semantic_anchor = false;
            let mut semantic_range = None;
            for cell in &row.cells {
                for part in cell.split(';') {
                    if part.contains(GAMEPLAY_PREFIX) {
                        has_semantic_anchor = true;
                        if let Some((_, range)) = path_and_range(part) {
                            semantic_range = range.or(semantic_range);
                        }
                    } else if let Some((_, range)) = short_filename_with_range(part) {
                        has_semantic_anchor = true;
                        semantic_range = range.or(semantic_range);
                    }
                }
            }
            if has_semantic_anchor && semantic_range.is_none() {
                for cell in &row.cells {
                    if let Some(range) = overlay_range(cell) {
                        semantic_range = Some(range);
                        break;
                    }
                }
            }
            let body_hash = row
                .header
                .iter()
                .position(|header| header_has_body_sha(header))
                .and_then(|index| row.cells.get(index))
                .map(|cell| cell.trim().trim_matches('`').to_string())
                .filter(|cell| is_lower_hex_64(cell));
            let class = gameplay_anchor_class(
                has_semantic_anchor,
                semantic_range.as_ref(),
                body_hash.as_deref(),
            );
            *classes.entry(class.to_string()).or_insert(0u64) += 1;
        }
        assert_eq!(accepted, 301);
        assert_eq!(classes.get("EXPLICIT_RANGE_WITH_BODY_SHA"), Some(&22));
        assert_eq!(classes.get("EXPLICIT_RANGE_NO_BODY_SHA"), Some(&212));
        assert_eq!(classes.get("FILE_OR_METHOD_ANCHOR_ONLY"), Some(&67));
    }

    #[test]
    fn normalized_slice_hash_matches_r38_accepted_anchors() {
        let root = repo_root();
        let source = fs::read(
            root.join("src/reconstructed/managed/semantic/gameplay/Calculation.DamageFormula.cs"),
        )
        .unwrap();
        assert_eq!(
            normalized_slice_hash(
                &source,
                &SourceRange {
                    start_line: 55,
                    end_line: 81
                }
            )
            .unwrap(),
            "104494b32b927284efc860883c49a233ff5ea3bc13efc3060873bb11873dd32d"
        );
        assert_eq!(
            normalized_slice_hash(
                &source,
                &SourceRange {
                    start_line: 188,
                    end_line: 233
                }
            )
            .unwrap(),
            "529466cc1e43d45854c03bb8a024e4f682163064023ed2572464432043506571"
        );
        let move_source = fs::read(root.join(
            "src/reconstructed/managed/semantic/gameplay/CharacterMoveComponent.JumpHandoff.cs",
        ))
        .unwrap();
        assert_eq!(
            normalized_slice_hash(
                &move_source,
                &SourceRange {
                    start_line: 56,
                    end_line: 117
                }
            )
            .unwrap(),
            "122a1cad1e7231f84c60e52a7921881f3f6e7f487cca8bf7652fff9ae1cd4103"
        );
    }

    #[test]
    fn curated_v5_status_counts_are_derived_without_promotion() {
        let path = repo_root().join("github_src/metadata/imports.v5.json");
        let value = crate::parse_strict_value(&fs::read(path).unwrap()).unwrap();
        let imports = value_array(&value, "imports").unwrap();
        assert_eq!(imports.len(), 13);
        let mut counts = BTreeMap::new();
        let mut methods = 0u64;
        for entry in imports {
            if let Some(list) = entry.get("methods").and_then(Value::as_array) {
                for method in list {
                    methods += 1;
                    let status = value_str(method, "evidence_status").unwrap();
                    *counts.entry(status.to_string()).or_insert(0u64) += 1;
                }
            } else if entry.get("method_identity").is_some() {
                methods += 1;
                *counts.entry("DIRECT_CIL_FULL".to_string()).or_insert(0u64) += 1;
            }
        }
        assert_eq!(methods, 102);
        assert_eq!(counts.get("DIRECT_CIL_FULL"), Some(&45));
        assert_eq!(counts.get("SOURCE_ONLY"), Some(&50));
        assert_eq!(counts.get("UNKNOWN"), Some(&6));
        assert_eq!(counts.get("METADATA_ONLY"), Some(&1));
    }

    #[test]
    fn gameplay_parser_prefers_leftmost_identity_cell_and_ignores_shard_hashes() {
        let markdown = "## Test\n\n| # | Overlay method | TypeDef / MethodDef | Source | Body SHA-256 |\n|---:|---|---|---|---|\n| 58 | `Foo|Bar.cs::Method` | `0x02000001 / 0x06000001` | `src/reconstructed/managed/full/CSharpScript/shards/0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef/source/Test.cs:1-2` | `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa` |\n";
        let rows = parse_gameplay_tables(markdown);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cells[0], "58");
        assert!(header_has_body_sha(&rows[0].header[4]));
        assert!(is_lower_hex_64(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ));
        assert!(!header_has_body_sha("Shard SHA256"));
        assert_eq!(
            identity_pair(&rows[0].cells[2]),
            Some(("0x02000001".to_string(), "0x06000001".to_string()))
        );
        assert_eq!(
            identity_pair("`0x02000001 / 0x06000001`; associated `0x02000002 / 0x06000002`"),
            None
        );
        assert_eq!(
            identity_pair("`0x02000001 / SomeType getter 0x06000001`"),
            None
        );
        assert_eq!(identity_pair("0x02000001 0x06000001"), None);
        let (_, range) = path_and_range(&rows[0].cells[3]).unwrap();
        let body = rows[0].cells[4].trim_matches('`').to_string();
        assert_eq!(
            gameplay_anchor_class(true, range.as_ref(), Some(&body)),
            "EXPLICIT_RANGE_WITH_BODY_SHA"
        );
        assert_eq!(
            gameplay_anchor_class(true, range.as_ref(), None),
            "EXPLICIT_RANGE_NO_BODY_SHA"
        );
        assert_eq!(
            gameplay_anchor_class(true, None, None),
            "FILE_OR_METHOD_ANCHOR_ONLY"
        );
        let single_line = "`src/reconstructed/managed/full/CSharpScript/shards/0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef/source/Test.cs:1`";
        assert_eq!(path_and_range(single_line).unwrap().1, None);
        let getter =
            "`src/reconstructed/managed/semantic/gameplay/RoleElementComponent.cs:getter 42-45`";
        let (getter_path, getter_range) = path_and_range(getter).unwrap();
        assert_eq!(
            getter_path,
            "src/reconstructed/managed/semantic/gameplay/RoleElementComponent.cs"
        );
        assert_eq!(
            getter_range,
            Some(SourceRange {
                start_line: 42,
                end_line: 45
            })
        );
    }

    #[test]
    fn performance_tiers_are_actual_22_exact_12_inferred_1_receipt_3_doc_only() {
        let root = repo_root();
        let baseline = root.join(
            "github_src/tools/re/local-recovery/full-corpus-preserve01/baseline.artifacts.jsonl",
        );
        let mut paths = BTreeSet::new();
        for line in fs::read_to_string(baseline).unwrap().lines() {
            if line.is_empty() {
                continue;
            }
            let value = crate::parse_strict_value(line.as_bytes()).unwrap();
            let path = value_str(&value, "path").unwrap();
            if path.starts_with(PERFORMANCE_PREFIX) && path.ends_with(".cs") {
                paths.insert(path.to_string());
            }
        }
        let exact = BTreeSet::from([
            "src/reconstructed/performance/managed/entity/Event.Emit.Enum2.DirectCIL.cs",
            "src/reconstructed/performance/managed/entity/Event.EmitInternal.DirectCIL.cs",
            "src/reconstructed/performance/managed/entity/Event.ExecuteHandlesInternal.cs",
            "src/reconstructed/performance/managed/entity/Event.RangeReceiver.cs",
            "src/reconstructed/performance/managed/loading/GameModePromise.cs",
            "src/reconstructed/performance/managed/Timer.cs",
            "src/reconstructed/performance/managed/TimerSystemInstance.cs",
            "src/reconstructed/performance/TaskGraph.cs",
            "src/reconstructed/performance/TaskUtils.cs",
            "src/reconstructed/performance/PriorityQueue.cs",
            "src/reconstructed/performance/TaskSystem.cs",
            "src/reconstructed/performance/TaskBase.cs",
            "src/reconstructed/performance/Queue.cs",
            "src/reconstructed/performance/AsyncTask.cs",
            "src/reconstructed/performance/Completion.cs",
            "src/reconstructed/performance/SubLevelController.cs",
            "src/reconstructed/performance/managed/loading/LoadMapController.cs",
            "src/reconstructed/performance/managed/loading/SubLevelController.cs",
            "src/reconstructed/performance/managed/loading/WorldGlobal.cs",
            "src/reconstructed/performance/managed/loading/SubLevel.cs",
            "src/reconstructed/performance/managed/loading/SubLevel.Visibility.cs",
            "src/reconstructed/performance/managed/entity/RangeComponent.cs",
        ]);
        let inferred = BTreeSet::from([
            "src/reconstructed/performance/managed/tick/Core.cs",
            "src/reconstructed/performance/managed/tick/Ticker.cs",
            "src/reconstructed/performance/managed/tick/TickSystem.cs",
            "src/reconstructed/performance/managed/tick/Time.cs",
            "src/reconstructed/performance/managed/entity/Entity.cs",
            "src/reconstructed/performance/managed/entity/EntityComponent.cs",
            "src/reconstructed/performance/managed/entity/EntitySystem.cs",
            "src/reconstructed/performance/managed/entity/TickComponentInfo.cs",
            "src/reconstructed/performance/managed/entity/TickComponentManager.cs",
            "src/reconstructed/performance/managed/entity/TickEntityGroup.cs",
            "src/reconstructed/performance/managed/entity/TickEntityManager.cs",
            "src/reconstructed/performance/managed/entity/RangeComponent.PendingEvents.cs",
        ]);
        let receipt = BTreeSet::from([
            "src/reconstructed/performance/managed/entity/PendingEventBoundaries.cs",
        ]);
        let doc_only = BTreeSet::from([
            "src/reconstructed/performance/managed/entity/EventArgs.Invoke.Enum2.cs",
            "src/reconstructed/performance/managed/entity/EventSystem.EmitWithTarget.Enum2.cs",
            "src/reconstructed/performance/managed/entity/RangeComponent.LifecycleRecovery.cs",
        ]);
        assert_eq!(exact.len(), 22);
        assert_eq!(inferred.len(), 12);
        assert_eq!(receipt.len(), 1);
        assert_eq!(doc_only.len(), 3);
        assert_eq!(paths.len(), 38);
        for path in exact
            .iter()
            .chain(inferred.iter())
            .chain(receipt.iter())
            .chain(doc_only.iter())
        {
            assert!(
                paths.contains(*path),
                "missing performance tier member {path}"
            );
        }
        let all = exact
            .union(&inferred)
            .cloned()
            .collect::<BTreeSet<_>>()
            .union(&receipt)
            .cloned()
            .collect::<BTreeSet<_>>()
            .union(&doc_only)
            .cloned()
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        assert_eq!(all, paths);
    }

    #[test]
    fn performance_method_tokens_resolve_uniquely_in_actual_canonical_authority() {
        let root = repo_root();
        let baseline = root.join(
            "github_src/tools/re/local-recovery/full-corpus-preserve01/baseline.artifacts.jsonl",
        );
        let mut tokens: BTreeMap<String, u32> = BTreeMap::new();
        for line in fs::read_to_string(baseline).unwrap().lines() {
            if line.is_empty() {
                continue;
            }
            let value = crate::parse_strict_value(line.as_bytes()).unwrap();
            let path = value_str(&value, "path").unwrap();
            if !is_canonical_provenance(path) {
                continue;
            }
            let text = fs::read_to_string(root.join(path)).unwrap();
            for row in text.lines() {
                if row.is_empty() {
                    continue;
                }
                let token = row
                    .find("\"method_token\":\"")
                    .map(|index| index + "\"method_token\":\"".len())
                    .and_then(|start| {
                        row[start..]
                            .find('"')
                            .map(|end| row[start..start + end].to_string())
                    });
                if let Some(token) = token {
                    *tokens.entry(token.to_lowercase()).or_insert(0) += 1;
                }
            }
        }
        let mut checked = 0u64;
        let mut check = |token: &str| {
            assert_eq!(
                tokens.get(&token.to_lowercase()),
                Some(&1),
                "performance method token {token} is not unique in canonical authority"
            );
            checked += 1;
        };
        let p02: Value = crate::parse_strict_value(
            &fs::read(root.join("evidence/performance/PERF02_EVIDENCE.json")).unwrap(),
        )
        .unwrap();
        for member in p02
            .pointer("/projection_method_mapping/members")
            .and_then(Value::as_array)
            .unwrap()
        {
            if matches!(
                member.get("projection").and_then(Value::as_str),
                Some("AsyncTask.cs") | Some("Completion.cs") | Some("SubLevelController.cs")
            ) {
                check(member.get("method_token").and_then(Value::as_str).unwrap());
            }
        }
        let p04: Value = crate::parse_strict_value(
            &fs::read(root.join("evidence/performance/PERF04_EVIDENCE.json")).unwrap(),
        )
        .unwrap();
        for member in p04
            .get("method_inventory")
            .and_then(Value::as_array)
            .unwrap()
        {
            let projection = member.get("projection").and_then(Value::as_str).unwrap();
            if matches!(
                short_name(projection).as_deref(),
                Some("LoadMapController.cs")
                    | Some("SubLevelController.cs")
                    | Some("WorldGlobal.cs")
                    | Some("SubLevel.cs")
            ) {
                check(member.get("method_token").and_then(Value::as_str).unwrap());
            }
        }
        let p04b: Value = crate::parse_strict_value(
            &fs::read(root.join("evidence/performance/PERF04B_EVIDENCE.json")).unwrap(),
        )
        .unwrap();
        for token in p04b
            .pointer("/source_reconstruction/source_body_tokens")
            .and_then(Value::as_array)
            .unwrap()
        {
            check(token.as_str().unwrap());
        }
        let p09: Value = crate::parse_strict_value(
            &fs::read(root.join("evidence/performance/PERF09_EVIDENCE.json")).unwrap(),
        )
        .unwrap();
        for method in p09
            .pointer("/source_checks/method_checks")
            .and_then(Value::as_array)
            .unwrap()
        {
            check(method.get("method_token").and_then(Value::as_str).unwrap());
        }
        let p11: Value = crate::parse_strict_value(
            &fs::read(root.join("evidence/performance/PERF11_EVIDENCE.json")).unwrap(),
        )
        .unwrap();
        for method in p11
            .get("selected_methods")
            .and_then(Value::as_array)
            .unwrap()
        {
            if method
                .get("name")
                .and_then(Value::as_str)
                .unwrap()
                .starts_with("RangeComponent.")
            {
                check(method.get("methoddef").and_then(Value::as_str).unwrap());
            }
        }
        for name in [
            "evidence/performance/PERF05_EVIDENCE.json",
            "evidence/performance/PERF07_EVIDENCE.json",
            "evidence/performance/PERF08_EVIDENCE.json",
        ] {
            let value: Value =
                crate::parse_strict_value(&fs::read(root.join(name)).unwrap()).unwrap();
            let methods = value
                .get("method_inventory")
                .or_else(|| value.get("selected_method_inventory"))
                .and_then(Value::as_array)
                .unwrap();
            for method in methods {
                if let Some(token) = method.get("method_token").and_then(Value::as_str) {
                    check(token);
                }
            }
        }
        assert_eq!(checked, 115);
    }

    #[test]
    fn authority_negative_bindings_fail_closed() {
        let expected = BTreeSet::from(["R5B2B-P1-P0".to_string(), "R5B2B-P1-P1".to_string()]);
        let omitted = BTreeSet::from(["R5B2B-P1-P0".to_string()]);
        assert!(require_authority_set("omitted preseed shard", &expected, &omitted).is_err());
        let left = BTreeSet::from(["0x02000001".to_string()]);
        let right = BTreeSet::from(["0x02000001".to_string()]);
        assert!(require_authority_disjoint("root collision", &left, &right).is_err());
        assert!(require_authority_count("declared method sum", 303456, 303455).is_err());
    }

    #[test]
    fn p1_overlap_helper_requires_full_qualified_identity() {
        let base = MethodKey {
            assembly_sha256: "a".repeat(64),
            mvid: "m".to_string(),
            type_token: "0x02000001".to_string(),
            method_token: "0x06000001".to_string(),
        };
        let other = MethodKey {
            assembly_sha256: "a".repeat(64),
            mvid: "m".to_string(),
            type_token: "0x02000002".to_string(),
            method_token: "0x06000002".to_string(),
        };
        let canonical = BTreeSet::from([base.sort_key()]);
        let p1 = BTreeSet::from([base.sort_key(), other.sort_key()]);
        assert_eq!(key_set_overlap(&canonical, &p1), 1);
        let same_md_different_td = BTreeSet::from([MethodKey {
            assembly_sha256: "a".repeat(64),
            mvid: "m".to_string(),
            type_token: "0x02000002".to_string(),
            method_token: "0x06000001".to_string(),
        }
        .sort_key()]);
        assert_eq!(key_set_overlap(&canonical, &same_md_different_td), 0);
        let same_md_different_authority = BTreeSet::from([MethodKey {
            assembly_sha256: "b".repeat(64),
            mvid: "m".to_string(),
            type_token: "0x02000001".to_string(),
            method_token: "0x06000001".to_string(),
        }
        .sort_key()]);
        assert_eq!(key_set_overlap(&canonical, &same_md_different_authority), 0);
    }

    #[test]
    fn association_summary_actual_catalogs_are_9_types_17_methods_11_generated_6_explicit() {
        let root = repo_root();
        let v5 = crate::parse_strict_value(
            &fs::read(root.join("github_src/metadata/imports.v5.json")).unwrap(),
        )
        .unwrap();
        let mut catalogs = Vec::new();
        for name in [
            "performance-source.v1.json",
            "scheduling-source.v1.json",
            "task-execution-source.v1.json",
        ] {
            catalogs.push(
                crate::parse_strict_value(
                    &fs::read(root.join("github_src/metadata").join(name)).unwrap(),
                )
                .unwrap(),
            );
        }
        let summary = association_summary(&v5, &catalogs).unwrap();
        assert_eq!(summary["associated_type_count"], 9);
        assert_eq!(summary["nested_method_count"], 17);
        assert_eq!(summary["generated_method_association_count"], 11);
        assert_eq!(summary["explicit_nested_member_count"], 6);
        assert_eq!(summary["queue_generated_count"], 0);
        assert_eq!(summary["additional_csharp_declarations"], 0);
        let mapping = summary["mapping"].as_array().unwrap();
        assert_eq!(mapping.len(), 9);
        let queue = mapping
            .iter()
            .filter(|entry| entry["record_id"] == "PEXEC_QUEUE")
            .collect::<Vec<_>>();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0]["association_class"], "EXPLICIT_NESTED_MEMBER");
        assert_eq!(queue[0]["nested_method_count"], 6);
    }

    #[test]
    fn frozen_v5_r38_statuses_remain_lagging() {
        let path = repo_root().join("github_src/metadata/imports.v5.json");
        let value = crate::parse_strict_value(&fs::read(path).unwrap()).unwrap();
        let imports = value_array(&value, "imports").unwrap();
        let d = imports
            .iter()
            .find(|entry| value_str(entry, "id").unwrap() == "D")
            .unwrap();
        let mut statuses = BTreeMap::new();
        for method in value_array(d, "methods").unwrap() {
            let identity = method.get("method_identity").unwrap();
            let md = value_str(identity, "methodDef").unwrap().to_lowercase();
            let evidence = value_str(method, "evidence_status").unwrap();
            statuses.insert(md, evidence.to_string());
        }
        assert_eq!(
            statuses.get("0x06018867").map(String::as_str),
            Some("DIRECT_CIL_FULL")
        );
        assert_eq!(
            statuses.get("0x06018865").map(String::as_str),
            Some("SOURCE_ONLY")
        );
        for md in [
            "0x06018868",
            "0x06018869",
            "0x0601886a",
            "0x0601886b",
            "0x0601886c",
            "0x06018873",
        ] {
            assert_eq!(
                statuses.get(md).map(String::as_str),
                Some("UNKNOWN"),
                "{md}"
            );
        }
    }
}
