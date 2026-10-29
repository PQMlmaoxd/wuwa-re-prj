use re_corpus::provenance::{
    AcceptedMethodMap, ERRATA_CAMPAIGN, ERRATA_SCHEMA, ERRATA_VERSION, ErrataAction,
    ErrataDocument, ErrataEntry, EvidenceAnchor, EvidenceBasis, EvidenceClass, FrozenFile,
    ImportedAnchor, IncorrectAnchor, METHOD_EVIDENCE_MAP, Outcome, Precision, ReviewStatus,
    RootKind, SupportRef, body_slice_sha256_lf, canonical_record_hash, checked_file_bytes,
    parse_method_map, validate_relative_metadata_path,
};
use re_corpus::{MethodIdentity, sha256_hex};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

struct JunctionGuard {
    link: PathBuf,
    active: bool,
}

impl JunctionGuard {
    fn unlink(&mut self) -> bool {
        if !self.active {
            return true;
        }
        match fs::remove_dir(&self.link) {
            Ok(()) => {
                self.active = false;
                true
            }
            Err(_) => false,
        }
    }
}

impl Drop for JunctionGuard {
    fn drop(&mut self) {
        let _ = self.unlink();
    }
}

struct FixtureRoot {
    path: PathBuf,
    junctions: Vec<JunctionGuard>,
}

impl FixtureRoot {
    fn new(label: &str) -> Self {
        let n = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("provenance-test-fixtures")
            .join(format!("{label}-{}-{n}", std::process::id()));
        fs::create_dir_all(&path).expect("create owned fixture");
        Self {
            path,
            junctions: Vec::new(),
        }
    }

    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(path, bytes).expect("write fixture");
    }

    fn junction(&mut self, relative: &str, target: &Path) {
        let link = self.path.join(relative);
        if let Some(parent) = link.parent() {
            fs::create_dir_all(parent).expect("create junction parent");
        }
        let link_text = link.display().to_string().replace('/', "\\");
        let target_text = target.display().to_string().replace('/', "\\");
        let status = Command::new("cmd")
            .args(["/C", "mklink", "/J", &link_text, &target_text])
            .status()
            .expect("run mklink");
        assert!(
            status.success(),
            "mklink failed: {link_text} -> {target_text}"
        );
        self.junctions.push(JunctionGuard { link, active: true });
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let mut clean = true;
        for junction in &mut self.junctions {
            if !junction.unlink() {
                clean = false;
            }
        }
        if clean {
            let _ = fs::remove_dir_all(&self.path);
        } else {
            // Keep the owned fixture tree intact rather than recursively
            // walking a still-live junction into its target.
            eprintln!(
                "retaining owned fixture after junction unlink failure: {}",
                self.path.display()
            );
        }
    }
}

const AUTHORITY: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const MVID: &str = "00000000-0000-0000-0000-000000000000";
const SEMANTIC: &str = "src/reconstructed/managed/semantic/gameplay/Test.cs";
const DESTINATION: &str = "src/managed/test/Test.cs";

const CANONICAL: &str = "src/reconstructed/managed/full/CSharpScript/shards/test-source/Test.cs";

fn line_source() -> String {
    (1..=10)
        .map(|line| format!("line {line}\n"))
        .collect::<String>()
}

fn map_fixture() -> String {
    format!(
        "| # | Overlay method | TypeDef / MethodDef | Canonical source and lines | Raw methods row | body bytes | Fresh CIL anchor or evidence level | FULL_SELECTED_BODY / unknown dependency chain |\n|---:|---|---|---|---|---:|---|---|\n| 1 | `Test.cs::M` | `0x02000001 / 0x06000001` | `{CANONICAL}:6-9` | `evidence/raw/methods.jsonl:1` | 1 | `source` | `{SEMANTIC}:2-4` |\n"
    )
}

fn support_ref(path: &str, locator: &str, sha: &str) -> SupportRef {
    SupportRef {
        path: path.to_owned(),
        locator: locator.to_owned(),
        artifact_sha256: Some(sha.to_owned()),
    }
}

