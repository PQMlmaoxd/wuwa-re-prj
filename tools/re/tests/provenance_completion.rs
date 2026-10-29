use re_corpus::provenance::{
    ASSOCIATED_BINDINGS, ERRATA_REL, ErrataAction, ErrataDocument, EvidenceClass, Outcome,
    PERFORMANCE_BINDINGS, Precision, ProvenanceProfile, ProvenanceTrust, ReviewStatus, SupportRef,
    canonical_record_hash,
};
use re_corpus::{
    load_active_receipt, run_lookup_at_with_trust, run_status_at_with_trust,
    run_verify_provenance_at_with_trust, sha256_hex,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

fn curated() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn research() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repository root")
        .to_path_buf()
}

fn candidate_bytes() -> Vec<u8> {
    fs::read(curated().join(
        "tools/re/local-recovery/bulk-provenance-fix01/phase-d-completion/CANDIDATE_533_D01.json",
    ))
    .expect("read CANDIDATE_533")
}

fn candidate() -> ErrataDocument {
    ErrataDocument::parse(&candidate_bytes()).expect("CANDIDATE_533 parses")
}

fn profile_counts(document: &ErrataDocument) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for entry in &document.entries {
        *counts.entry(entry.profile.as_str()).or_insert(0) += 1;
    }
    counts
}

#[test]
fn private_candidate_533_rederives_all_completion_profiles() {
    let document = candidate();
    assert_eq!(document.entries.len(), 533);
    let counts = profile_counts(&document);
    assert_eq!(counts.get("ORDINARY_GAMEPLAY_MAP"), Some(&292));
    assert_eq!(counts.get("REGISTRY_FILE_ASSOCIATION"), Some(&106));
    assert_eq!(counts.get("ASSOCIATED_GAMEPLAY_MAP"), Some(&18));
    assert_eq!(counts.get("PERFORMANCE_EVIDENCE_ASSOCIATION"), Some(&13));
    assert_eq!(counts.get("LEGACY_V5_SOURCE_ASSOCIATION"), Some(&102));
    assert_eq!(counts.get("MIGRATION_RECORD_ASSOCIATION"), Some(&2));
    assert!(counts.get("UNRESOLVED").is_none());
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
        81
    );
    assert_eq!(
        document
            .entries
            .iter()
            .filter(|entry| entry.action == ErrataAction::Refine)
            .count(),
        268
    );
    assert_eq!(
        document
            .entries
            .iter()
            .filter(|entry| entry.outcome == Outcome::ResolvedExact)
            .count(),
        231
    );
    assert_eq!(
        document
            .entries
            .iter()
            .filter(|entry| entry.outcome == Outcome::ResolvedConservative)
            .count(),
        302
    );

    let ordinary: Vec<_> = document
        .entries
        .iter()
        .filter(|entry| entry.profile == ProvenanceProfile::OrdinaryGameplayMap)
        .collect();
    assert_eq!(
        ordinary
            .iter()
            .filter(
                |entry| entry.corrected_imported_source_anchor.precision == Precision::ExactRange
            )
            .count(),
        225
    );
    assert_eq!(
        ordinary
            .iter()
            .filter(|entry| entry.corrected_imported_source_anchor.precision
                == Precision::FileMethodAssociation)
            .count(),
        67
    );
    let associated: Vec<_> = document
        .entries
        .iter()
        .filter(|entry| entry.profile == ProvenanceProfile::AssociatedGameplayMap)
        .collect();
    assert_eq!(
        associated
            .iter()
            .filter(
                |entry| entry.corrected_imported_source_anchor.precision == Precision::ExactRange
            )
            .count(),
        6
    );

    document
        .validate_against_roots(&curated(), &research())
        .expect("CANDIDATE_533 must rederive every profile from saved evidence");
    let pending = document
        .validate_for_activation(&curated(), &research())
        .expect_err("pending reviews remain blocked");
    assert_eq!(pending.code, "ACTIVATION_REVIEW_NOT_APPROVED");

    let mut approved = document.clone();
    for entry in &mut approved.entries {
        entry.review_status = ReviewStatus::PrimaryApproved;
    }
    approved
        .validate_for_activation(&curated(), &research())
        .expect("approved 533 candidate must be activatable after primary review");
}

