use re_corpus::bulk::CATALOG_REL;
use re_corpus::provenance::{
    ERRATA_REL, ErrataAction, ErrataDocument, Outcome, Precision, ProvenanceTrust, ReviewStatus,
    canonical_record_hash,
};
use re_corpus::{
    CliCommand, RECEIPT_REL, RECEIPT_V2_REL, RECEIPT_V3_REL, RECEIPT_V4_REL, RECEIPT_V5_REL,
    load_active_receipt, parse_cli, run_lookup_at_with_trust, run_status_at_with_trust,
    run_verify_at, run_verify_provenance_at_with_trust, sha256_hex,
};
use serde_json::Value;
use std::ffi::OsString;
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
        "tools/re/local-recovery/bulk-provenance-fix01/phase-c-integration/CANDIDATE_292.json",
    ))
    .expect("read CANDIDATE_292")
}

#[test]
fn private_candidate_292_is_field_derived_and_validates_against_actual_roots() {
    let bytes = candidate_bytes();
    let document = ErrataDocument::parse(&bytes).expect("CANDIDATE_292 shape");
    assert_eq!(document.entries.len(), 292);
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
        75
    );
    assert_eq!(
        document
            .entries
            .iter()
            .filter(|entry| entry.action == ErrataAction::Refine)
            .count(),
        33
    );

    let (curated_root, research_root) = (curated(), research());
    document
        .validate_against_roots(&curated_root, &research_root)
        .expect("CANDIDATE_292 must validate against actual roots");

    let pending = document
        .validate_for_activation(&curated_root, &research_root)
        .expect_err("pending reviews must not be activatable");
    assert_eq!(pending.code, "ACTIVATION_REVIEW_NOT_APPROVED");

    let mut approved = document.clone();
    for entry in &mut approved.entries {
        entry.review_status = ReviewStatus::PrimaryApproved;
    }
    approved
        .validate_for_activation(&curated_root, &research_root)
        .expect("approved CANDIDATE_292 must be activatable");

    let catalog_bytes =
        fs::read(curated_root.join("metadata/curation/bulk-curate01.json")).expect("catalog bytes");
    let catalog: Value = serde_json::from_slice(&catalog_bytes).expect("catalog json");
    let records = catalog["records"].as_array().expect("records");
    let mut refine_exact = 0;
    let mut refine_single = 0;
    for entry in &document.entries {
        let record = records
            .iter()
            .find(|record| record["id"] == entry.target_catalog_record_id)
            .expect("target record");
        assert_eq!(record["cohort"], "gameplay66");
        let method = record["methods"]
            .as_array()
            .expect("methods")
            .iter()
            .find(|method| {
                method["type_def"] == entry.method_identity.type_def
                    && method["method_def"] == entry.method_identity.method_def
            })
            .expect("target method");
        assert_eq!(method["role"], "ORDINARY");
        assert_eq!(
            canonical_record_hash(record).expect("record hash"),
            entry.expected_original_record_hash
        );
        if entry.action == ErrataAction::Refine {
            assert!(method["source_lines"].is_null());
            assert_eq!(
                entry.correction_class,
                re_corpus::provenance::CorrectionClass::NotACorrection
            );
            if entry.corrected_imported_source_anchor.precision == Precision::ExactRange {
                refine_exact += 1;
            } else {
                refine_single += 1;
            }
        }
    }
    assert_eq!(refine_exact, 31);
    assert_eq!(refine_single, 2);

    for entry in &document.entries {
        if entry.action == ErrataAction::Correct {
            assert!(!entry.incorrect_anchors.is_empty());
            assert!(
                entry.outcome == Outcome::ResolvedExact
                    || entry.outcome == Outcome::ResolvedConservative
            );
        }
    }
}

struct FixtureRoot {
    path: PathBuf,
}

impl FixtureRoot {
    fn v5(label: &str) -> Self {
        let n = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("provenance-integration-fixtures")
            .join(format!("{label}-{}-{n}", std::process::id()));
        fs::create_dir_all(&path).expect("create owned fixture");
        for relative in [
            re_corpus::MANIFEST_REL,
            RECEIPT_REL,
            RECEIPT_V2_REL,
            RECEIPT_V3_REL,
            RECEIPT_V4_REL,
            RECEIPT_V5_REL,
            "metadata/performance-source.v1.json",
            "metadata/scheduling-source.v1.json",
            "metadata/task-execution-source.v1.json",
            CATALOG_REL,
        ] {
            let destination = path.join(relative);
            fs::create_dir_all(destination.parent().expect("metadata parent"))
                .expect("metadata directory");
            fs::copy(curated().join(relative), destination).expect("copy frozen metadata");
        }
        Self { path }
    }