fn identity() -> MethodIdentity {
    MethodIdentity {
        full_authority_sha256: AUTHORITY.to_owned(),
        mvid: MVID.to_owned(),
        type_def: "0x02000001".to_owned(),
        method_def: "0x06000001".to_owned(),
    }
}

fn build_document() -> (FixtureRoot, ErrataDocument, Value) {
    let fixture = FixtureRoot::new("valid");
    let source = line_source();
    fixture.write(SEMANTIC, source.as_bytes());
    fixture.write(CANONICAL, source.as_bytes());
    fixture.write(DESTINATION, source.as_bytes());
    let source_sha = sha256_hex(source.as_bytes());
    let map = map_fixture();
    fixture.write(METHOD_EVIDENCE_MAP, map.as_bytes());
    let map_sha = sha256_hex(map.as_bytes());
    let map_line = 3u64;
    let record = json!({
        "id": "G_TEST",
        "cohort": "gameplay66",
        "source_path": SEMANTIC,
        "source_sha256": source_sha,
        "destination_path": DESTINATION,
        "authority_sha256": AUTHORITY,
        "authority_mvid": MVID,
        "methods": [{
            "type_def": "0x02000001",
            "method_def": "0x06000001",
            "evidence_references": [{
                "path": SEMANTIC,
                "locator": "source lines 6-9",
                "artifact_sha256": null
            }]
        }]
    });
    let record_hash = canonical_record_hash(&record).expect("record hash");
    let catalog_value = json!({ "records": [record.clone()] });
    let catalog_bytes = serde_json::to_vec(&catalog_value).expect("catalog bytes");
    fixture.write("metadata/catalog.json", &catalog_bytes);
    let index_bytes = b"{}\n";
    fixture.write("metadata/index.jsonl", index_bytes);
    let identity = identity();
    let entry = ErrataEntry {
        target_catalog_record_id: "G_TEST".to_owned(),
        expected_original_record_hash: record_hash,
        method_identity: identity.clone(),
        action: ErrataAction::Correct,
        incorrect_anchors: vec![IncorrectAnchor {
            path: SEMANTIC.to_owned(),
            locator: "source lines 6-9".to_owned(),
            artifact_sha256: None,
        }],
        corrected_imported_source_anchor: ImportedAnchor {
            root: RootKind::CuratedRoot,
            path: DESTINATION.to_owned(),
            whole_file_sha256: source_sha.clone(),
            method_identity: identity.clone(),
            precision: Precision::ExactRange,
            start_line: Some(2),
            end_line: Some(4),
            scope: "selected method body".to_owned(),
            support_refs: vec![support_ref(METHOD_EVIDENCE_MAP, "accepted row 1", &map_sha)],
        },
        corrected_evidence_source_anchors: vec![
            EvidenceAnchor {
                root: RootKind::ResearchRoot,
                path: METHOD_EVIDENCE_MAP.to_owned(),
                artifact_sha256: map_sha.clone(),
                method_identity: identity.clone(),
                evidence_class: EvidenceClass::AcceptedMethodMap,
                precision: Precision::FileMethodAssociation,
                start_line: None,
                end_line: None,
                accepted_map_row: Some(1),
                accepted_map_line: Some(map_line),
                basis: "accepted map row".to_owned(),
                support_refs: Vec::new(),
            },
            EvidenceAnchor {
                root: RootKind::ResearchRoot,
                path: SEMANTIC.to_owned(),
                artifact_sha256: source_sha.clone(),
                method_identity: identity.clone(),
                evidence_class: EvidenceClass::SemanticSource,
                precision: Precision::ExactRange,
                start_line: Some(2),
                end_line: Some(4),
                accepted_map_row: Some(1),
                accepted_map_line: Some(map_line),
                basis: "accepted semantic source range".to_owned(),
                support_refs: Vec::new(),
            },
            EvidenceAnchor {
                root: RootKind::ResearchRoot,
                path: CANONICAL.to_owned(),
                artifact_sha256: source_sha.clone(),
                method_identity: identity.clone(),
                evidence_class: EvidenceClass::CanonicalSource,
                precision: Precision::ExactRange,
                start_line: Some(6),
                end_line: Some(9),
                accepted_map_row: Some(1),
                accepted_map_line: Some(map_line),
                basis: "accepted canonical cell range".to_owned(),
                support_refs: Vec::new(),
            },
        ],
        evidence_basis: EvidenceBasis {
            accepted_map_path: METHOD_EVIDENCE_MAP.to_owned(),
            accepted_map_sha256: map_sha.clone(),
            accepted_map_row: 1,
            accepted_map_line: map_line,
            basis: "accepted map row 1".to_owned(),
            proof_refs: Vec::new(),
        },
        correction_class: re_corpus::provenance::CorrectionClass::WrongMethodBody,
        review_status: ReviewStatus::PrimaryReviewPending,
        outcome: Outcome::ResolvedExact,
        profile: re_corpus::provenance::ProvenanceProfile::OrdinaryGameplayMap,
        target_catalog_path: None,
        association_proof: None,
    };
    let document = ErrataDocument {
        schema: ERRATA_SCHEMA.to_owned(),
        version: ERRATA_VERSION,
        campaign: ERRATA_CAMPAIGN.to_owned(),
        catalog: FrozenFile {
            path: "metadata/catalog.json".to_owned(),
            bytes: catalog_bytes.len() as u64,
            sha256: sha256_hex(&catalog_bytes),
        },
        index: FrozenFile {
            path: "metadata/index.jsonl".to_owned(),
            bytes: index_bytes.len() as u64,
            sha256: sha256_hex(index_bytes),
        },
        accepted_map: FrozenFile {
            path: METHOD_EVIDENCE_MAP.to_owned(),
            bytes: map.len() as u64,
            sha256: map_sha,
        },
        entries: vec![entry],
        bound_catalogs: Vec::new(),
        v5_receipt: None,
    };
    (fixture, document, record)
}