#[test]
fn all_target_record_hashes_are_canonical_for_their_bound_catalog() {
    let document = candidate();
    let bulk: Value = serde_json::from_slice(
        &fs::read(curated().join("metadata/curation/bulk-curate01.json")).expect("bulk catalog"),
    )
    .expect("bulk catalog json");
    let mut catalogs = BTreeMap::new();
    for (path, key) in [
        ("metadata/migration.json", "records"),
        ("metadata/performance-source.v1.json", "records"),
        ("metadata/scheduling-source.v1.json", "records"),
        ("metadata/task-execution-source.v1.json", "records"),
    ] {
        let value: Value =
            serde_json::from_slice(&fs::read(curated().join(path)).expect("bound catalog bytes"))
                .expect("bound catalog json");
        let records = value[key]
            .as_array()
            .expect("records")
            .iter()
            .map(|item| item.get("record").cloned().unwrap_or_else(|| item.clone()))
            .collect::<Vec<_>>();
        catalogs.insert(path.to_owned(), records);
    }
    let bulk_records = bulk["records"].as_array().expect("bulk records");
    for entry in &document.entries {
        let records = entry
            .target_catalog_path
            .as_deref()
            .and_then(|path| catalogs.get(path))
            .map(Vec::as_slice)
            .unwrap_or(bulk_records);
        let record = records
            .iter()
            .find(|record| record["id"] == entry.target_catalog_record_id)
            .expect("target record");
        assert_eq!(
            canonical_record_hash(record).expect("canonical hash"),
            entry.expected_original_record_hash,
            "{}",
            entry.target_catalog_record_id
        );
    }
}

fn approved_533() -> ErrataDocument {
    let mut document = candidate();
    for entry in &mut document.entries {
        entry.review_status = ReviewStatus::PrimaryApproved;
    }
    document
}

struct FixtureRoot {
    path: PathBuf,
}