    fn v6(label: &str) -> Self {
        let fixture = Self::v5(label);
        fixture.write(
            "metadata/curation-batches.jsonl",
            &fs::read(curated().join("metadata/curation-batches.jsonl")).expect("read index"),
        );
        fixture
    }

    fn copy_active_files(&self) {
        let active = load_active_receipt(&curated())
            .expect("actual active receipt")
            .expect("actual v6 active receipt");
        for file in &active.files {
            let source = curated().join(&file.imported.path);
            let destination = self.path.join(&file.imported.path);
            fs::create_dir_all(destination.parent().expect("imported parent"))
                .expect("imported parent directory");
            fs::copy(&source, &destination)
                .unwrap_or_else(|error| panic!("copy {}: {error}", file.imported.path));
        }
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
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn fixture_trust(fixture: &FixtureRoot) -> ProvenanceTrust {
    let path = fixture.path.join(ERRATA_REL);
    if !path.is_file() {
        return ProvenanceTrust::production();
    }
    let bytes = fs::read(path).expect("read fixture errata");
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

fn candidate_document() -> ErrataDocument {
    ErrataDocument::parse(&candidate_bytes()).expect("CANDIDATE_292 parses")
}

fn approved_single(action: ErrataAction) -> ErrataDocument {
    let mut document = candidate_document();
    let entry = document
        .entries
        .iter()
        .find(|entry| entry.action == action)
        .expect("entry with requested action")
        .clone();
    document.entries = vec![entry];
    for entry in &mut document.entries {
        entry.review_status = ReviewStatus::PrimaryApproved;
    }
    document
}

fn approved_292() -> ErrataDocument {
    let mut document = candidate_document();
    for entry in &mut document.entries {
        entry.review_status = ReviewStatus::PrimaryApproved;
    }
    document
}

fn catalog_value() -> Value {
    serde_json::from_slice(
        &fs::read(curated().join("metadata/curation/bulk-curate01.json")).expect("catalog"),
    )
    .expect("catalog json")
}

fn parse_args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
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
fn existing_v5_fixture_reports_not_applicable_and_does_not_claim_pass() {
    let fixture = FixtureRoot::v5("v5");
    let (status_exit, status) = status(&fixture);
    assert_ne!(status_exit, 2, "v5 fixture must remain loadable");
    assert_eq!(
        status["provenance"]["status"],
        "NOT_APPLICABLE_BULK_CORRECTION"
    );
    assert_eq!(status["provenance"]["verification"], "NOT_APPLICABLE");
    assert_eq!(status["counts"]["imported_file_count"], 13);
    assert_eq!(
        status["provenance"]["coverage"]["global_required_keys"],
        102
    );

    let (_exit, unrooted) = verify_provenance(&fixture, None);
    assert_eq!(unrooted["overall_acceptance"], "BLOCKED");
    assert!(
        finding_codes(&unrooted).contains(&"PROVENANCE_NOT_APPLICABLE_BULK_CORRECTION".to_owned())
    );
}

#[test]
fn owned_v6_fixture_without_errata_is_absent_and_verify_provenance_fails() {
    let fixture = FixtureRoot::v6("absent-v6");
    fixture.copy_active_files();
    let (status_exit, status) = status(&fixture);
    assert_eq!(status_exit, 0);
    assert_eq!(status["provenance"]["status"], "ABSENT");
    assert_eq!(status["provenance"]["verification"], "BLOCKED");
    assert_eq!(
        status["provenance"]["coverage"]["global_required_keys"],
        533
    );
    assert_eq!(status["provenance"]["coverage"]["covered"], 0);
    assert_eq!(status["provenance"]["coverage"]["not_validated"], 533);

    let (verify_exit, verify) = run_verify_at(&fixture.path, Some(research()));
    assert_eq!(verify_exit, 0);
    assert_eq!(verify["verify_scope"], "BYTE_INTEGRITY_ONLY");
    assert_eq!(verify["provenance"]["status"], "ABSENT");
    assert_eq!(verify["provenance"]["verification"], "BLOCKED");

    let (exit, output) = verify_provenance(&fixture, Some(research()));
    assert_eq!(exit, 1);
    assert_eq!(output["overall_acceptance"], "BLOCKED");
    assert_eq!(output["activation_verification"]["status"], "NOT_RUN");
    assert_eq!(output["counts"]["active_methods"], 533);
    assert_eq!(output["counts"]["covered"], 0);
    assert_eq!(output["counts"]["not_validated"], 533);
    let codes = finding_codes(&output);
    assert!(codes.contains(&"PROVENANCE_ERRATA_ABSENT".to_owned()));
    assert!(codes.contains(&"PROVENANCE_COVERAGE_INCOMPLETE".to_owned()));
    assert_eq!(
        output["provenance"]["coverage"]["not_validated"]["ordinary_gameplay"],
        292
    );
    assert_eq!(
        output["provenance"]["coverage"]["not_validated"]["gameplay_associated"],
        18
    );
    assert_eq!(
        output["provenance"]["coverage"]["not_validated"]["performance"],
        121
    );
    assert_eq!(
        output["provenance"]["coverage"]["not_validated"]["legacy_v5"],
        102
    );
}

#[test]
fn approved_single_correction_applies_typed_view_and_removes_unmarked_numeric_ref() {
    let fixture = FixtureRoot::v6("approved-single");
    let document = approved_single(ErrataAction::Correct);
    let anchor = document.entries[0].incorrect_anchors[0].clone();
    let method_def = document.entries[0].method_identity.method_def.clone();
    fixture.write_document(&document);

    let (status_exit, status) = status(&fixture);
    assert_ne!(status_exit, 2);
    assert_eq!(status["provenance"]["status"], "PARTIAL");
    assert_eq!(status["provenance"]["verification"], "NOT_CHECKED");
    assert_eq!(status["provenance"]["coverage"]["covered"], 1);
    assert_eq!(status["provenance"]["coverage"]["not_validated"], 532);

    let (exit, lookup) = lookup(&fixture, "MANAGED-CS-E3FC67839851", &method_def);
    assert_eq!(exit, 0, "lookup: {lookup}");
    let method = &lookup["matches"][0]["current"][0];
    let provenance = &method["source_provenance"];
    assert_eq!(provenance["status"], "CORRECTED");
    assert_eq!(provenance["action"], "CORRECT");
    assert_eq!(provenance["verification"], "PROVENANCE_NOT_VERIFIED");
    assert_eq!(provenance["imported_source"]["root"], "CURATED_ROOT");
    assert!(
        provenance["supporting_anchors"]
            .as_array()
            .expect("supporting anchors")
            .iter()
            .all(|anchor| anchor["root"] == "RESEARCH_ROOT")
    );
    let legacy = provenance["legacy_original_references"]
        .as_array()
        .expect("legacy refs");
    assert_eq!(legacy.len(), 1);
    assert_eq!(legacy[0]["status"], "LEGACY_ORIGINAL_UNCORRECTED");
    assert_eq!(legacy[0]["path"], anchor.path);
    assert_eq!(legacy[0]["locator"], anchor.locator.as_str());
    let active = method["evidence_references"].as_array().expect("refs");
    assert!(
        active
            .iter()
            .all(|reference| reference["locator"] != anchor.locator),
        "the false numeric reference must not remain an unmarked active exact reference"
    );

    let (_exit, unrooted) = verify_provenance(&fixture, None);
    assert_eq!(unrooted["overall_acceptance"], "BLOCKED");
    assert!(finding_codes(&unrooted).contains(&"PROVENANCE_RESEARCH_ROOT_REQUIRED".to_owned()));
    assert!(finding_codes(&unrooted).contains(&"PROVENANCE_COVERAGE_INCOMPLETE".to_owned()));
}

#[test]
fn approved_ordinary_profile_covers_292_and_leaves_241_explicitly_unvalidated() {
    let fixture = FixtureRoot::v6("approved-292");
    fixture.write_document(&approved_292());
    let (status_exit, status) = status(&fixture);
    assert_ne!(status_exit, 2);
    assert_eq!(status["provenance"]["status"], "PARTIAL");
    assert_eq!(status["provenance"]["coverage"]["covered"], 292);
    assert_eq!(status["provenance"]["coverage"]["not_validated"], 241);
    assert_eq!(status["provenance"]["actions"]["CORRECT"], 184);
    assert_eq!(status["provenance"]["actions"]["CONFIRM"], 75);
    assert_eq!(status["provenance"]["actions"]["REFINE"], 33);
    assert_eq!(
        status["provenance"]["coverage"]["global_required_keys"],
        533
    );
    assert_eq!(status["provenance"]["overall_acceptance"], "BLOCKED");

    let catalog = catalog_value();
    let records = catalog["records"].as_array().expect("records");
    let associated = records
        .iter()
        .find(|record| {
            record["cohort"] == "gameplay66"
                && record["methods"]
                    .as_array()
                    .expect("methods")
                    .iter()
                    .any(|method| method["role"] == "ASSOCIATED")
        })
        .expect("associated gameplay record");
    let associated_method = associated["methods"]
        .as_array()
        .expect("methods")
        .iter()
        .find(|method| method["role"] == "ASSOCIATED")
        .expect("associated method");
    let associated_def = associated_method["method_def"]
        .as_str()
        .expect("method def");
    let (exit, lookup) = lookup(&fixture, "MANAGED-CS-E3FC67839851", associated_def);
    assert_eq!(exit, 0, "associated lookup: {lookup}");
    let method = &lookup["matches"][0]["current"][0];
    assert_eq!(
        method["source_provenance"]["status"],
        "NOT_VALIDATED_BY_CURRENT_PROVENANCE_PROFILE"
    );

    let (_exit, unrooted) = verify_provenance(&fixture, None);
    assert_eq!(unrooted["overall_acceptance"], "BLOCKED");
    assert_eq!(unrooted["counts"]["covered"], 292);
    assert_eq!(unrooted["counts"]["not_validated"], 241);
    let not_validated = &unrooted["provenance"]["coverage"]["not_validated"];
    assert_eq!(not_validated["ordinary_gameplay"], 0);
    assert_eq!(not_validated["gameplay_associated"], 18);
    assert_eq!(not_validated["performance"], 121);
    assert_eq!(not_validated["legacy_v5"], 102);
}

#[test]
fn approved_partial_activation_passes_but_overall_acceptance_stays_blocked() {
    let fixture = FixtureRoot::v6("approved-full");
    let active = load_active_receipt(&curated())
        .expect("actual active receipt")
        .expect("actual v6 active receipt");
    assert_eq!(active.version, 6);
    for file in &active.files {
        let source = curated().join(&file.imported.path);
        let destination = fixture.path.join(&file.imported.path);
        fs::create_dir_all(destination.parent().expect("imported parent"))
            .expect("imported parent directory");
        fs::copy(&source, &destination)
            .unwrap_or_else(|error| panic!("copy {}: {error}", file.imported.path));
    }
    fixture.write_document(&approved_292());

    let (exit, output) = verify_provenance(&fixture, Some(research()));
    assert_eq!(
        exit, 1,
        "241 uncovered keys must keep overall acceptance nonzero"
    );
    assert_eq!(output["byte_verify"]["exit"], 0);
    assert_eq!(output["activation_verification"]["status"], "PASS");
    assert_eq!(output["counts"]["covered"], 292);
    assert_eq!(output["counts"]["not_validated"], 241);
    assert_eq!(output["overall_acceptance"], "BLOCKED");
    assert!(finding_codes(&output).contains(&"PROVENANCE_COVERAGE_INCOMPLETE".to_owned()));
}

#[test]
fn public_errata_without_compiled_pin_fails_closed() {
    let fixture = FixtureRoot::v6("unpinned-public");
    fixture.write_document(&approved_single(ErrataAction::Confirm));
    let (exit, output) = run_status_at_with_trust(
        &fixture.path,
        &ProvenanceTrust {
            public_errata_sha256: None,
        },
    );
    assert_eq!(exit, 2);
    assert_eq!(
        output["findings"][0]["code"],
        "PUBLIC_ERRATA_DIGEST_NOT_PINNED"
    );
}

#[test]
fn pending_private_candidate_fails_closed_without_invalid_fallback() {
    let fixture = FixtureRoot::v6("pending");
    fixture.write(ERRATA_REL, &candidate_bytes());
    let (status_exit, status) = status(&fixture);
    assert_ne!(
        status_exit, 2,
        "pending review is a blocked state, not a schema failure"
    );
    assert_eq!(status["provenance"]["status"], "PENDING_PRIMARY_REVIEW");
    assert_eq!(status["provenance"]["verification"], "BLOCKED");
    assert_eq!(status["provenance"]["coverage"]["covered"], 0);
    assert_eq!(
        status["provenance"]["activation_block"],
        "ACTIVATION_REVIEW_NOT_APPROVED"
    );
    assert_eq!(status["provenance"]["overall_acceptance"], "BLOCKED");

    let (lookup_exit, lookup) = lookup(&fixture, "MANAGED-CS-E3FC67839851", "0x060186E0");
    assert_eq!(
        lookup_exit, 0,
        "no invalid fallback after a pending document"
    );
    assert_eq!(
        lookup["matches"][0]["current"][0]["source_provenance"]["status"],
        "NOT_VALIDATED_BY_CURRENT_PROVENANCE_PROFILE"
    );

    let (exit, output) = verify_provenance(&fixture, None);
    assert_eq!(exit, 1);
    assert_eq!(output["activation_verification"]["status"], "FAIL");
    assert_eq!(
        output["activation_verification"]["code"],
        "ACTIVATION_REVIEW_NOT_APPROVED"
    );
    assert_eq!(output["overall_acceptance"], "BLOCKED");
    assert_eq!(output["counts"]["covered"], 0);
    assert_eq!(output["counts"]["not_validated"], 533);
    assert!(finding_codes(&output).contains(&"ACTIVATION_REVIEW_NOT_APPROVED".to_owned()));
}

#[test]
fn stale_target_wrong_identity_and_competing_corrections_fail_closed() {
    let fixture = FixtureRoot::v6("mutations");
    let mut stale = approved_single(ErrataAction::Correct);
    stale.entries[0].expected_original_record_hash = "0".repeat(64);
    fixture.write_document(&stale);
    let (exit, output) = status(&fixture);
    assert_eq!(exit, 2);
    assert_eq!(output["findings"][0]["code"], "STALE_TARGET_RECORD_HASH");

    let mut wrong_identity = approved_single(ErrataAction::Correct);
    wrong_identity.entries[0].method_identity.type_def = "0x0200FFFF".to_owned();
    fixture.write_document(&wrong_identity);
    let (exit, output) = status(&fixture);
    assert_eq!(exit, 2);
    assert_eq!(output["findings"][0]["code"], "TARGET_METHOD_MISSING");

    let mut competing = approved_single(ErrataAction::Correct);
    let duplicate = competing.entries[0].clone();
    competing.entries.push(duplicate);
    let mut bytes = serde_json::to_vec(&competing).expect("serialize competing");
    bytes.push(b'\n');
    fixture.write(ERRATA_REL, &bytes);
    let (exit, output) = status(&fixture);
    assert_eq!(exit, 2);
    assert_eq!(output["findings"][0]["code"], "COMPETING_ERRATA_TARGET");

    let mut malformed = approved_single(ErrataAction::Correct);
    malformed.entries[0].corrected_imported_source_anchor.path =
        "src/managed/cs-e3fc67839851/not-the-destination.cs".to_owned();
    fixture.write_document(&malformed);
    let (exit, output) = status(&fixture);
    assert_eq!(exit, 2);
    assert_eq!(
        output["findings"][0]["code"],
        "STRUCTURAL_SOURCE_BINDING_MISMATCH"
    );
}

#[test]
fn competing_errata_versions_and_syntactically_invalid_document_fail_closed() {
    let fixture = FixtureRoot::v6("competing-versions");
    let document = approved_single(ErrataAction::Confirm);
    fixture.write_document(&document);
    let mut second = serde_json::to_vec(&document).expect("serialize second version");
    second.push(b'\n');
    fixture.write(
        "metadata/curation/bulk-curate01.provenance-errata.v2.json",
        &second,
    );
    let (exit, output) = status(&fixture);
    assert_eq!(exit, 2);
    assert_eq!(output["findings"][0]["code"], "COMPETING_ERRATA_VERSIONS");

    let fixture = FixtureRoot::v6("malformed");
    fixture.write(ERRATA_REL, b"{not valid json\n");
    let (exit, output) = status(&fixture);
    assert_eq!(exit, 2);
    assert_eq!(output["findings"][0]["code"], "MALFORMED_JSON");
}

#[test]
fn verify_provenance_cli_parses_and_unknown_arguments_stay_unknown() {
    let parsed = parse_cli(parse_args(&[
        "verify-provenance",
        "--research-root",
        "C:\\research",
        "--json",
    ]))
    .expect("verify-provenance parses");
    assert_eq!(parsed.command_name, "verify-provenance");
    assert!(matches!(
        parsed.command,
        CliCommand::VerifyProvenance {
            research_root: Some(_)
        }
    ));

    let error = parse_cli(parse_args(&["verify-provenance", "--bogus"]))
        .expect_err("unknown option stays rejected");
    assert_eq!(error.code, "UNKNOWN_OPTION");
    let error = parse_cli(parse_args(&["status", "--research-root", "C:\\research"]))
        .expect_err("status still rejects research root");
    assert_eq!(error.code, "UNKNOWN_OPTION");
    let error = parse_cli(parse_args(&["verify-provenance", "extra"]))
        .expect_err("positional argument stays rejected");
    assert_eq!(error.code, "INVALID_ARGUMENTS");
    let error =
        parse_cli(parse_args(&["unknown-command"])).expect_err("unknown command stays rejected");
    assert_eq!(error.code, "UNKNOWN_COMMAND");
}