fn roots(fixture: &FixtureRoot) -> (PathBuf, PathBuf) {
    (fixture.path.clone(), fixture.path.clone())
}

#[test]
fn valid_exact_correction_uses_accepted_map_and_actual_bytes() {
    let (fixture, document, _) = build_document();
    let (curated, research) = roots(&fixture);
    document
        .validate_against_roots(&curated, &research)
        .expect("valid correction");
}

#[test]
fn wrong_file_evidence_anchor_is_rejected() {
    let (fixture, mut document, _) = build_document();
    let other = SEMANTIC.replace("Test.cs", "Other.cs");
    fixture.write(&other, line_source().as_bytes());
    let semantic = document.entries[0]
        .corrected_evidence_source_anchors
        .iter_mut()
        .find(|anchor| anchor.evidence_class == EvidenceClass::SemanticSource)
        .expect("semantic anchor");
    semantic.path = other;
    let (curated, research) = roots(&fixture);
    let error = document
        .validate_against_roots(&curated, &research)
        .expect_err("wrong file must fail");
    assert_eq!(error.code, "SEMANTIC_ANCHOR_PATH_MISMATCH");
}

#[test]
fn resealed_imported_file_tamper_is_rejected_by_source_binding() {
    let (fixture, mut document, _) = build_document();
    let tampered = line_source().replace("line 1\n", "tampered\n");
    fixture.write(DESTINATION, tampered.as_bytes());
    document.entries[0]
        .corrected_imported_source_anchor
        .whole_file_sha256 = sha256_hex(tampered.as_bytes());
    let (curated, research) = roots(&fixture);
    let error = document
        .validate_against_roots(&curated, &research)
        .expect_err("resealed imported bytes must not pass");
    assert_eq!(error.code, "IMPORTED_SOURCE_BYTES_MISMATCH");
}

#[test]
fn canonical_exact_anchor_must_match_accepted_canonical_cell() {
    let (fixture, mut document, _) = build_document();
    let other = "src/reconstructed/managed/full/CSharpScript/shards/other/Other.cs";
    fixture.write(other, line_source().as_bytes());
    let canonical = document.entries[0]
        .corrected_evidence_source_anchors
        .iter_mut()
        .find(|anchor| anchor.evidence_class == EvidenceClass::CanonicalSource)
        .expect("canonical anchor");
    canonical.path = other.to_owned();
    let (curated, research) = roots(&fixture);
    let error = document
        .validate_against_roots(&curated, &research)
        .expect_err("wrong canonical file must fail");
    assert_eq!(error.code, "CANONICAL_ANCHOR_PATH_MISMATCH");
}