impl FixtureRoot {
    fn v6(label: &str) -> Self {
        let n = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("provenance-completion-fixtures")
            .join(format!("{label}-{}-{n}", std::process::id()));
        fs::create_dir_all(&path).expect("create owned fixture");
        for relative in [
            re_corpus::MANIFEST_REL,
            re_corpus::RECEIPT_REL,
            re_corpus::RECEIPT_V2_REL,
            re_corpus::RECEIPT_V3_REL,
            re_corpus::RECEIPT_V4_REL,
            re_corpus::RECEIPT_V5_REL,
            "metadata/performance-source.v1.json",
            "metadata/scheduling-source.v1.json",
            "metadata/task-execution-source.v1.json",
            "metadata/curation/bulk-curate01.json",
            "metadata/curation-batches.jsonl",
        ] {
            let destination = path.join(relative);
            fs::create_dir_all(destination.parent().expect("fixture parent"))
                .expect("fixture parent");
            fs::copy(curated().join(relative), destination).expect("copy frozen metadata");
        }
        Self { path }
    }

    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.path.join(relative);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture parent");
        fs::write(path, bytes).expect("write fixture file");
    }

    fn write_document(&self, document: &ErrataDocument) {
        let mut bytes = serde_json::to_vec(document).expect("serialize errata fixture");
        bytes.push(b'\n');
        self.write(ERRATA_REL, &bytes);
    }

    fn link_registry(&self) {
        let destination = self.path.join(re_corpus::provenance::REGISTRY_REL);
        fs::create_dir_all(destination.parent().expect("registry parent"))
            .expect("registry parent");
        fs::hard_link(
            curated().join(re_corpus::provenance::REGISTRY_REL),
            destination,
        )
        .expect("hard-link pinned registry without copying it");
    }

    fn copy_active_files(&self) {
        let active = load_active_receipt(&curated())
            .expect("actual active receipt")
            .expect("actual v6 active receipt");
        for file in &active.files {
            let source = curated().join(&file.imported.path);
            let destination = self.path.join(&file.imported.path);
            fs::create_dir_all(destination.parent().expect("imported parent"))
                .expect("imported parent");
            fs::copy(&source, &destination)
                .unwrap_or_else(|error| panic!("copy {}: {error}", file.imported.path));
        }
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn fixture_trust(fixture: &FixtureRoot) -> ProvenanceTrust {
    let bytes = fs::read(fixture.path.join(ERRATA_REL)).expect("read fixture errata");
    ProvenanceTrust::fixture(sha256_hex(&bytes))
}

fn status(fixture: &FixtureRoot) -> (u8, Value) {
    run_status_at_with_trust(&fixture.path, &fixture_trust(fixture))
}

fn lookup(fixture: &FixtureRoot, authority: &str, token: &str) -> (u8, Value) {
    run_lookup_at_with_trust(&fixture.path, authority, token, &fixture_trust(fixture))
}

fn verify_provenance(fixture: &FixtureRoot, research_root: Option<PathBuf>) -> (u8, Value) {
    run_verify_provenance_at_with_trust(&fixture.path, research_root, &fixture_trust(fixture))
}

fn finding_codes(output: &Value) -> Vec<String> {
    output["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .filter_map(|finding| finding["code"].as_str().map(str::to_owned))
        .collect()
}

#[test]
fn completion_candidate_preserves_every_ordinary_292_entry() {
    let original = ErrataDocument::parse(
        &fs::read(curated().join(
            "tools/re/local-recovery/bulk-provenance-fix01/phase-c-integration/CANDIDATE_292.json",
        ))
        .expect("read CANDIDATE_292"),
    )
    .expect("CANDIDATE_292 parses");
    let document = candidate();
    let mut by_key = BTreeMap::new();
    for entry in &original.entries {
        by_key.insert(
            (
                entry.target_catalog_record_id.clone(),
                entry.method_identity.type_def.clone(),
                entry.method_identity.method_def.clone(),
            ),
            entry,
        );
    }
    for entry in document
        .entries
        .iter()
        .filter(|entry| entry.profile == ProvenanceProfile::OrdinaryGameplayMap)
    {
        let key = (
            entry.target_catalog_record_id.clone(),
            entry.method_identity.type_def.clone(),
            entry.method_identity.method_def.clone(),
        );
        let prior = by_key
            .remove(&key)
            .expect("ordinary entry existed in CANDIDATE_292");
        assert_eq!(entry, prior, "ordinary entry changed: {key:?}");
    }
    assert!(by_key.is_empty());
}

#[test]
fn approved_full_candidate_passes_complete_533_coverage_in_a_fixture() {
    let fixture = FixtureRoot::v6("full-approved");
    fixture.copy_active_files();
    fixture.link_registry();
    fixture.write_document(&approved_533());
    let (exit, output) = verify_provenance(&fixture, Some(research()));
    assert_eq!(exit, 0, "{output}");
    assert_eq!(output["byte_verify"]["exit"], 0);
    assert_eq!(output["activation_verification"]["status"], "PASS");
    assert_eq!(output["counts"]["active_methods"], 533);
    assert_eq!(output["counts"]["covered"], 533);
    assert_eq!(output["counts"]["not_validated"], 0);
    assert_eq!(output["overall_acceptance"], "PASS");
    assert!(!finding_codes(&output).contains(&"PROVENANCE_COVERAGE_INCOMPLETE".to_owned()));
}

#[test]
fn pending_full_candidate_is_visible_but_not_applied() {
    let fixture = FixtureRoot::v6("full-pending");
    fixture.write(ERRATA_REL, &candidate_bytes());
    let (status_exit, status) = status(&fixture);
    assert_ne!(status_exit, 2);
    assert_eq!(status["provenance"]["status"], "PENDING_PRIMARY_REVIEW");
    assert_eq!(status["provenance"]["entries"], 533);
    assert_eq!(status["provenance"]["coverage"]["covered"], 0);
    assert_eq!(
        status["provenance"]["activation_block"],
        "ACTIVATION_REVIEW_NOT_APPROVED"
    );
    let (lookup_exit, lookup) = lookup(&fixture, "MANAGED-CS-E3FC67839851", "0x0600042E");
    assert_eq!(lookup_exit, 0);
    assert_eq!(
        lookup["matches"][0]["current"][0]["source_provenance"]["status"],
        "NOT_VALIDATED_BY_CURRENT_PROVENANCE_PROFILE"
    );
}

#[test]
fn effective_view_preserves_metadata_only_and_unknown_legacy_statuses() {
    let fixture = FixtureRoot::v6("full-applied");
    fixture.write_document(&approved_533());
    let (exit, task_base) = lookup(&fixture, "MANAGED-CS-E3FC67839851", "0x0601C913");
    assert_eq!(exit, 0, "{task_base}");
    let method = &task_base["matches"][0]["current"][0];
    assert_eq!(method["evidence_status"], "METADATA_ONLY");
    assert_eq!(method["source_status"], "METADATA_ONLY");
    assert_eq!(method["source_provenance"]["status"], "REFINED");
    assert_eq!(
        method["source_provenance"]["imported_source"]["precision"],
        "FILE_METHOD_ASSOCIATION"
    );

    let (exit, unknown) = lookup(&fixture, "MANAGED-CS-E3FC67839851", "0x06018868");
    assert_eq!(exit, 0, "{unknown}");
    let method = &unknown["matches"][0]["current"][0];
    assert_eq!(method["evidence_status"], "UNKNOWN");
    assert_eq!(method["source_status"], "UNKNOWN");
    assert_eq!(method["source_provenance"]["status"], "REFINED");
}

#[test]
fn registry_canonical_fallback_and_resealed_registry_are_rejected() {
    let mut fallback = candidate();
    let entry = fallback
        .entries
        .iter_mut()
        .find(|entry| entry.profile == ProvenanceProfile::RegistryFileAssociation)
        .expect("registry entry");
    let anchor = entry
        .corrected_evidence_source_anchors
        .iter_mut()
        .find(|anchor| {
            anchor.evidence_class == re_corpus::provenance::EvidenceClass::PerformanceSource
        })
        .expect("performance source anchor");
    anchor.path =
        "src/reconstructed/managed/full/CSharpScript/shards/alternate/source.cs".to_owned();
    let error = fallback
        .validate_against_roots(&curated(), &research())
        .expect_err("canonical fallback must not satisfy the selected source association");
    assert_eq!(error.code, "REGISTRY_SOURCE_ANCHOR_MISSING");

    let fixture = FixtureRoot::v6("resealed-registry");
    fixture.write(re_corpus::provenance::REGISTRY_REL, b"resealed\n");
    let error = candidate()
        .validate_against_roots(&fixture.path, &research())
        .expect_err("a resealed wrong registry must fail the pinned digest");
    assert_eq!(error.code, "REGISTRY_SIZE_MISMATCH");
}

#[test]
fn same_hash_wrong_authority_and_stale_target_fail_structurally() {
    let mut wrong_authority = candidate();
    let entry = wrong_authority
        .entries
        .iter_mut()
        .find(|entry| entry.target_catalog_record_id == "E18")
        .expect("E18 legacy entry");
    entry.method_identity.full_authority_sha256 = "f".repeat(64);
    let error = wrong_authority
        .validate_structural(&curated())
        .expect_err("same source hash with wrong authority must not pass");
    assert_eq!(error.code, "TARGET_METHOD_IDENTITY_MISMATCH");

    let mut stale = candidate();
    let entry = stale
        .entries
        .iter_mut()
        .find(|entry| entry.profile == ProvenanceProfile::LegacyV5SourceAssociation)
        .expect("legacy entry");
    entry.expected_original_record_hash = "0".repeat(64);
    let error = stale
        .validate_structural(&curated())
        .expect_err("stale target record hash must fail");
    assert_eq!(error.code, "STALE_TARGET_RECORD_HASH");
}

#[test]
fn associated_bad_method_and_report_wrong_selected_file_fail() {
    let mut associated = candidate();
    let entry = associated
        .entries
        .iter_mut()
        .find(|entry| {
            entry.profile == ProvenanceProfile::AssociatedGameplayMap
                && entry.method_identity.method_def == "0x06018666"
        })
        .expect("associated entry");
    entry
        .association_proof
        .as_mut()
        .expect("association proof")
        .literal_method_def = Some("0x06018667".to_owned());
    let error = associated
        .validate_against_roots(&curated(), &research())
        .expect_err("association row must name the entry method");
    assert_eq!(error.code, "ASSOCIATED_BINDING_SHAPE_MISMATCH");

    let mut report = candidate();
    let entry = report
        .entries
        .iter_mut()
        .find(|entry| {
            entry.profile == ProvenanceProfile::PerformanceEvidenceAssociation
                && entry.method_identity.method_def == "0x0604A4E5"
        })
        .expect("performance report entry");
    let report_ref = &mut entry
        .association_proof
        .as_mut()
        .expect("report proof")
        .report_refs[0];
    report_ref.path = "docs/performance/PERF05_UPDATE_TICK_ARCHITECTURE.md".to_owned();
    report_ref.artifact_sha256 =
        Some("df50418d6cc9c21a05aae29b3e6cab7a550148bd13cf23093d728d6cf6770f65".to_owned());
    let error = report
        .validate_against_roots(&curated(), &research())
        .expect_err("accepted report must name the selected source file");
    assert_eq!(error.code, "PERFORMANCE_BINDING_REPORT_MISMATCH");
}

#[test]
fn completion_binding_tables_cover_exact_associated_and_performance_keys() {
    let document = candidate();
    for binding in ASSOCIATED_BINDINGS {
        let entry = document
            .entries
            .iter()
            .find(|entry| {
                entry.profile == ProvenanceProfile::AssociatedGameplayMap
                    && entry.target_catalog_record_id == binding.record_id
                    && entry
                        .method_identity
                        .type_def
                        .eq_ignore_ascii_case(binding.type_def)
                    && entry
                        .method_identity
                        .method_def
                        .eq_ignore_ascii_case(binding.method_def)
            })
            .expect("associated binding has a candidate entry");
        let proof = entry.association_proof.as_ref().expect("association proof");
        assert_eq!(proof.map_line, Some(binding.map_line));
        assert_eq!(
            proof.literal_method_def.as_deref(),
            Some(binding.method_def)
        );
        assert!(
            entry
                .corrected_evidence_source_anchors
                .iter()
                .any(|anchor| {
                    anchor.evidence_class == EvidenceClass::SemanticSource
                        && anchor.path == binding.source_path
                })
        );
        assert_eq!(
            entry.corrected_imported_source_anchor.start_line,
            binding.exact.map(|range| range.0)
        );
        assert_eq!(
            entry.corrected_imported_source_anchor.end_line,
            binding.exact.map(|range| range.1)
        );
    }
    for binding in PERFORMANCE_BINDINGS {
        let entry = document
            .entries
            .iter()
            .find(|entry| {
                entry.target_catalog_record_id == binding.record_id
                    && entry
                        .method_identity
                        .type_def
                        .eq_ignore_ascii_case(binding.type_def)
                    && entry
                        .method_identity
                        .method_def
                        .eq_ignore_ascii_case(binding.method_def)
            })
            .expect("performance binding has a candidate entry");
        let proof = entry.association_proof.as_ref().expect("report proof");
        assert_eq!(proof.report_refs.len(), 1);
        assert_eq!(proof.report_refs[0].path, binding.report_path);
        assert_eq!(
            proof.report_refs[0].artifact_sha256.as_deref(),
            Some(binding.report_sha256)
        );
    }
}

#[test]
fn neighboring_range_and_unrelated_section_keyword_are_rejected() {
    let mut neighbor = candidate();
    let entry = neighbor
        .entries
        .iter_mut()
        .find(|entry| {
            entry.profile == ProvenanceProfile::AssociatedGameplayMap
                && entry.method_identity.method_def == "0x06018666"
        })
        .expect("associated entry");
    entry.association_proof.as_mut().expect("proof").map_line = Some(4546);
    entry.corrected_imported_source_anchor.start_line = Some(410);
    entry.corrected_imported_source_anchor.end_line = Some(421);
    let error = neighbor
        .validate_against_roots(&curated(), &research())
        .expect_err("neighboring row/range must not satisfy the target binding");
    assert_eq!(error.code, "ASSOCIATED_BINDING_SHAPE_MISMATCH");

    let mut wrong_source = candidate();
    let entry = wrong_source
        .entries
        .iter_mut()
        .find(|entry| {
            entry.profile == ProvenanceProfile::AssociatedGameplayMap
                && entry.method_identity.method_def == "0x06018666"
        })
        .expect("associated entry");
    let anchor = entry
        .corrected_evidence_source_anchors
        .iter_mut()
        .find(|anchor| anchor.evidence_class == EvidenceClass::SemanticSource)
        .expect("semantic anchor");
    anchor.path = "src/reconstructed/managed/semantic/gameplay/Skill.cs".to_owned();
    let error = wrong_source
        .validate_against_roots(&curated(), &research())
        .expect_err("section wording cannot replace the target source file");
    assert_eq!(error.code, "COMPLETION_EVIDENCE_ANCHOR_UNSUPPORTED");
}

#[test]
fn performance_report_must_be_the_bound_report_and_matching_identity() {
    let mut report = candidate();
    let entry = report
        .entries
        .iter_mut()
        .find(|entry| {
            entry.profile == ProvenanceProfile::PerformanceEvidenceAssociation
                && entry.method_identity.method_def == "0x0604A4E5"
        })
        .expect("performance entry");
    let report_ref = &mut entry
        .association_proof
        .as_mut()
        .expect("report proof")
        .report_refs[0];
    report_ref.path = "docs/performance/PERF05_UPDATE_TICK_ARCHITECTURE.md".to_owned();
    report_ref.artifact_sha256 =
        Some("df50418d6cc9c21a05aae29b3e6cab7a550148bd13cf23093d728d6cf6770f65".to_owned());
    let error = report
        .validate_against_roots(&curated(), &research())
        .expect_err("an unrelated accepted report must not satisfy the binding");
    assert_eq!(error.code, "PERFORMANCE_BINDING_REPORT_MISMATCH");

    let mut identity = candidate();
    let entry = identity
        .entries
        .iter_mut()
        .find(|entry| entry.method_identity.method_def == "0x06046713")
        .expect("core entry");
    entry.method_identity.type_def = "0x02007188".to_owned();
    let error = identity
        .validate_structural(&curated())
        .expect_err("wrong actual TypeDef with the same MethodDef must fail");
    assert_eq!(error.code, "TARGET_METHOD_MISSING");
}

#[test]
fn extra_evidence_anchors_must_be_target_bound() {
    let mut out_of_bounds = candidate();
    let entry = out_of_bounds
        .entries
        .iter_mut()
        .find(|entry| entry.profile == ProvenanceProfile::RegistryFileAssociation)
        .expect("registry entry");
    let mut extra = entry
        .corrected_evidence_source_anchors
        .iter()
        .find(|anchor| anchor.evidence_class == EvidenceClass::PerformanceSource)
        .expect("source anchor")
        .clone();
    extra.precision = Precision::ExactRange;
    extra.start_line = Some(999_999);
    extra.end_line = Some(1_000_000);
    entry.corrected_evidence_source_anchors.push(extra);
    let error = out_of_bounds
        .validate_against_roots(&curated(), &research())
        .expect_err("out-of-bounds extra anchor must fail");
    assert_eq!(error.code, "COMPLETION_SOURCE_ANCHOR_MISMATCH");

    let mut in_bounds_wrong = candidate();
    let entry = in_bounds_wrong
        .entries
        .iter_mut()
        .find(|entry| {
            entry.profile == ProvenanceProfile::AssociatedGameplayMap
                && entry.method_identity.method_def == "0x06018666"
        })
        .expect("associated entry");
    let mut extra = entry
        .corrected_evidence_source_anchors
        .iter()
        .find(|anchor| anchor.evidence_class == EvidenceClass::SemanticSource)
        .expect("semantic anchor")
        .clone();
    extra.precision = Precision::ExactRange;
    extra.start_line = Some(1);
    extra.end_line = Some(2);
    entry.corrected_evidence_source_anchors.push(extra);
    let error = in_bounds_wrong
        .validate_against_roots(&curated(), &research())
        .expect_err("in-bounds wrong extra anchor must fail");
    assert_eq!(error.code, "COMPLETION_SOURCE_ANCHOR_MISMATCH");
}

#[test]
fn support_reference_hash_and_unknown_path_are_rejected() {
    let mut hash = candidate();
    let entry = hash
        .entries
        .iter_mut()
        .find(|entry| entry.profile == ProvenanceProfile::RegistryFileAssociation)
        .expect("registry entry");
    entry.corrected_imported_source_anchor.support_refs[0].artifact_sha256 = Some("0".repeat(64));
    let error = hash
        .validate_against_roots(&curated(), &research())
        .expect_err("wrong support reference hash must fail");
    assert_eq!(error.code, "COMPLETION_SUPPORT_REFERENCE_SHA_MISMATCH");

    let mut unknown = candidate();
    let entry = unknown
        .entries
        .iter_mut()
        .find(|entry| entry.profile == ProvenanceProfile::LegacyV5SourceAssociation)
        .expect("legacy entry");
    entry
        .corrected_imported_source_anchor
        .support_refs
        .push(SupportRef {
            path: "evidence/not-accepted.json".to_owned(),
            locator: "unverified".to_owned(),
            artifact_sha256: Some("1".repeat(64)),
        });
    let error = unknown
        .validate_against_roots(&curated(), &research())
        .expect_err("unknown support path must fail");
    assert_eq!(error.code, "COMPLETION_SUPPORT_REFERENCE_UNVALIDATED");
}

#[test]
fn effective_view_quarantines_unsupported_precise_refs_and_keeps_exact_ones() {
    let fixture = FixtureRoot::v6("quarantine-applied");
    fixture.write_document(&approved_533());
    for (method_def, locator) in [
        ("0x06030B43", "source lines 37-152"),
        ("0x06049EB8", "source lines 207-264"),
    ] {
        let (exit, lookup) = lookup(&fixture, "MANAGED-CS-E3FC67839851", method_def);
        assert_eq!(exit, 0, "{lookup}");
        let method = &lookup["matches"][0]["current"][0];
        assert_eq!(method["source_provenance"]["status"], "REFINED");
        assert!(
            !method["evidence_references"]
                .as_array()
                .expect("active refs")
                .iter()
                .any(|reference| reference["locator"]
                    .as_str()
                    .is_some_and(|value| value.contains(locator)))
        );
        assert!(
            method["source_provenance"]["legacy_original_references"]
                .as_array()
                .expect("legacy refs")
                .iter()
                .any(|reference| reference["locator"]
                    .as_str()
                    .is_some_and(|value| value.contains(locator)))
        );
    }

    let (exit, lookup) = lookup(&fixture, "MANAGED-CS-E3FC67839851", "0x06018666");
    assert_eq!(exit, 0, "{lookup}");
    let method = &lookup["matches"][0]["current"][0];
    assert_eq!(method["source_provenance"]["status"], "CONFIRMED");
    assert!(
        method["evidence_references"]
            .as_array()
            .expect("active refs")
            .iter()
            .any(|reference| reference["locator"] == "source lines 373-387")
    );
    assert!(
        method["source_provenance"]["legacy_original_references"]
            .as_array()
            .expect("legacy refs")
            .is_empty()
    );
}

#[test]
fn completion_binding_manifest_is_complete() {
    let manifest = re_corpus::provenance::completion_binding_manifest();
    assert_eq!(
        manifest["associated"].as_array().expect("associated").len(),
        18
    );
    assert_eq!(
        manifest["performance"]
            .as_array()
            .expect("performance")
            .len(),
        13
    );
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("completion_bindings.json");
    fs::write(
        out,
        serde_json::to_vec_pretty(&manifest).expect("serialize binding manifest"),
    )
    .expect("write binding manifest to target");
}
