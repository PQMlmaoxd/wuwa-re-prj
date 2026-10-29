use re_corpus::bulk::{
    BASELINE_V5_RECEIPT_SHA256, CAMPAIGN_ID, CATALOG_REL, CATALOG_SCHEMA, CATALOG_SHA256,
    FROZEN_REGISTRY_RECEIPT_SHA256, FROZEN_SNAPSHOT_ID, GAMEPLAY_COHORT, INDEX_REL, INDEX_SCHEMA,
    PERFORMANCE_COHORT, append_activation_bytes, apply_activated_catalog, apply_approved_cohort,
    apply_index, canonical_json_line, load_campaign_catalog, parse_index_bytes, write_copy_receipt,
};
use re_corpus::{
    EvidenceStatus, MANIFEST_REL, Manifest, SourceStatus, curated_root, load_active_receipt,
    parse_manifest_bytes, run_lookup_at, run_status_at, run_verify_at, sha256_hex,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

struct FixtureRoot {
    path: PathBuf,
}

impl FixtureRoot {
    fn new() -> Self {
        let n = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("bulk-test-fixtures")
            .join(format!("{}-{n}", std::process::id()));
        fs::create_dir_all(&path).expect("create owned fixture");
        Self { path }
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn actual_manifest() -> Manifest {
    let path = curated_root().join(MANIFEST_REL);
    parse_manifest_bytes(&fs::read(path).expect("read actual manifest"))
        .expect("parse actual manifest")
}

fn v5_fixture() -> FixtureRoot {
    let fixture = FixtureRoot::new();
    fs::create_dir_all(fixture.path.join("metadata/curation")).expect("metadata dirs");
    let curated = curated_root();
    for relative in [
        MANIFEST_REL,
        "metadata/imports.v1.json",
        "metadata/imports.v2.json",
        "metadata/imports.v3.json",
        "metadata/imports.v4.json",
        "metadata/imports.v5.json",
        "metadata/performance-source.v1.json",
        "metadata/scheduling-source.v1.json",
        "metadata/task-execution-source.v1.json",
        CATALOG_REL,
    ] {
        fs::copy(curated.join(relative), fixture.path.join(relative)).expect("copy base metadata");
    }
    fixture
}

fn copy_base_imports(fixture: &FixtureRoot) {
    let active = load_active_receipt(&fixture.path)
        .expect("fixture v5 active")
        .expect("v5 active exists");
    for file in &active.files {
        let destination = fixture.path.join(&file.imported.path);
        fs::create_dir_all(destination.parent().expect("import parent")).expect("import dirs");
        fs::copy(curated_root().join(&file.imported.path), &destination)
            .expect("copy baseline imported file");
    }
}

#[cfg(windows)]
struct JunctionGuard {
    link: PathBuf,
    active: bool,
}

#[cfg(windows)]
impl JunctionGuard {
    fn new(link: &std::path::Path, target: &std::path::Path) -> Self {
        let link_text = link.to_string_lossy().replace('/', "\\");
        let target_text = target.to_string_lossy().replace('/', "\\");
        let output = std::process::Command::new("cmd.exe")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(&link_text)
            .arg(&target_text)
            .output()
            .expect("run native cmd.exe mklink /J");
        assert!(
            output.status.success(),
            "junction creation failed: status={:?} stdout={} stderr={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Self {
            link: link.to_path_buf(),
            active: true,
        }
    }

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

#[cfg(windows)]
impl Drop for JunctionGuard {
    fn drop(&mut self) {
        if !self.unlink() {
            // Never recurse through a still-live junction from fixture cleanup.
            std::mem::forget(std::mem::replace(
                &mut self.link,
                PathBuf::from("junction-unlink-failed"),
            ));
        }
    }
}

fn canonical_line(value: &Value) -> Vec<u8> {
    canonical_json_line(value).expect("canonical line")
}

fn header_value() -> Value {
    json!({
        "schema": INDEX_SCHEMA,
        "version": 1,
        "campaign": CAMPAIGN_ID,
        "baseline_v5_sha256": BASELINE_V5_RECEIPT_SHA256,
        "frozen_snapshot_id": FROZEN_SNAPSHOT_ID,
        "frozen_registry_receipt_sha256": FROZEN_REGISTRY_RECEIPT_SHA256,
        "catalog_path": CATALOG_REL,
        "catalog_sha256": CATALOG_SHA256,
        "approved_cohorts": [GAMEPLAY_COHORT, PERFORMANCE_COHORT]
    })
}

fn activation_value(
    sequence: u64,
    cohort: &str,
    previous: &str,
    descriptors: u64,
    methods: u64,
) -> Value {
    json!({
        "schema": "wuwa-reconstruction-curation-batch-activation-v1",
        "version": 1,
        "sequence": sequence,
        "cohort": cohort,
        "catalog_path": CATALOG_REL,
        "catalog_sha256": CATALOG_SHA256,
        "descriptor_count": descriptors,
        "method_count": methods,
        "raw_previous_prefix_sha256": previous
    })
}

fn gameplay_index_bytes() -> Vec<u8> {
    let mut bytes = canonical_line(&header_value());
    let prefix = sha256_hex(&bytes);
    bytes.extend_from_slice(&canonical_line(&activation_value(
        1,
        GAMEPLAY_COHORT,
        &prefix,
        66,
        310,
    )));
    bytes
}

fn indexed_fixture(index: &[u8]) -> FixtureRoot {
    let fixture = v5_fixture();
    fs::write(fixture.path.join(INDEX_REL), index).expect("write index fixture");
    fixture
}

fn p_fingerprint(manifest: &Manifest) -> PFields {
    let p = manifest
        .records
        .iter()
        .find(|record| record.id == "P")
        .expect("P record");
    PFields {
        id: p.id.clone(),
        original_path: p.original_path.clone(),
        original_size: p.original_size,
        original_sha256: p.original_sha256.clone(),
        evidence_status: format!("{:?}", p.evidence_status),
        source_status: format!("{:?}", p.source_status),
        type_def: p.type_def.clone(),
        verification_basis: p.verification_basis.clone(),
        scope: p.scope.clone(),
        completeness: p.completeness.clone(),
        representation: p.representation.clone(),
        namespace_context: p.namespace_context.clone(),
        relationship: p.relationship.clone(),
        explanation: p.explanation.clone(),
        dependencies: p.dependencies.clone(),
        methods: p
            .methods
            .current
            .iter()
            .map(|method| {
                format!(
                    "{}|{}|{}|{}|{:?}|{:?}|{}|{}",
                    method.symbol_key.type_def,
                    method.method_def,
                    method.symbol_key.mvid,
                    method.symbol_key.full_authority_sha256,
                    method.evidence_status,
                    method.source_status,
                    method.coverage,
                    method.verification_basis
                )
            })
            .collect(),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PFields {
    id: String,
    original_path: String,
    original_size: u64,
    original_sha256: String,
    evidence_status: String,
    source_status: String,
    type_def: Option<String>,
    verification_basis: String,
    scope: String,
    completeness: String,
    representation: String,
    namespace_context: Option<String>,
    relationship: String,
    explanation: String,
    dependencies: Vec<String>,
    methods: Vec<String>,
}

#[test]
fn approved_catalog_is_exact_deterministic_and_has_no_excluded_imports() {
    let root = curated_root();
    let path = root.join(CATALOG_REL);
    let bytes = fs::read(&path).expect("read approved catalog");
    assert_eq!(sha256_hex(&bytes), CATALOG_SHA256);
    assert_eq!(bytes.last(), Some(&b'\n'));
    let catalog = load_campaign_catalog(&root).expect("approved catalog validates");
    assert_eq!(catalog.schema, CATALOG_SCHEMA);
    assert_eq!(catalog.records.len(), 91);
    let gameplay: Vec<_> = catalog
        .records
        .iter()
        .filter(|record| record.cohort == GAMEPLAY_COHORT)
        .collect();
    let performance: Vec<_> = catalog
        .records
        .iter()
        .filter(|record| record.cohort == PERFORMANCE_COHORT)
        .collect();
    assert_eq!(gameplay.len(), 66);
    assert_eq!(performance.len(), 25);
    let gp_methods: Vec<_> = gameplay.iter().flat_map(|record| &record.methods).collect();
    let perf_methods: Vec<_> = performance
        .iter()
        .flat_map(|record| &record.methods)
        .collect();
    assert_eq!(gp_methods.len(), 310);
    assert_eq!(
        gp_methods
            .iter()
            .filter(|method| method.role == "ORDINARY")
            .count(),
        292
    );
    assert_eq!(
        gp_methods
            .iter()
            .filter(|method| method.role == "ASSOCIATED")
            .count(),
        18
    );
    assert_eq!(
        gp_methods
            .iter()
            .filter(|method| method.evidence_status == EvidenceStatus::DirectCilFull)
            .count(),
        152
    );
    assert_eq!(
        gp_methods
            .iter()
            .filter(|method| method.evidence_status == EvidenceStatus::SourceOnly)
            .count(),
        158
    );
    assert_eq!(perf_methods.len(), 119);
    assert_eq!(
        perf_methods
            .iter()
            .filter(|method| method.role == "ORDINARY")
            .count(),
        100
    );
    assert_eq!(
        perf_methods
            .iter()
            .filter(|method| method.role == "ASSOCIATED")
            .count(),
        19
    );
    assert_eq!(
        perf_methods
            .iter()
            .filter(|method| method.evidence_status == EvidenceStatus::DirectCilFull)
            .count(),
        47
    );
    assert_eq!(
        perf_methods
            .iter()
            .filter(|method| method.evidence_status == EvidenceStatus::SourceOnly)
            .count(),
        72
    );
    for method_def in [
        "0x0601A016",
        "0x0601A01E",
        "0x0601A01F",
        "0x0601A031",
        "0x0601A032",
        "0x0601A035",
        "0x06019FFB",
    ] {
        let method = gp_methods
            .iter()
            .find(|method| method.method_def == method_def)
            .expect("primary-only row");
        assert_eq!(method.evidence_status, EvidenceStatus::SourceOnly);
        assert!(
            method
                .reported_labels
                .contains(&"PRIMARY_FRESH_CIL".to_owned())
        );
        assert!(
            method
                .reported_labels
                .contains(&"FULL_SELECTED_BODY".to_owned())
        );
    }
    let inferred: Vec<_> = performance
        .iter()
        .filter(|record| record.source_status == SourceStatus::InferredConditional)
        .collect();
    assert_eq!(inferred.len(), 11);
    assert!(inferred.iter().all(|record| {
        record.methods.iter().all(|method| {
            method.source_status == SourceStatus::InferredConditional
                && method.evidence_status == EvidenceStatus::SourceOnly
        })
    }));
    let completion = performance
        .iter()
        .find(|record| record.source_path.ends_with("Completion.cs"))
        .expect("Completion descriptor");
    assert_eq!(completion.type_def, None);
    assert!(
        !completion
            .methods
            .iter()
            .any(|method| method.method_def == "0x06000101")
    );
    let getter = completion
        .methods
        .iter()
        .find(|method| method.method_def == "0x060000FD")
        .expect("getter FD");
    assert_eq!(getter.evidence_status, EvidenceStatus::SourceOnly);
    assert!(
        getter
            .reported_labels
            .contains(&"UNKNOWN_DIRECT_CIL".to_owned())
    );
    let namespaces: Vec<_> = performance
        .iter()
        .filter_map(|record| record.namespace_context.as_deref())
        .collect();
    assert!(namespaces.contains(&"CSharpScript.Game.World.Controller"));
    assert!(namespaces.contains(&"CSharpScript.Game.NewWorld.Character.Custom.Components"));
    assert_eq!(
        gameplay
            .iter()
            .find(|record| record.source_path.ends_with("GroupSkillCdInfo.cs"))
            .and_then(|record| record.namespace_context.as_deref()),
        Some("CSharpScript.Game.Module.Battle")
    );
    assert_eq!(
        gameplay
            .iter()
            .find(|record| record.source_path.ends_with("StateMachineCommon.Switch.cs"))
            .and_then(|record| record.namespace_context.as_deref()),
        Some("CSharpScript.Core.Utils.StateMachine")
    );
    assert!(!catalog.records.iter().any(|record| record.id == "H"));
    assert!(
        !catalog
            .records
            .iter()
            .any(|record| record.source_path.ends_with("Event.RangeReceiver.cs"))
    );
    assert!(!catalog.records.iter().any(|record| {
        record
            .source_path
            .ends_with("RangeComponent.PendingEvents.cs")
    }));
    assert!(
        catalog
            .records
            .iter()
            .any(|record| record.source_path.ends_with("PendingEventBoundaries.cs"))
    );
    assert!(catalog.records.iter().any(|record| {
        record
            .source_path
            .ends_with("Calculation.FormulaOneInvocation.cs")
    }));
    assert!(
        !catalog
            .records
            .iter()
            .any(|record| record.source_path.contains("StateMachineCommon.Enter"))
    );
    let all_methods: Vec<_> = catalog
        .records
        .iter()
        .flat_map(|record| &record.methods)
        .collect();
    assert_eq!(
        all_methods
            .iter()
            .filter(|method| method.role == "ORDINARY")
            .count(),
        392
    );
    // P contributes the other two ordinary selected methods from its existing record.
    assert_eq!(
        all_methods
            .iter()
            .filter(|method| method.role == "ASSOCIATED")
            .count(),
        37
    );
    assert_eq!(
        catalog
            .records
            .iter()
            .filter(|record| record.existing_record_id.as_deref() == Some("P"))
            .count(),
        1
    );
    let p = catalog
        .records
        .iter()
        .find(|record| record.existing_record_id.as_deref() == Some("P"))
        .expect("P descriptor");
    assert!(p.methods.is_empty());
    assert_eq!(
        p.destination_path,
        "src/managed/cs-e3fc67839851/loading/GameModePromise.cs"
    );
    let active_buff = gameplay
        .iter()
        .find(|record| record.source_path.ends_with("ActiveBuffInternal.cs"))
        .expect("ActiveBuffInternal");
    assert_eq!(active_buff.methods.len(), 9);
    let active_dependencies = active_buff.dependencies.join(" ");
    assert!(active_dependencies.contains("BuffDefinition"));
    assert!(active_dependencies.contains("13 expression-bodied"));
    assert!(active_dependencies.contains("no qualified MethodDef"));
    assert!(
        active_buff
            .completeness
            .contains("13 source-context properties")
    );
    assert!(active_buff.completeness.contains("no MethodDef"));
    let formula = gameplay
        .iter()
        .find(|record| record.id == "G_CALCULATION_FORMULAONEINVOCATION")
        .expect("Formula");
    assert!(
        formula
            .relationship
            .contains("Calculation.DamageFormula.cs")
    );
    assert!(formula.relationship.contains("Calculation.cs"));
    assert!(
        formula
            .relationship
            .contains("DISJOINT_SELECTED_METHOD_COMPLEMENT")
    );
    assert!(
        formula
            .dependencies
            .iter()
            .any(|dep| dep.contains("0x06049F94..0x06049F9A"))
    );
    assert!(formula.completeness.contains("seven callbacks"));
    assert!(
        gameplay
            .iter()
            .all(|record| !record.dependencies.is_empty())
    );
    assert_eq!(
        gameplay
            .iter()
            .find(|record| record.source_path.ends_with("PassiveSkillCdInfo.cs"))
            .and_then(|record| record.namespace_context.as_deref()),
        Some("CSharpScript.Game.Module.Battle")
    );
    assert_eq!(
        performance
            .iter()
            .filter(|record| {
                record.namespace_context.as_deref()
                    == Some("CSharpScript.Game.NewWorld.Character.Custom.Components")
            })
            .count(),
        2
    );
    assert_eq!(
        performance
            .iter()
            .find(|record| record.id == "P_PENDINGEVENTBOUNDARIES")
            .and_then(|record| record.namespace_context.as_deref()),
        Some("global (observed)")
    );
    for (ids, td) in [
        (
            ["G_BASEATTRIBUTECOMPONENT", "G_BASEATTRIBUTESTORAGE"].as_slice(),
            "0x02002E53",
        ),
        (
            [
                "G_BASEDAMAGECOMPONENT",
                "G_BASEDAMAGECOMPONENT_EXECDAMAGEEVENT",
            ]
            .as_slice(),
            "0x02002E65",
        ),
        (
            [
                "G_CHARACTERMOVECOMPONENT",
                "G_CHARACTERMOVECOMPONENT_JUMPHANDOFF",
                "G_CHARACTERMOVECOMPONENT_LANDING",
            ]
            .as_slice(),
            "0x0200305B",
        ),
        (
            [
                "G_CHARACTERSTATISTICSCOMPONENT_DAMAGECOMBAT",
                "G_CHARACTERSTATISTICSCOMPONENT_DAMAGERECEIVER",
                "G_CHARACTERSTATISTICSCOMPONENT_LISTVIEWFILTER",
            ]
            .as_slice(),
            "0x02002ED4",
        ),
        (["G_SKILL", "G_SKILLEFFECTS"].as_slice(), "0x0200312B"),
        (
            ["G_ROLEELEMENTCOMPONENT", "G_ROLEELEMENTRESOURCE"].as_slice(),
            "0x020031EE",
        ),
        (
            ["P_SUBLEVEL", "P_SUBLEVEL_VISIBILITY"].as_slice(),
            "0x0200347D",
        ),
        (
            [
                "P_SUBLEVELCONTROLLER_EXECUTION",
                "P_SUBLEVELCONTROLLER_STREAMING",
            ]
            .as_slice(),
            "0x020046E8",
        ),
        (
            ["P_RANGECOMPONENT", "P_RANGECOMPONENT_LIFECYCLERECOVERY"].as_slice(),
            "0x020048E7",
        ),
    ] {
        let mut seen = BTreeSet::new();
        for id in ids {
            let record = catalog
                .records
                .iter()
                .find(|record| record.id == *id)
                .unwrap_or_else(|| panic!("missing {id}"));
            assert!(
                record
                    .relationship
                    .contains("DISJOINT_SELECTED_METHOD_COMPLEMENT")
            );
            for method in &record.methods {
                if method.type_def == td {
                    assert!(
                        seen.insert(method.method_def.clone()),
                        "duplicate {id} {td}"
                    );
                }
            }
        }
    }
    let range = performance
        .iter()
        .find(|record| record.id == "P_RANGECOMPONENT")
        .expect("RangeComponent");
    assert!(
        range
            .relationship
            .contains("RangeComponent.PendingEvents.cs")
    );
    assert!(range.relationship.contains("four unique methods"));
    assert!(range.completeness.contains("2 partial lifecycle overlaps"));
    let ids: BTreeSet<_> = catalog
        .records
        .iter()
        .map(|record| record.id.as_str())
        .collect();
    assert_eq!(ids.len(), catalog.records.len());
    let destinations: BTreeSet<_> = catalog
        .records
        .iter()
        .map(|record| record.destination_path.to_ascii_lowercase())
        .collect();
    assert_eq!(destinations.len(), catalog.records.len());
    for record in &catalog.records {
        assert!(record.namespace_context.is_some());
        assert!(
            record
                .destination_path
                .starts_with("src/managed/cs-e3fc67839851/")
        );
        let source = root
            .parent()
            .expect("research root")
            .join(&record.source_path);
        let source_bytes = fs::read(&source).expect("read catalog source");
        assert_eq!(source_bytes.len() as u64, record.source_size);
        assert_eq!(sha256_hex(&source_bytes), record.source_sha256);
    }
    assert_eq!(bytes, fs::read(&path).expect("re-read catalog"));
}

#[test]
fn canonical_index_rejects_noncanonical_and_prefix_drift() {
    let mut index = canonical_line(&header_value());
    let header_prefix = sha256_hex(&index);
    index.extend_from_slice(&canonical_line(&activation_value(
        1,
        GAMEPLAY_COHORT,
        &header_prefix,
        66,
        310,
    )));
    let parsed = parse_index_bytes(&index).expect("canonical G index");
    assert_eq!(parsed.activations.len(), 1);
    let next = append_activation_bytes(&index, PERFORMANCE_COHORT).expect("append P");
    assert!(next.starts_with(&index));
    assert_eq!(parse_index_bytes(&next).expect("G+P").activations.len(), 2);
    assert!(append_activation_bytes(&index, GAMEPLAY_COHORT).is_err());
    assert!(append_activation_bytes(&canonical_line(&header_value()), PERFORMANCE_COHORT).is_err());
    let mut truncated = index.clone();
    truncated.pop();
    assert!(parse_index_bytes(&truncated).is_err());
    let mut whitespace = index.clone();
    whitespace.insert(1, b' ');
    assert!(parse_index_bytes(&whitespace).is_err());
    let crlf = String::from_utf8(index.clone())
        .expect("utf8")
        .replace('\n', "\r\n");
    assert!(parse_index_bytes(crlf.as_bytes()).is_err());
    let mut extra = header_value();
    extra["extra_field"] = json!(true);
    assert!(parse_index_bytes(&canonical_line(&extra)).is_err());
    let header_text = String::from_utf8(canonical_line(&header_value())).expect("utf8");
    let duplicate = header_text.replacen("\"version\":1", "\"version\":1,\"version\":1", 1);
    assert!(parse_index_bytes(duplicate.as_bytes()).is_err());
    let mut bad_prefix = index.clone();
    bad_prefix.extend_from_slice(&canonical_line(&activation_value(
        2,
        PERFORMANCE_COHORT,
        &"00".repeat(32),
        25,
        121,
    )));
    assert!(parse_index_bytes(&bad_prefix).is_err());
    let reordered = format!(
        "{{\"version\":1,\"schema\":\"{}\",\"campaign\":\"{}\",\"baseline_v5_sha256\":\"{}\",\"frozen_snapshot_id\":\"{}\",\"frozen_registry_receipt_sha256\":\"{}\",\"catalog_path\":\"{}\",\"catalog_sha256\":\"{}\",\"approved_cohorts\":[\"{}\",\"{}\"]}}\n",
        INDEX_SCHEMA,
        CAMPAIGN_ID,
        BASELINE_V5_RECEIPT_SHA256,
        FROZEN_SNAPSHOT_ID,
        FROZEN_REGISTRY_RECEIPT_SHA256,
        CATALOG_REL,
        CATALOG_SHA256,
        GAMEPLAY_COHORT,
        PERFORMANCE_COHORT
    );
    assert!(parse_index_bytes(reordered.as_bytes()).is_err());
}

#[test]
fn activated_g_only_and_g_plus_p_preserve_v5_and_reuse_p() {
    let fixture = v5_fixture();
    let catalog = load_campaign_catalog(&curated_root()).expect("catalog");
    let mut manifest = actual_manifest();
    let mut active = Some(
        load_active_receipt(&fixture.path)
            .expect("fixture active")
            .expect("v5 active"),
    );
    let p_before = p_fingerprint(&manifest);
    apply_activated_catalog(
        &mut manifest,
        &mut active,
        &catalog,
        &[GAMEPLAY_COHORT.to_owned()],
    )
    .expect("activate G");
    let active = active.as_ref().expect("G active");
    assert_eq!(active.version, 6);
    assert_eq!(active.files.len(), 79);
    assert_eq!(
        active
            .files
            .iter()
            .map(|file| file.methods.len())
            .sum::<usize>(),
        412
    );
    assert!(!active.files.iter().any(|file| file.id == "P"));
    assert_eq!(p_fingerprint(&manifest), p_before);
    let group = manifest
        .records
        .iter()
        .find(|record| record.id == "G_GROUPSKILLCDINFO")
        .expect("GroupSkill record");
    assert_eq!(
        group.namespace_context.as_deref(),
        Some("CSharpScript.Game.Module.Battle")
    );
    assert!(!group.proposed_path_reason.contains("Phase B"));
    assert_eq!(
        group.validation.final_receipt,
        "EXTERNAL_PRIMARY_RECEIPT_REQUIRED"
    );
    assert_eq!(
        group.validation.compilation,
        "NOT_BUILDABLE_WITH_CURRENT_CONTEXT"
    );

    let fixture2 = v5_fixture();
    let mut manifest2 = actual_manifest();
    let mut active2 = Some(
        load_active_receipt(&fixture2.path)
            .expect("fixture2 active")
            .expect("v5 active"),
    );
    apply_activated_catalog(
        &mut manifest2,
        &mut active2,
        &catalog,
        &[GAMEPLAY_COHORT.to_owned(), PERFORMANCE_COHORT.to_owned()],
    )
    .expect("activate G+P");
    let active2 = active2.as_ref().expect("G+P active");
    assert_eq!(active2.files.len(), 104);
    assert_eq!(
        active2
            .files
            .iter()
            .map(|file| file.methods.len())
            .sum::<usize>(),
        533
    );
    assert_eq!(
        active2.files.iter().filter(|file| file.id == "P").count(),
        1
    );
    assert_eq!(
        active2.files.iter().filter(|file| file.id == "H").count(),
        0
    );
    assert_eq!(p_fingerprint(&manifest2), p_before);
    assert!(
        manifest2
            .records
            .iter()
            .any(|record| record.id == "G_CALCULATION_FORMULAONEINVOCATION")
    );
    for (id, expected) in [
        (
            "P_SUBLEVELCONTROLLER_STREAMING",
            "CSharpScript.Game.World.Controller",
        ),
        (
            "P_RANGECOMPONENT",
            "CSharpScript.Game.NewWorld.Character.Custom.Components",
        ),
        (
            "P_RANGECOMPONENT_LIFECYCLERECOVERY",
            "CSharpScript.Game.NewWorld.Character.Custom.Components",
        ),
        ("P_PENDINGEVENTBOUNDARIES", "global (observed)"),
    ] {
        let record = manifest2
            .records
            .iter()
            .find(|record| record.id == id)
            .unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(record.namespace_context.as_deref(), Some(expected), "{id}");
    }
}

#[test]
fn duplicate_p_key_and_inferred_or_primary_promotion_are_rejected() {
    let fixture = v5_fixture();
    let manifest = actual_manifest();
    let p = manifest
        .records
        .iter()
        .find(|record| record.id == "P")
        .expect("P");
    let p_td = p.methods.current[0].symbol_key.type_def.clone();
    let p_md = p.methods.current[0].method_def.clone();

    let mut duplicate = load_campaign_catalog(&curated_root()).expect("catalog");
    let record = duplicate
        .records
        .iter_mut()
        .find(|record| record.cohort == PERFORMANCE_COHORT && record.existing_record_id.is_none())
        .expect("new performance descriptor");
    record.type_def = None;
    let method = &mut record.methods[0];
    method.type_def = p_td.clone();
    method.root_type = p_td.clone();
    method.method_def = p_md.clone();
    method.evidence_status = EvidenceStatus::SourceOnly;
    method.source_status = SourceStatus::SourceOnly;
    let mut active = Some(
        load_active_receipt(&fixture.path)
            .expect("fixture active")
            .expect("v5 active"),
    );
    let mut manifest = manifest;
    let error = apply_activated_catalog(
        &mut manifest,
        &mut active,
        &duplicate,
        &[GAMEPLAY_COHORT.to_owned(), PERFORMANCE_COHORT.to_owned()],
    )
    .expect_err("P key collision must reject");
    assert_eq!(error.code, "BULK_QUALIFIED_METHOD_OVERLAP");

    let mut inferred = load_campaign_catalog(&curated_root()).expect("catalog");
    let inferred_record = inferred
        .records
        .iter_mut()
        .find(|record| record.source_status == SourceStatus::InferredConditional)
        .expect("inferred descriptor");
    inferred_record.methods[0].evidence_status = EvidenceStatus::DirectCilFull;
    inferred_record.methods[0].source_status = SourceStatus::SourceOnly;
    assert_eq!(
        re_corpus::bulk::validate_campaign_catalog(&inferred)
            .expect_err("inferred promotion rejected")
            .code,
        "BULK_INFERRED_EVIDENCE_PROMOTION"
    );

    let mut primary = load_campaign_catalog(&curated_root()).expect("catalog");
    let base_skill = primary
        .records
        .iter_mut()
        .find(|record| record.source_path.ends_with("BaseSkillComponent.cs"))
        .expect("BaseSkill");
    let method = base_skill
        .methods
        .iter_mut()
        .find(|method| method.method_def == "0x0601A016")
        .expect("primary only");
    method.evidence_status = EvidenceStatus::DirectCilFull;
    assert!(re_corpus::bulk::validate_campaign_catalog(&primary).is_err());
}

#[test]
fn production_apply_route_copies_then_activates_and_public_dtos_see_v6() {
    let fixture = v5_fixture();
    copy_base_imports(&fixture);
    let source_root = curated_root()
        .parent()
        .expect("research root")
        .to_path_buf();
    let g = apply_approved_cohort(&source_root, &fixture.path, GAMEPLAY_COHORT)
        .expect("production G apply");
    assert!(g.copy.complete);
    assert_eq!(g.copy.files.len(), 66);
    assert!(fixture.path.join(INDEX_REL).is_file());
    let index_g = fs::read(fixture.path.join(INDEX_REL)).expect("index G");
    assert!(index_g.ends_with(b"\n"));
    assert_eq!(
        parse_index_bytes(&index_g)
            .expect("parse G")
            .activations
            .len(),
        1
    );
    let (exit, status) = run_status_at(&fixture.path);
    assert_eq!(exit, 0, "status={status}");
    assert_eq!(status["active_ledger"]["version"], 6);
    assert_eq!(status["counts"]["imported_file_count"], 79);
    assert_eq!(status["counts"]["imported_qualified_methods"], 412);
    let (lookup_exit, lookup) =
        run_lookup_at(&fixture.path, "MANAGED-CS-E3FC67839851", "0x06018666");
    assert_eq!(lookup_exit, 0);
    assert_eq!(lookup["matches"][0]["id"], "G_BASEDAMAGECOMPONENT");
    let (rooted_exit, rooted) = run_verify_at(&fixture.path, Some(source_root.clone()));
    assert_eq!(rooted_exit, 0, "rooted={rooted}");
    let (unrooted_exit, unrooted) = run_verify_at(&fixture.path, None);
    assert_eq!(unrooted_exit, 1);
    assert!(
        unrooted["findings"]
            .as_array()
            .expect("findings")
            .iter()
            .any(|finding| finding["code"] == "REQUIRED_ORIGINALS_NOT_CHECKED")
    );
    let active = load_active_receipt(&fixture.path)
        .expect("active")
        .expect("v6");
    let expected_ids: BTreeSet<_> = active.files.iter().map(|file| file.id.as_str()).collect();
    let unchecked_ids: BTreeSet<_> = unrooted["not_checked"]
        .as_array()
        .expect("not checked")
        .iter()
        .filter(|item| item["kind"] == "ORIGINAL_SOURCE")
        .filter_map(|item| item["id"].as_str())
        .collect();
    assert!(expected_ids.iter().all(|id| unchecked_ids.contains(id)));

    let p = apply_approved_cohort(&source_root, &fixture.path, PERFORMANCE_COHORT)
        .expect("production P apply");
    assert!(p.copy.complete);
    assert_eq!(p.copy.files.len(), 25);
    let index_gp = fs::read(fixture.path.join(INDEX_REL)).expect("index G+P");
    assert!(index_gp.starts_with(&index_g));
    assert_eq!(
        parse_index_bytes(&index_gp)
            .expect("parse G+P")
            .activations
            .len(),
        2
    );
    let active = load_active_receipt(&fixture.path)
        .expect("active")
        .expect("v6");
    assert_eq!(active.files.len(), 104);
    assert_eq!(
        active
            .files
            .iter()
            .map(|file| file.methods.len())
            .sum::<usize>(),
        533
    );
}

#[test]
fn production_route_rejects_missing_v5_invalid_index_and_collision_without_activation() {
    let source_root = curated_root()
        .parent()
        .expect("research root")
        .to_path_buf();
    let missing_v5 = v5_fixture();
    fs::remove_file(missing_v5.path.join("metadata/imports.v5.json")).expect("remove v5");
    assert!(apply_approved_cohort(&source_root, &missing_v5.path, GAMEPLAY_COHORT).is_err());
    assert!(!missing_v5.path.join(INDEX_REL).exists());

    let invalid = v5_fixture();
    fs::write(invalid.path.join(INDEX_REL), b"{}\n").expect("invalid index");
    let invalid_result = apply_approved_cohort(&source_root, &invalid.path, GAMEPLAY_COHORT);
    assert!(invalid_result.is_err());
    assert_eq!(
        fs::read(invalid.path.join(INDEX_REL)).expect("invalid unchanged"),
        b"{}\n"
    );

    let collision = v5_fixture();
    copy_base_imports(&collision);
    let catalog = load_campaign_catalog(&curated_root()).expect("catalog");
    let target = collision.path.join(
        &catalog
            .records
            .iter()
            .find(|record| record.cohort == GAMEPLAY_COHORT)
            .expect("G descriptor")
            .destination_path,
    );
    fs::create_dir_all(target.parent().expect("parent")).expect("parent");
    fs::write(&target, b"preexisting").expect("collision file");
    let failed = apply_approved_cohort(&source_root, &collision.path, GAMEPLAY_COHORT)
        .expect_err("collision must stop before activation");
    assert!(failed.copy_failure.is_some());
    assert!(!collision.path.join(INDEX_REL).exists());
    assert_eq!(
        fs::read(&target).expect("collision unchanged"),
        b"preexisting"
    );
}

#[cfg(windows)]
#[test]
fn production_route_rejects_junction_destination_and_receipt_parent() {
    let fixture = v5_fixture();
    copy_base_imports(&fixture);
    let outside = FixtureRoot::new();
    let catalog = load_campaign_catalog(&curated_root()).expect("catalog");
    let destination = catalog
        .records
        .iter()
        .find(|record| record.cohort == GAMEPLAY_COHORT)
        .expect("G descriptor")
        .destination_path
        .clone();
    let parent_relative = destination.rsplit_once('/').expect("parent").0.to_owned();
    let parent = fixture.path.join(&parent_relative);
    fs::create_dir_all(parent.parent().expect("grandparent")).expect("grandparent dirs");
    let guard = JunctionGuard::new(&parent, &outside.path);
    let source_root = curated_root()
        .parent()
        .expect("research root")
        .to_path_buf();
    let failure = apply_approved_cohort(&source_root, &fixture.path, GAMEPLAY_COHORT)
        .expect_err("junction destination must reject");
    assert_eq!(
        failure.copy_failure.as_ref().expect("copy failure").code,
        "BULK_COPY_DESTINATION_UNSAFE"
    );
    assert!(!fixture.path.join(INDEX_REL).exists());
    assert_eq!(fs::read_dir(&outside.path).expect("outside dir").count(), 0);
    let receipt = re_corpus::bulk::CopyReceipt {
        campaign: CAMPAIGN_ID.to_owned(),
        cohort: GAMEPLAY_COHORT.to_owned(),
        files: Vec::new(),
        complete: false,
        failed_attempt: None,
    };
    assert!(
        write_copy_receipt(
            &fixture.path,
            &format!("{}/receipt.json", parent_relative),
            &receipt,
        )
        .is_err()
    );
    assert_eq!(fs::read_dir(&outside.path).expect("outside dir").count(), 0);
    drop(guard);
}

#[test]
fn isolated_v5_fixture_preserves_public_fallback_and_apply_index_contracts() {
    let fixture = v5_fixture();
    copy_base_imports(&fixture);
    let active = load_active_receipt(&fixture.path)
        .expect("fixture active")
        .expect("v5 active");
    assert_eq!(active.version, 5);
    assert_eq!(active.files.len(), 13);
    assert_eq!(
        active
            .files
            .iter()
            .map(|file| file.methods.len())
            .sum::<usize>(),
        102
    );
    let (exit, status) = run_status_at(&fixture.path);
    assert_eq!(exit, 0);
    assert_eq!(status["active_ledger"]["version"], 5);
    assert_eq!(status["counts"]["imported_file_count"], 13);

    let index = gameplay_index_bytes();
    let indexed = indexed_fixture(&index);
    let mut manifest = actual_manifest();
    let mut active = Some(
        load_active_receipt(&fixture.path)
            .expect("base fixture active")
            .expect("v5 active"),
    );
    apply_index(&indexed.path, &mut manifest, &mut active).expect("apply_index G");
    let active = active.expect("v6");
    assert_eq!(active.version, 6);
    assert_eq!(active.files.len(), 79);

    let missing_v5 = indexed_fixture(&index);
    let mut manifest = actual_manifest();
    let mut active = Some(
        load_active_receipt(&fixture.path)
            .expect("base fixture active")
            .expect("v5 active"),
    );
    fs::remove_file(missing_v5.path.join("metadata/imports.v5.json")).expect("remove v5");
    assert!(apply_index(&missing_v5.path, &mut manifest, &mut active).is_err());

    let corrupt = indexed_fixture(&index);
    let mut manifest = actual_manifest();
    let mut active = Some(
        load_active_receipt(&fixture.path)
            .expect("base fixture active")
            .expect("v5 active"),
    );
    let catalog_path = corrupt.path.join(CATALOG_REL);
    let mut catalog_bytes = fs::read(&catalog_path).expect("catalog");
    catalog_bytes[0] ^= 1;
    fs::write(&catalog_path, catalog_bytes).expect("corrupt catalog");
    assert!(apply_index(&corrupt.path, &mut manifest, &mut active).is_err());
}

#[test]
fn receipt_writer_is_relative_and_create_new_only() {
    let fixture = FixtureRoot::new();
    let receipt = re_corpus::bulk::CopyReceipt {
        campaign: CAMPAIGN_ID.to_owned(),
        cohort: GAMEPLAY_COHORT.to_owned(),
        files: Vec::new(),
        complete: false,
        failed_attempt: None,
    };
    write_copy_receipt(&fixture.path, "copy-receipt.json", &receipt).expect("write receipt");
    assert!(fixture.path.join("copy-receipt.json").is_file());
    assert!(write_copy_receipt(&fixture.path, "../escape.json", &receipt).is_err());
    assert!(write_copy_receipt(&fixture.path, "copy-receipt.json", &receipt).is_err());
}