#[test]
fn report_or_raw_anchor_without_adapter_is_rejected() {
    let (fixture, mut document, _) = build_document();
    let semantic = document.entries[0]
        .corrected_evidence_source_anchors
        .iter_mut()
        .find(|anchor| anchor.evidence_class == EvidenceClass::SemanticSource)
        .expect("semantic anchor");
    semantic.evidence_class = EvidenceClass::Report;
    let (curated, research) = roots(&fixture);
    let error = document
        .validate_against_roots(&curated, &research)
        .expect_err("unbound report anchor must fail");
    assert_eq!(error.code, "UNVERIFIED_EVIDENCE_ANCHOR");
}

#[test]
fn support_reference_must_be_from_accepted_metadata_or_marked_resolved() {
    let (fixture, mut document, _) = build_document();
    document.entries[0]
        .corrected_imported_source_anchor
        .support_refs[0]
        .path = "evidence/not-accepted.json".to_owned();
    let (curated, research) = roots(&fixture);
    let error = document
        .validate_against_roots(&curated, &research)
        .expect_err("unverified support reference must fail");
    assert_eq!(error.code, "IMPORTED_SUPPORT_REFERENCE_NOT_CHECKED");
}

#[test]
fn unresolved_proposal_shape_is_accounted_but_activation_is_rejected() {
    let (fixture, mut document, _) = build_document();
    let entry = &mut document.entries[0];
    entry.review_status = ReviewStatus::PrimaryApproved;
    entry.outcome = Outcome::UnresolvedInsufficientEvidence;
    entry.corrected_imported_source_anchor.precision = Precision::Unresolved;
    entry.corrected_imported_source_anchor.start_line = None;
    entry.corrected_imported_source_anchor.end_line = None;
    document
        .validate_shape()
        .expect("unresolved proposal shape is allowed for accounting");
    let (curated, research) = roots(&fixture);
    let error = document
        .validate_for_activation(&curated, &research)
        .expect_err("unresolved activation must fail");
    assert_eq!(error.code, "ACTIVATION_OUTCOME_UNRESOLVED_OR_CONFLICTING");
}

#[test]
fn pending_and_rejected_reviews_are_not_activatable() {
    let (fixture, mut document, _) = build_document();
    let (curated, research) = roots(&fixture);
    let error = document
        .validate_for_activation(&curated, &research)
        .expect_err("pending activation must fail");
    assert_eq!(error.code, "ACTIVATION_REVIEW_NOT_APPROVED");

    document.entries[0].review_status = ReviewStatus::PrimaryRejected;
    let error = document
        .validate_against_roots(&curated, &research)
        .expect_err("rejected proposal must fail");
    assert_eq!(error.code, "REVIEW_REJECTED");
}

#[test]
fn approved_resolved_proposal_is_activatable() {
    let (fixture, mut document, _) = build_document();
    document.entries[0].review_status = ReviewStatus::PrimaryApproved;
    let (curated, research) = roots(&fixture);
    document
        .validate_for_activation(&curated, &research)
        .expect("approved resolved proposal");
}

#[test]
fn out_of_order_entries_are_rejected() {
    let (_fixture, mut document, _) = build_document();
    let mut earlier = document.entries[0].clone();
    earlier.target_catalog_record_id = "A_TEST".to_owned();
    document.entries.push(earlier);
    let error = document
        .validate_shape()
        .expect_err("out-of-order must fail");
    assert_eq!(error.code, "ERRATA_OUT_OF_ORDER");
}

#[test]
fn promotion_to_file_level_with_exact_range_is_rejected() {
    let (_fixture, mut document, _) = build_document();
    document.entries[0].outcome = Outcome::ResolvedConservative;
    let error = document
        .validate_shape()
        .expect_err("exact range cannot be relabeled conservative");
    assert_eq!(error.code, "CONSERVATIVE_OUTCOME_HAS_EXACT_IMPORTED_ANCHOR");
}

#[test]
fn inbounds_wrong_method_body_is_rejected() {
    let (fixture, mut document, _) = build_document();
    let entry = &mut document.entries[0];
    entry.corrected_imported_source_anchor.start_line = Some(6);
    entry.corrected_imported_source_anchor.end_line = Some(9);
    let (curated, research) = roots(&fixture);
    let error = document
        .validate_against_roots(&curated, &research)
        .expect_err("wrong in-bounds range must fail");
    assert_eq!(error.code, "IMPORTED_RANGE_DOES_NOT_MATCH_ACCEPTED_MAP");
}

#[test]
fn out_of_bounds_evidence_anchor_is_rejected() {
    let (fixture, mut document, _) = build_document();
    let entry = &mut document.entries[0];
    let semantic = entry
        .corrected_evidence_source_anchors
        .iter_mut()
        .find(|anchor| anchor.evidence_class == EvidenceClass::SemanticSource)
        .expect("semantic anchor");
    semantic.start_line = Some(20);
    semantic.end_line = Some(22);
    let (curated, research) = roots(&fixture);
    let error = document
        .validate_against_roots(&curated, &research)
        .expect_err("out-of-bounds evidence must fail");
    assert_eq!(error.code, "EVIDENCE_ANCHOR_RANGE_OUT_OF_BOUNDS");
}

#[test]
fn stale_target_hash_is_rejected() {
    let (fixture, mut document, _) = build_document();
    document.entries[0].expected_original_record_hash = "0".repeat(64);
    let (curated, research) = roots(&fixture);
    let error = document
        .validate_against_roots(&curated, &research)
        .expect_err("stale hash must fail");
    assert_eq!(error.code, "STALE_TARGET_RECORD_HASH");
}

#[test]
fn duplicate_target_corrections_are_rejected_before_file_access() {
    let (_fixture, mut document, _) = build_document();
    let duplicate = document.entries[0].clone();
    document.entries.push(duplicate);
    let error = document
        .validate_shape()
        .expect_err("competing correction must fail");
    assert_eq!(error.code, "COMPETING_ERRATA_TARGET");
}

#[test]
fn competing_target_with_case_changed_identity_is_rejected() {
    let (_fixture, mut document, _) = build_document();
    let mut duplicate = document.entries[0].clone();
    duplicate.target_catalog_record_id = "g_test".to_owned();
    duplicate.method_identity.type_def = "0x02000001".to_owned();
    duplicate.method_identity.method_def = "0x06000001".to_owned();
    document.entries.push(duplicate);
    let error = document
        .validate_shape()
        .expect_err("case-changed target must still compete");
    assert_eq!(error.code, "COMPETING_ERRATA_TARGET");
}

#[test]
fn duplicate_json_keys_are_rejected() {
    let bytes = br#"{"schema":"a","schema":"b"}"#;
    let error = ErrataDocument::parse(bytes).expect_err("duplicate key");
    assert_eq!(error.code, "DUPLICATE_JSON_KEY");
}

#[test]
fn unsafe_relative_paths_and_junctions_are_rejected() {
    let fixture = FixtureRoot::new("junction");
    let outside = fixture.path.join("outside");
    fs::create_dir_all(&outside).expect("outside");
    fs::write(outside.join("file.txt"), b"outside\n").expect("outside file");
    let mut fixture = fixture;
    fixture.junction("root/link", &outside);
    fixture.junction("root/leaf", &outside);
    let root = fixture.path.join("root");
    let error = checked_file_bytes(&root, "link/file.txt").expect_err("ancestor junction");
    assert_eq!(error.code, "UNSAFE_REPARSE_OR_SYMLINK");
    let error = checked_file_bytes(&root, "leaf").expect_err("leaf junction");
    assert_eq!(error.code, "UNSAFE_REPARSE_OR_SYMLINK");
    let error = validate_relative_metadata_path("../outside").expect_err("traversal");
    assert_eq!(error.code, "UNSAFE_RELATIVE_PATH");
}

#[test]
fn map_parser_requires_header_and_does_not_use_canonical_range_as_semantic() {
    let map = format!(
        "| # | Overlay method | TypeDef / MethodDef | Canonical source and lines | Raw methods row | body bytes | Fresh CIL anchor or evidence level | FULL_SELECTED_BODY / unknown dependency chain |\n|---:|---|---|---|---|---:|---|---|\n| 1 | `Test.cs::M` | `0x02000001 / 0x06000001` | `src/reconstructed/managed/full/other.cs:6-9` | `methods.jsonl:1` | 1 | `source` | `{SEMANTIC}:2-4` |\n"
    );
    let parsed: AcceptedMethodMap = parse_method_map(map.as_bytes()).expect("map parse");
    let row = parsed.row_for("0x02000001", "0x06000001").expect("row");
    assert_eq!(row.semantic_path, SEMANTIC);
    assert_eq!((row.start_line, row.end_line), (Some(2), Some(4)));
    assert_eq!(
        row.canonical_path.as_deref(),
        Some("src/reconstructed/managed/full/other.cs")
    );
    assert!(parsed.row_for("0x02000001", "0x06000002").is_none());

    let short_canonical_header = "| # | Overlay method | TypeDef / MethodDef | Canonical source and lines |\n|---:|---|---|---|\n| 1 | `Test.cs::M` | `0x02000001 / 0x06000001` | `Test.cs:6-9` |\n";
    let parsed = parse_method_map(short_canonical_header.as_bytes()).expect("fallback parse");
    let row = parsed
        .row_for("0x02000001", "0x06000001")
        .expect("fallback row");
    assert_eq!(row.semantic_path, SEMANTIC);
    assert_eq!(
        (row.start_line, row.end_line, row.single_line),
        (None, None, None)
    );

    let headerless = format!("| 1 | `0x02000001 / 0x06000001` | `{SEMANTIC}:2-4` |\n");
    assert_eq!(
        parse_method_map(headerless.as_bytes())
            .expect_err("headerless row must not parse")
            .code,
        "MAP_HAS_NO_ACCEPTED_ROWS"
    );
    let type_binding = "| # | TypeDef / MethodDef | Semantic source |\n|---:|---|---|\n| 58 | `0x02000001 / 0x02000002` | `Test.cs:1-2` |\n";
    assert_eq!(
        parse_method_map(type_binding.as_bytes())
            .expect_err("type binding must not parse as method")
            .code,
        "MAP_HAS_NO_ACCEPTED_ROWS"
    );
    let malformed_pair = "| # | Overlay method | TypeDef / MethodDef | Semantic source |\n|---:|---|---|---|\n| 1 | `Test.cs::M` | `0x02000001 / not-a-method / 0x06000001` | `Test.cs:2-4` |\n";
    assert_eq!(
        parse_method_map(malformed_pair.as_bytes())
            .expect_err("text between slash and method token must fail")
            .code,
        "MAP_HAS_NO_ACCEPTED_ROWS"
    );
}

#[test]
fn actual_method_map_distribution_and_body_hashes() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repository root");
    let map_bytes = fs::read(repo_root.join(METHOD_EVIDENCE_MAP)).expect("actual method map");
    let map = parse_method_map(&map_bytes).expect("actual method map parse");
    let ranges = map
        .rows
        .iter()
        .filter(|row| row.start_line.is_some() && row.end_line.is_some())
        .count();
    let single = map
        .rows
        .iter()
        .filter(|row| row.single_line.is_some())
        .count();
    let file = map
        .rows
        .iter()
        .filter(|row| {
            row.start_line.is_none() && row.end_line.is_none() && row.single_line.is_none()
        })
        .count();
    let body = map
        .rows
        .iter()
        .filter(|row| row.body_sha256.is_some())
        .count();
    assert_eq!(map.rows.len(), 301);
    assert_eq!(ranges, 234);
    assert_eq!(single, 2);
    assert_eq!(file, 65);
    assert_eq!(body, 22);
    let report = map
        .validate_body_hashes(repo_root)
        .expect("actual body hashes must match");
    assert_eq!(report.rows, 22);
    assert_eq!(report.checked, 22);
    assert_eq!(report.mismatched, 0);
}

#[test]
fn actual_campaign_map_partition_is_field_derived() {
    let curated_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let research_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repository root");
    let map_bytes = fs::read(research_root.join(METHOD_EVIDENCE_MAP)).expect("actual method map");
    let map = parse_method_map(&map_bytes).expect("actual method map parse");
    let catalog_value: Value = serde_json::from_slice(
        &fs::read(curated_root.join("metadata/curation/bulk-curate01.json"))
            .expect("actual campaign catalog"),
    )
    .expect("catalog JSON");
    let mut campaign_keys = std::collections::BTreeSet::new();
    for record in catalog_value["records"].as_array().expect("records") {
        if record["cohort"].as_str() != Some("gameplay66") {
            continue;
        }
        for method in record["methods"].as_array().expect("methods") {
            campaign_keys.insert(format!(
                "{}/{}",
                method["type_def"].as_str().expect("type_def"),
                method["method_def"].as_str().expect("method_def")
            ));
        }
    }
    let campaign_rows: Vec<_> = map
        .rows
        .iter()
        .filter(|row| campaign_keys.contains(&format!("{}/{}", row.type_def, row.method_def)))
        .collect();
    assert_eq!(campaign_rows.len(), 292);
    assert_eq!(
        campaign_rows
            .iter()
            .filter(|row| row.start_line.is_some() && row.end_line.is_some())
            .count(),
        225
    );
    assert_eq!(
        campaign_rows
            .iter()
            .filter(|row| row.single_line.is_some())
            .count(),
        2
    );
    assert_eq!(
        campaign_rows
            .iter()
            .filter(|row| {
                row.start_line.is_none() && row.end_line.is_none() && row.single_line.is_none()
            })
            .count(),
        65
    );
    assert_eq!(
        campaign_rows
            .iter()
            .filter(|row| row.body_sha256.is_some())
            .count(),
        14
    );
}

#[test]
fn private_candidate_431_has_complete_key_set_and_explicit_unresolved_actions() {
    let curated = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let research = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repository root");
    let path =
        curated.join("tools/re/local-recovery/bulk-provenance-fix01/phase-b/CANDIDATE_431.json");
    let document =
        ErrataDocument::parse(&fs::read(path).expect("candidate")).expect("candidate shape");
    assert_eq!(document.entries.len(), 431);
    assert_eq!(
        document
            .entries
            .iter()
            .filter(|entry| entry.action == ErrataAction::Correct)
            .count(),
        184
    );
    assert_eq!(
        document
            .entries
            .iter()
            .filter(|entry| entry.action == ErrataAction::Confirm)
            .count(),
        108
    );
    assert_eq!(
        document
            .entries
            .iter()
            .filter(|entry| entry.action == ErrataAction::Refine)
            .count(),
        139
    );
    assert!(
        document.validate_against_roots(&curated, research).is_err(),
        "unresolved candidate must not become an activation/root PASS"
    );
}

#[test]
fn body_slice_is_one_based_inclusive_and_lf_normalized() {
    let bytes = b"one\r\ntwo\r\nthree\r\nfour\r\n";
    let digest = body_slice_sha256_lf(bytes, 2, 3).expect("slice");
    assert_eq!(digest, sha256_hex(b"two\nthree\n"));
    assert_eq!(
        body_slice_sha256_lf(bytes, 2, 9)
            .expect_err("out of range")
            .code,
        "SOURCE_RANGE_OUT_OF_BOUNDS"
    );
}

#[test]
fn canonical_record_hash_is_independent_of_object_key_order() {
    let first: Value = serde_json::from_str(r#"{"b":2,"a":{"z":true,"y":null}}"#).expect("first");
    let second: Value = serde_json::from_str(r#"{"a":{"y":null,"z":true},"b":2}"#).expect("second");
    assert_eq!(
        canonical_record_hash(&first).expect("first hash"),
        canonical_record_hash(&second).expect("second hash")
    );
}
