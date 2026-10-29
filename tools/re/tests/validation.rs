use re_corpus::bulk::INDEX_REL;
use re_corpus::{
    ActiveReceipt, ImportEntry, ImportEntryV2, ImportReceiptV3, ImportReceiptV4, ImportReceiptV5,
    ImportedFileView, MANAGED_AUTHORITY_ID, MANIFEST_REL, Manifest, PERFORMANCE_SOURCE_REL,
    PerformanceSourceCatalog, RECEIPT_REL, RECEIPT_V2_REL, RECEIPT_V3_REL, RECEIPT_V4_REL,
    RECEIPT_V5_REL, REQUIRED_IMPORT_IDS, SCHEDULING_SOURCE_REL, SchedulingSourceCatalog,
    TASK_EXECUTION_SOURCE_REL, TaskExecutionSourceCatalog, check_import_file,
    check_imported_file_view, curated_root, load_active_receipt, parse_manifest_bytes,
    parse_performance_source_bytes, parse_receipt_bytes, parse_receipt_v2_bytes,
    parse_receipt_v3_bytes, parse_receipt_v4_bytes, parse_receipt_v5_bytes,
    parse_scheduling_source_bytes, parse_strict_value, parse_task_execution_source_bytes,
    reparse_attributes_rejected, run_lookup_at, run_status_at, run_verify_at, run_with_args,
    validate_relative_path,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct FixtureRoot {
    path: PathBuf,
}

impl FixtureRoot {
    fn new() -> Self {
        let number = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("test-fixtures")
            .join(format!("{}-{number}", std::process::id()));
        fs::create_dir_all(&path).expect("create owned fixture root");
        Self { path }
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(windows)]
struct JunctionCase {
    link: PathBuf,
    root: Option<FixtureRoot>,
    target: Option<FixtureRoot>,
    active: bool,
}

#[cfg(windows)]
impl JunctionCase {
    fn new(link: PathBuf, root: FixtureRoot, target: FixtureRoot, active: bool) -> Self {
        Self {
            link,
            root: Some(root),
            target: Some(target),
            active,
        }
    }

    fn root_path(&self) -> &Path {
        &self.root.as_ref().expect("owned junction root").path
    }

    fn target_path(&self) -> &Path {
        &self.target.as_ref().expect("owned junction target").path
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

    fn cleanup(mut self) {
        assert!(self.unlink(), "explicit junction unlink failed");
        let root = self.root.take().expect("owned junction root");
        let target = self.target.take().expect("owned junction target");
        let root_path = root.path.clone();
        let target_path = target.path.clone();
        drop(root);
        drop(target);
        assert!(
            !root_path.exists(),
            "junction root cleanup left {}",
            root_path.display()
        );
        assert!(
            !target_path.exists(),
            "junction target cleanup left {}",
            target_path.display()
        );
    }
}

#[cfg(windows)]
impl Drop for JunctionCase {
    fn drop(&mut self) {
        if !self.unlink() {
            if let Some(root) = self.root.take() {
                std::mem::forget(root);
            }
            if let Some(target) = self.target.take() {
                std::mem::forget(target);
            }
        }
    }
}

#[cfg(windows)]
fn create_test_junction(link: &std::path::Path, target: &std::path::Path) -> (i32, String, String) {
    let link = link.to_string_lossy().replace('/', "\\");
    let target = target.to_string_lossy().replace('/', "\\");
    let output = std::process::Command::new("cmd.exe")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()
        .expect("run native cmd.exe mklink /J");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

fn run(values: &[&str]) -> (u8, Value) {
    run_with_args(args(values))
}

fn manifest_bytes() -> Vec<u8> {
    fs::read(curated_root().join(MANIFEST_REL)).expect("read accepted manifest")
}

fn manifest() -> Manifest {
    parse_manifest_bytes(&manifest_bytes()).expect("accepted manifest parses")
}

fn manifest_value() -> Value {
    serde_json::from_slice(&manifest_bytes()).expect("accepted manifest is JSON")
}

fn record_mut<'a>(value: &'a mut Value, id: &str) -> &'a mut Value {
    value["records"]
        .as_array_mut()
        .expect("records array")
        .iter_mut()
        .find(|record| record["id"] == id)
        .expect("record exists")
}

fn parse_mutated(value: Value) -> re_corpus::ValidationError {
    let bytes = serde_json::to_vec(&value).expect("serialize fixture JSON");
    parse_manifest_bytes(&bytes).expect_err("mutation must be rejected")
}

fn import_value(manifest: &Manifest, id: &str, imported_sha256: &str, imported_size: u64) -> Value {
    let record = manifest
        .records
        .iter()
        .find(|record| record.id == id)
        .expect("approved record");
    let method = record.methods.current.first().expect("selected method");
    json!({
        "schema": "wuwa-reconstruction-imports-v1",
        "version": 1,
        "manifest_sha256": re_corpus::BASELINE_MANIFEST_SHA256,
        "imports": [{
            "id": id,
            "authority_id": record.authority_id,
            "method_identity": {
                "full_authority_sha256": &method.symbol_key.full_authority_sha256,
                "mvid": &method.symbol_key.mvid,
                "typeDef": &method.symbol_key.type_def,
                "methodDef": &method.symbol_key.method_def
            },
            "original_path": record.original_path,
            "imported_path": record.proposed_path,
            "original_size": record.original_size,
            "imported_size": imported_size,
            "original_sha256": record.original_sha256,
            "imported_sha256": imported_sha256,
            "exact_byte_comparison": "PASS",
            "implementation_scope": record.scope,
            "source_completeness": record.completeness,
            "unresolved_dependencies": record.dependencies,
            "editorial_status": "NEEDS_EDITORIAL_REVIEW",
            "compilation_status": "NOT_RUN",
            "equivalence_status": "NOT_ESTABLISHED",
            "publication_status": "DISABLED",
            "runtime_status": "PARKED"
        }]
    })
}

fn import_entry(manifest: &Manifest, id: &str, bytes: &[u8]) -> ImportEntry {
    let value = import_value(
        manifest,
        id,
        &re_corpus::sha256_hex(bytes),
        bytes.len() as u64,
    );
    let receipt = parse_receipt_bytes(
        &serde_json::to_vec(&value).expect("serialize receipt fixture"),
        manifest,
    )
    .expect("receipt fixture validates");
    receipt.imports.into_iter().next().expect("one import")
}

fn receipt_bytes() -> Vec<u8> {
    fs::read(curated_root().join(RECEIPT_REL)).expect("read accepted receipt")
}

fn historical_receipt() -> re_corpus::ImportReceipt {
    parse_receipt_bytes(&receipt_bytes(), &manifest()).expect("accepted historical receipt")
}

fn manifest_record_value<'a>(value: &'a Value, id: &str) -> &'a Value {
    value["records"]
        .as_array()
        .expect("records array")
        .iter()
        .find(|record| record["id"] == id)
        .expect("manifest record")
}

fn mapped_import_value(manifest_value: &Value, id: &str) -> Value {
    let record = manifest_record_value(manifest_value, id);
    let methods = record["methods"]["current"]
        .as_array()
        .expect("current method array")
        .iter()
        .map(|method| {
            json!({
                "method_identity": method["symbol_key"].clone(),
                "evidence_status": method["evidence_status"].clone(),
                "source_status": method["source_status"].clone()
            })
        })
        .collect::<Vec<_>>();
    json!({
        "id": id,
        "authority_id": record["authority_id"].clone(),
        "methods": methods,
        "original_path": record["original_path"].clone(),
        "imported_path": record["proposed_path"].clone(),
        "original_size": record["original_size"].clone(),
        "imported_size": record["original_size"].clone(),
        "original_sha256": record["original_sha256"].clone(),
        "imported_sha256": record["original_sha256"].clone(),
        "exact_byte_comparison": "PASS",
        "implementation_scope": record["scope"].clone(),
        "source_completeness": record["completeness"].clone(),
        "unresolved_dependencies": record["dependencies"].clone(),
        "evidence_status": record["evidence_status"].clone(),
        "source_status": record["source_status"].clone(),
        "editorial_status": "NEEDS_EDITORIAL_REVIEW",
        "compilation_status": "NOT_RUN",
        "equivalence_status": "NOT_ESTABLISHED",
        "publication_status": "DISABLED",
        "runtime_status": "PARKED"
    })
}

fn v2_value() -> Value {
    let historical: Value = serde_json::from_slice(&receipt_bytes()).expect("historical JSON");
    let mut imports = historical["imports"]
        .as_array()
        .expect("historical imports")
        .clone();
    let manifest = manifest_value();
    imports.push(mapped_import_value(&manifest, "D"));
    imports.push(mapped_import_value(&manifest, "T"));
    json!({
        "schema": "wuwa-reconstruction-imports-v2",
        "version": 2,
        "manifest_sha256": re_corpus::BASELINE_MANIFEST_SHA256,
        "previous_receipt_sha256": re_corpus::BASELINE_RECEIPT_V1_SHA256,
        "imports": imports
    })
}

fn parse_v2_value(value: Value) -> Result<re_corpus::ImportReceiptV2, re_corpus::ValidationError> {
    parse_receipt_v2_bytes(
        &serde_json::to_vec(&value).expect("serialize v2 fixture"),
        &manifest(),
        &historical_receipt(),
    )
}

fn v2_entry_mut<'a>(value: &'a mut Value, id: &str) -> &'a mut Value {
    value["imports"]
        .as_array_mut()
        .expect("v2 imports")
        .iter_mut()
        .find(|entry| entry["id"] == id)
        .expect("v2 entry")
}

fn write_fixture_receipts(fixture: &FixtureRoot, v2: Option<&Value>, v1_bytes: &[u8]) {
    let metadata = fixture.path.join("metadata");
    fs::create_dir_all(&metadata).expect("create fixture metadata");
    fs::write(metadata.join("migration.json"), manifest_bytes()).expect("write fixture manifest");
    fs::write(metadata.join("imports.v1.json"), v1_bytes).expect("write fixture v1");
    if let Some(value) = v2 {
        fs::write(
            metadata.join("imports.v2.json"),
            serde_json::to_vec(value).expect("serialize fixture v2"),
        )
        .expect("write fixture v2");
    }
}

fn fixture_active(v2: Option<&Value>) -> (FixtureRoot, ActiveReceipt) {
    let fixture = FixtureRoot::new();
    write_fixture_receipts(&fixture, v2, &receipt_bytes());
    let active = load_active_receipt(&fixture.path)
        .expect("fixture active receipt loads")
        .expect("fixture active receipt exists");
    (fixture, active)
}

fn catalog_bytes() -> Vec<u8> {
    fs::read(curated_root().join(PERFORMANCE_SOURCE_REL)).expect("read performance source catalog")
}

fn catalog_value() -> Value {
    serde_json::from_slice(&catalog_bytes()).expect("performance source catalog is JSON")
}

fn performance_catalog() -> PerformanceSourceCatalog {
    parse_performance_source_bytes(&catalog_bytes(), &manifest())
        .expect("performance source catalog validates")
}

fn catalog_record_value<'a>(catalog: &'a Value, id: &str) -> &'a Value {
    catalog["records"]
        .as_array()
        .expect("catalog records array")
        .iter()
        .find(|record| record["id"] == id)
        .expect("catalog record")
}

fn catalog_record_mut<'a>(catalog: &'a mut Value, id: &str) -> &'a mut Value {
    catalog["records"]
        .as_array_mut()
        .expect("catalog records array")
        .iter_mut()
        .find(|record| record["id"] == id)
        .expect("catalog record")
}

fn parse_catalog_value(
    value: Value,
) -> Result<PerformanceSourceCatalog, re_corpus::ValidationError> {
    parse_performance_source_bytes(
        &serde_json::to_vec(&value).expect("serialize catalog fixture"),
        &manifest(),
    )
}

fn receipt_v2_bytes() -> Vec<u8> {
    fs::read(curated_root().join(RECEIPT_V2_REL)).expect("read accepted v2 receipt")
}

fn historical_receipt_v2() -> re_corpus::ImportReceiptV2 {
    parse_receipt_v2_bytes(&receipt_v2_bytes(), &manifest(), &historical_receipt())
        .expect("accepted v2 receipt validates")
}

fn mapped_catalog_import_value(catalog: &Value, id: &str) -> Value {
    let record = catalog_record_value(catalog, id);
    let methods = record["methods"]["current"]
        .as_array()
        .expect("catalog current method array")
        .iter()
        .map(|method| {
            json!({
                "method_identity": method["symbol_key"].clone(),
                "evidence_status": method["evidence_status"].clone(),
                "source_status": method["source_status"].clone()
            })
        })
        .collect::<Vec<_>>();
    json!({
        "id": id,
        "authority_id": record["authority_id"].clone(),
        "methods": methods,
        "original_path": record["original_path"].clone(),
        "imported_path": record["proposed_path"].clone(),
        "original_size": record["original_size"].clone(),
        "imported_size": record["original_size"].clone(),
        "original_sha256": record["original_sha256"].clone(),
        "imported_sha256": record["original_sha256"].clone(),
        "exact_byte_comparison": "PASS",
        "implementation_scope": record["scope"].clone(),
        "source_completeness": record["completeness"].clone(),
        "unresolved_dependencies": record["dependencies"].clone(),
        "evidence_status": record["evidence_status"].clone(),
        "source_status": record["source_status"].clone(),
        "editorial_status": "NEEDS_EDITORIAL_REVIEW",
        "compilation_status": "NOT_RUN",
        "equivalence_status": "NOT_ESTABLISHED",
        "publication_status": "DISABLED",
        "runtime_status": "PARKED"
    })
}

fn v3_value() -> Value {
    let previous: Value =
        serde_json::from_slice(&receipt_v2_bytes()).expect("accepted v2 receipt JSON");
    let catalog = catalog_value();
    let mut imports = previous["imports"].as_array().expect("v2 imports").clone();
    imports.push(mapped_catalog_import_value(&catalog, "PTIMER"));
    imports.push(mapped_catalog_import_value(&catalog, "PTIMER_INSTANCE"));
    json!({
        "schema": "wuwa-reconstruction-imports-v3",
        "version": 3,
        "manifest_sha256": re_corpus::BASELINE_MANIFEST_SHA256,
        "previous_receipt_sha256": re_corpus::BASELINE_RECEIPT_V2_SHA256,
        "performance_source_sha256": re_corpus::PERFORMANCE_SOURCE_V1_SHA256,
        "imports": imports
    })
}

fn parse_v3_value(value: Value) -> Result<ImportReceiptV3, re_corpus::ValidationError> {
    parse_receipt_v3_bytes(
        &serde_json::to_vec(&value).expect("serialize v3 fixture"),
        &manifest(),
        &historical_receipt(),
        &historical_receipt_v2(),
        &performance_catalog(),
    )
}

fn v3_entry_mut<'a>(value: &'a mut Value, id: &str) -> &'a mut Value {
    value["imports"]
        .as_array_mut()
        .expect("v3 imports")
        .iter_mut()
        .find(|entry| entry["id"] == id)
        .expect("v3 entry")
}

fn write_fixture_v3(
    fixture: &FixtureRoot,
    v3: Option<&Value>,
    catalog: Option<&[u8]>,
    v2_bytes: &[u8],
    v1_bytes: &[u8],
) {
    let metadata = fixture.path.join("metadata");
    fs::create_dir_all(&metadata).expect("create fixture metadata");
    fs::write(metadata.join("migration.json"), manifest_bytes()).expect("write fixture manifest");
    fs::write(metadata.join("imports.v1.json"), v1_bytes).expect("write fixture v1");
    fs::write(metadata.join("imports.v2.json"), v2_bytes).expect("write fixture v2");
    if let Some(bytes) = catalog {
        fs::write(metadata.join("performance-source.v1.json"), bytes)
            .expect("write fixture catalog");
    }
    if let Some(value) = v3 {
        fs::write(
            metadata.join("imports.v3.json"),
            serde_json::to_vec(value).expect("serialize fixture v3"),
        )
        .expect("write fixture v3");
    }
}

fn fixture_v3_active() -> (FixtureRoot, ActiveReceipt) {
    let fixture = FixtureRoot::new();
    let v3 = v3_value();
    write_fixture_v3(
        &fixture,
        Some(&v3),
        Some(&catalog_bytes()),
        &receipt_v2_bytes(),
        &receipt_bytes(),
    );
    let active = load_active_receipt(&fixture.path)
        .expect("fixture v3 active receipt loads")
        .expect("fixture v3 active receipt exists");
    (fixture, active)
}

fn scheduling_catalog_bytes() -> Vec<u8> {
    fs::read(curated_root().join(SCHEDULING_SOURCE_REL)).expect("read scheduling source catalog")
}

fn scheduling_catalog_value() -> Value {
    serde_json::from_slice(&scheduling_catalog_bytes()).expect("scheduling catalog is JSON")
}

fn scheduling_catalog() -> SchedulingSourceCatalog {
    parse_scheduling_source_bytes(
        &scheduling_catalog_bytes(),
        &manifest(),
        &performance_catalog(),
    )
    .expect("scheduling catalog validates")
}

fn scheduling_record_value<'a>(catalog: &'a Value, id: &str) -> &'a Value {
    catalog["records"]
        .as_array()
        .expect("scheduling records")
        .iter()
        .find(|wrapper| wrapper["record"]["id"] == id)
        .expect("scheduling record")
}

fn scheduling_record_mut<'a>(catalog: &'a mut Value, id: &str) -> &'a mut Value {
    catalog["records"]
        .as_array_mut()
        .expect("scheduling records")
        .iter_mut()
        .find(|wrapper| wrapper["record"]["id"] == id)
        .expect("scheduling record")
}

fn scheduling_method_mut<'a>(catalog: &'a mut Value, id: &str, method_def: &str) -> &'a mut Value {
    scheduling_record_mut(catalog, id)["record"]["methods"]["current"]
        .as_array_mut()
        .expect("scheduling methods")
        .iter_mut()
        .find(|method| method["methodDef"] == method_def)
        .expect("scheduling method")
}

fn parse_scheduling_value(
    value: Value,
) -> Result<SchedulingSourceCatalog, re_corpus::ValidationError> {
    parse_scheduling_source_bytes(
        &serde_json::to_vec(&value).expect("serialize scheduling fixture"),
        &manifest(),
        &performance_catalog(),
    )
}

fn mapped_scheduling_import_value(catalog: &Value, id: &str) -> Value {
    let record = &scheduling_record_value(catalog, id)["record"];
    let methods = record["methods"]["current"]
        .as_array()
        .expect("scheduling current methods")
        .iter()
        .map(|method| {
            json!({
                "method_identity": method["symbol_key"].clone(),
                "evidence_status": method["evidence_status"].clone(),
                "source_status": method["source_status"].clone()
            })
        })
        .collect::<Vec<_>>();
    json!({
        "id": id,
        "authority_id": record["authority_id"].clone(),
        "methods": methods,
        "original_path": record["original_path"].clone(),
        "imported_path": record["proposed_path"].clone(),
        "original_size": record["original_size"].clone(),
        "imported_size": record["original_size"].clone(),
        "original_sha256": record["original_sha256"].clone(),
        "imported_sha256": record["original_sha256"].clone(),
        "exact_byte_comparison": "PASS",
        "implementation_scope": record["scope"].clone(),
        "source_completeness": record["completeness"].clone(),
        "unresolved_dependencies": record["dependencies"].clone(),
        "evidence_status": record["evidence_status"].clone(),
        "source_status": record["source_status"].clone(),
        "editorial_status": "NEEDS_EDITORIAL_REVIEW",
        "compilation_status": "NOT_RUN",
        "equivalence_status": "NOT_ESTABLISHED",
        "publication_status": "DISABLED",
        "runtime_status": "PARKED"
    })
}

fn receipt_v3_bytes() -> Vec<u8> {
    fs::read(curated_root().join(RECEIPT_V3_REL)).expect("read accepted v3 receipt")
}

fn historical_receipt_v3() -> ImportReceiptV3 {
    parse_receipt_v3_bytes(
        &receipt_v3_bytes(),
        &manifest(),
        &historical_receipt(),
        &historical_receipt_v2(),
        &performance_catalog(),
    )
    .expect("accepted v3 receipt validates")
}

fn v4_value() -> Value {
    let previous: Value =
        serde_json::from_slice(&receipt_v3_bytes()).expect("accepted v3 receipt JSON");
    let scheduling = scheduling_catalog_value();
    let mut imports = previous["imports"].as_array().expect("v3 imports").clone();
    imports.push(mapped_scheduling_import_value(&scheduling, "PSCHED_GRAPH"));
    imports.push(mapped_scheduling_import_value(&scheduling, "PSCHED_UTILS"));
    imports.push(mapped_scheduling_import_value(&scheduling, "PSCHED_HEAP"));
    json!({
        "schema": "wuwa-reconstruction-imports-v4",
        "version": 4,
        "manifest_sha256": re_corpus::BASELINE_MANIFEST_SHA256,
        "previous_receipt_sha256": re_corpus::BASELINE_RECEIPT_V3_SHA256,
        "performance_source_sha256": re_corpus::PERFORMANCE_SOURCE_V1_SHA256,
        "scheduling_source_sha256": re_corpus::SCHEDULING_SOURCE_V1_SHA256,
        "imports": imports
    })
}

fn parse_v4_value(value: Value) -> Result<ImportReceiptV4, re_corpus::ValidationError> {
    parse_receipt_v4_bytes(
        &serde_json::to_vec(&value).expect("serialize v4 fixture"),
        &manifest(),
        &historical_receipt(),
        &historical_receipt_v2(),
        &historical_receipt_v3(),
        &performance_catalog(),
        &scheduling_catalog(),
    )
}

fn v4_entry_mut<'a>(value: &'a mut Value, id: &str) -> &'a mut Value {
    value["imports"]
        .as_array_mut()
        .expect("v4 imports")
        .iter_mut()
        .find(|entry| entry["id"] == id)
        .expect("v4 entry")
}

fn write_fixture_v4(
    fixture: &FixtureRoot,
    v4: Option<&Value>,
    scheduling: Option<&[u8]>,
    performance: &[u8],
    v3_bytes: &[u8],
    v2_bytes: &[u8],
    v1_bytes: &[u8],
) {
    let metadata = fixture.path.join("metadata");
    fs::create_dir_all(&metadata).expect("create fixture metadata");
    fs::write(metadata.join("migration.json"), manifest_bytes()).expect("write fixture manifest");
    fs::write(metadata.join("imports.v1.json"), v1_bytes).expect("write fixture v1");
    fs::write(metadata.join("imports.v2.json"), v2_bytes).expect("write fixture v2");
    fs::write(metadata.join("imports.v3.json"), v3_bytes).expect("write fixture v3");
    fs::write(metadata.join("performance-source.v1.json"), performance)
        .expect("write fixture performance catalog");
    if let Some(bytes) = scheduling {
        fs::write(metadata.join("scheduling-source.v1.json"), bytes)
            .expect("write fixture scheduling catalog");
    }
    if let Some(value) = v4 {
        fs::write(
            metadata.join("imports.v4.json"),
            serde_json::to_vec(value).expect("serialize fixture v4"),
        )
        .expect("write fixture v4");
    }
}

fn fixture_v4_active() -> (FixtureRoot, ActiveReceipt) {
    let fixture = FixtureRoot::new();
    let v4 = v4_value();
    write_fixture_v4(
        &fixture,
        Some(&v4),
        Some(&scheduling_catalog_bytes()),
        &catalog_bytes(),
        &receipt_v3_bytes(),
        &receipt_v2_bytes(),
        &receipt_bytes(),
    );
    let active = load_active_receipt(&fixture.path)
        .expect("fixture v4 active receipt loads")
        .expect("fixture v4 active receipt exists");
    (fixture, active)
}

fn task_catalog_bytes() -> Vec<u8> {
    fs::read(curated_root().join(TASK_EXECUTION_SOURCE_REL)).expect("read task execution catalog")
}

fn task_catalog_value() -> Value {
    serde_json::from_slice(&task_catalog_bytes()).expect("task execution catalog is JSON")
}

fn task_catalog() -> TaskExecutionSourceCatalog {
    parse_task_execution_source_bytes(
        &task_catalog_bytes(),
        &manifest(),
        &performance_catalog(),
        &scheduling_catalog(),
    )
    .expect("task execution catalog validates")
}

fn task_record_value<'a>(catalog: &'a Value, id: &str) -> &'a Value {
    catalog["records"]
        .as_array()
        .expect("task records")
        .iter()
        .find(|wrapper| wrapper["record"]["id"] == id)
        .expect("task record")
}

fn task_record_mut<'a>(catalog: &'a mut Value, id: &str) -> &'a mut Value {
    catalog["records"]
        .as_array_mut()
        .expect("task records")
        .iter_mut()
        .find(|wrapper| wrapper["record"]["id"] == id)
        .expect("task record")
}

fn task_method_mut<'a>(catalog: &'a mut Value, id: &str, method_def: &str) -> &'a mut Value {
    task_record_mut(catalog, id)["record"]["methods"]["current"]
        .as_array_mut()
        .expect("task methods")
        .iter_mut()
        .find(|method| method["methodDef"] == method_def)
        .expect("task method")
}

fn parse_task_value(
    value: Value,
) -> Result<TaskExecutionSourceCatalog, re_corpus::ValidationError> {
    parse_task_execution_source_bytes(
        &serde_json::to_vec(&value).expect("serialize task fixture"),
        &manifest(),
        &performance_catalog(),
        &scheduling_catalog(),
    )
}

fn mapped_task_import_value(catalog: &Value, id: &str) -> Value {
    let record = &task_record_value(catalog, id)["record"];
    let methods = record["methods"]["current"]
        .as_array()
        .expect("task current methods")
        .iter()
        .map(|method| {
            json!({
                "method_identity": method["symbol_key"].clone(),
                "evidence_status": method["evidence_status"].clone(),
                "source_status": method["source_status"].clone()
            })
        })
        .collect::<Vec<_>>();
    json!({
        "id": id,
        "authority_id": record["authority_id"].clone(),
        "methods": methods,
        "original_path": record["original_path"].clone(),
        "imported_path": record["proposed_path"].clone(),
        "original_size": record["original_size"].clone(),
        "imported_size": record["original_size"].clone(),
        "original_sha256": record["original_sha256"].clone(),
        "imported_sha256": record["original_sha256"].clone(),
        "exact_byte_comparison": "PASS",
        "implementation_scope": record["scope"].clone(),
        "source_completeness": record["completeness"].clone(),
        "unresolved_dependencies": record["dependencies"].clone(),
        "evidence_status": record["evidence_status"].clone(),
        "source_status": record["source_status"].clone(),
        "editorial_status": "NEEDS_EDITORIAL_REVIEW",
        "compilation_status": "NOT_RUN",
        "equivalence_status": "NOT_ESTABLISHED",
        "publication_status": "DISABLED",
        "runtime_status": "PARKED"
    })
}

fn receipt_v4_bytes() -> Vec<u8> {
    fs::read(curated_root().join(RECEIPT_V4_REL)).expect("read accepted v4 receipt")
}

fn historical_receipt_v4() -> ImportReceiptV4 {
    parse_receipt_v4_bytes(
        &receipt_v4_bytes(),
        &manifest(),
        &historical_receipt(),
        &historical_receipt_v2(),
        &historical_receipt_v3(),
        &performance_catalog(),
        &scheduling_catalog(),
    )
    .expect("accepted v4 receipt validates")
}

fn v5_value() -> Value {
    let previous: Value =
        serde_json::from_slice(&receipt_v4_bytes()).expect("accepted v4 receipt JSON");
    let task = task_catalog_value();
    let mut imports = previous["imports"].as_array().expect("v4 imports").clone();
    imports.push(mapped_task_import_value(&task, "PEXEC_SYSTEM"));
    imports.push(mapped_task_import_value(&task, "PEXEC_BASE"));
    imports.push(mapped_task_import_value(&task, "PEXEC_QUEUE"));
    json!({
        "schema": "wuwa-reconstruction-imports-v5",
        "version": 5,
        "manifest_sha256": re_corpus::BASELINE_MANIFEST_SHA256,
        "previous_receipt_sha256": re_corpus::BASELINE_RECEIPT_V4_SHA256,
        "performance_source_sha256": re_corpus::PERFORMANCE_SOURCE_V1_SHA256,
        "scheduling_source_sha256": re_corpus::SCHEDULING_SOURCE_V1_SHA256,
        "task_execution_source_sha256": re_corpus::TASK_EXECUTION_SOURCE_V1_SHA256,
        "imports": imports
    })
}

fn parse_v5_value(value: Value) -> Result<ImportReceiptV5, re_corpus::ValidationError> {
    parse_receipt_v5_bytes(
        &serde_json::to_vec(&value).expect("serialize v5 fixture"),
        &manifest(),
        &historical_receipt(),
        &historical_receipt_v2(),
        &historical_receipt_v3(),
        &historical_receipt_v4(),
        &performance_catalog(),
        &scheduling_catalog(),
        &task_catalog(),
    )
}

fn v5_entry_mut<'a>(value: &'a mut Value, id: &str) -> &'a mut Value {
    value["imports"]
        .as_array_mut()
        .expect("v5 imports")
        .iter_mut()
        .find(|entry| entry["id"] == id)
        .expect("v5 entry")
}

fn write_fixture_v5(
    fixture: &FixtureRoot,
    v5: Option<&Value>,
    task: Option<&[u8]>,
    scheduling: &[u8],
    performance: &[u8],
    v4_bytes: &[u8],
    v3_bytes: &[u8],
    v2_bytes: &[u8],
    v1_bytes: &[u8],
) {
    let metadata = fixture.path.join("metadata");
    fs::create_dir_all(&metadata).expect("create fixture metadata");
    fs::write(metadata.join("migration.json"), manifest_bytes()).expect("write fixture manifest");
    fs::write(metadata.join("imports.v1.json"), v1_bytes).expect("write fixture v1");
    fs::write(metadata.join("imports.v2.json"), v2_bytes).expect("write fixture v2");
    fs::write(metadata.join("imports.v3.json"), v3_bytes).expect("write fixture v3");
    fs::write(metadata.join("imports.v4.json"), v4_bytes).expect("write fixture v4");
    fs::write(metadata.join("performance-source.v1.json"), performance)
        .expect("write fixture performance catalog");
    fs::write(metadata.join("scheduling-source.v1.json"), scheduling)
        .expect("write fixture scheduling catalog");
    if let Some(bytes) = task {
        fs::write(metadata.join("task-execution-source.v1.json"), bytes)
            .expect("write fixture task catalog");
    }
    if let Some(value) = v5 {
        fs::write(
            metadata.join("imports.v5.json"),
            serde_json::to_vec(value).expect("serialize fixture v5"),
        )
        .expect("write fixture v5");
    }
}

fn fixture_v5_active() -> (FixtureRoot, ActiveReceipt) {
    let fixture = FixtureRoot::new();
    let v5 = v5_value();
    write_fixture_v5(
        &fixture,
        Some(&v5),
        Some(&task_catalog_bytes()),
        &scheduling_catalog_bytes(),
        &catalog_bytes(),
        &receipt_v4_bytes(),
        &receipt_v3_bytes(),
        &receipt_v2_bytes(),
        &receipt_bytes(),
    );
    let active = load_active_receipt(&fixture.path)
        .expect("fixture v5 active receipt loads")
        .expect("fixture v5 active receipt exists");
    (fixture, active)
}

#[test]
fn positive_inventory_status_and_planned_paths_are_not_imports() {
    let (exit, output) = run(&["status", "--json"]);
    assert_eq!(exit, 0);
    assert_eq!(output["ok"], true);
    assert_eq!(output["counts"]["historical_entries"], 18);
    let active = load_active_receipt(&curated_root()).expect("actual active receipt loads");
    match active.as_ref().map(|receipt| receipt.version) {
        None | Some(1) | Some(2) => {
            assert_eq!(output["counts"]["current_entries"], 18);
            assert_eq!(output["counts"]["unique_qualified_symbols"], 14);
            assert_eq!(output["counts"]["planned"], 6);
        }
        Some(3) => {
            assert_eq!(output["counts"]["current_entries"], 24);
            assert_eq!(output["counts"]["unique_qualified_symbols"], 20);
            assert_eq!(output["counts"]["planned"], 8);
        }
        Some(4) => {
            assert_eq!(output["counts"]["current_entries"], 57);
            assert_eq!(output["counts"]["unique_qualified_symbols"], 53);
            assert_eq!(output["counts"]["planned"], 11);
        }
        Some(5) => {
            assert_eq!(output["counts"]["current_entries"], 108);
            assert_eq!(output["counts"]["unique_qualified_symbols"], 104);
            assert_eq!(output["counts"]["planned"], 14);
        }
        Some(6) => {
            assert_eq!(output["active_ledger"]["path"], INDEX_REL);
            assert_eq!(output["active_ledger"]["version"], 6);
            assert!(output["counts"]["imported"].as_u64().expect("imported") >= 13);
            assert!(
                output["counts"]["imported_qualified_methods"]
                    .as_u64()
                    .expect("methods")
                    >= 102
            );
        }
        Some(version) => panic!("unexpected active receipt version {version}"),
    }
    let receipt_path = curated_root().join(RECEIPT_REL);
    match fs::symlink_metadata(&receipt_path) {
        Ok(metadata) => {
            assert!(metadata.is_file());
            assert!(!metadata.file_type().is_symlink());
            let receipt = parse_receipt_bytes(
                &fs::read(&receipt_path).expect("read actual import receipt"),
                &manifest(),
            )
            .expect("actual import receipt validates");
            assert_eq!(receipt.imports.len(), 3);
            let historical_ids: BTreeSet<&str> = receipt
                .imports
                .iter()
                .map(|entry| entry.id.as_str())
                .collect();
            assert_eq!(historical_ids, BTreeSet::from(["E13", "E17", "E18"]));
            let expected_active_ids: BTreeSet<&str> = match active.as_ref().expect("active receipt")
            {
                active if active.version == 1 => {
                    let active_ids: BTreeSet<&str> =
                        active.files.iter().map(|file| file.id.as_str()).collect();
                    assert_eq!(active_ids, BTreeSet::from(["E13", "E17", "E18"]));
                    assert_eq!(active.files.len(), 3);
                    assert_eq!(
                        active
                            .files
                            .iter()
                            .map(|file| file.methods.len())
                            .sum::<usize>(),
                        3
                    );
                    assert_eq!(output["counts"]["imported"], 3);
                    assert_eq!(output["counts"]["imported_file_count"], 3);
                    assert_eq!(output["counts"]["imported_qualified_methods"], 3);
                    assert_eq!(output["counts"]["missing_imports"], 3);
                    assert_eq!(output["active_ledger"]["path"], RECEIPT_REL);
                    assert_eq!(output["active_ledger"]["version"], 1);
                    BTreeSet::from(["E13", "E17", "E18"])
                }
                active if active.version == 2 => {
                    let active_ids: BTreeSet<&str> =
                        active.files.iter().map(|file| file.id.as_str()).collect();
                    assert_eq!(active_ids, BTreeSet::from(["D", "E13", "E17", "E18", "T"]));
                    assert_eq!(active.files.len(), 5);
                    assert_eq!(
                        active
                            .files
                            .iter()
                            .map(|file| file.methods.len())
                            .sum::<usize>(),
                        12
                    );
                    assert_eq!(output["counts"]["imported"], 5);
                    assert_eq!(output["counts"]["imported_file_count"], 5);
                    assert_eq!(output["counts"]["imported_qualified_methods"], 12);
                    assert_eq!(output["counts"]["missing_imports"], 1);
                    assert_eq!(output["active_ledger"]["path"], RECEIPT_V2_REL);
                    assert_eq!(output["active_ledger"]["version"], 2);
                    BTreeSet::from(["D", "E13", "E17", "E18", "T"])
                }
                active if active.version == 3 => {
                    let active_ids: BTreeSet<&str> =
                        active.files.iter().map(|file| file.id.as_str()).collect();
                    assert_eq!(
                        active_ids,
                        BTreeSet::from([
                            "D",
                            "E13",
                            "E17",
                            "E18",
                            "PTIMER",
                            "PTIMER_INSTANCE",
                            "T",
                        ])
                    );
                    assert_eq!(active.files.len(), 7);
                    assert_eq!(
                        active
                            .files
                            .iter()
                            .map(|file| file.methods.len())
                            .sum::<usize>(),
                        18
                    );
                    assert_eq!(output["counts"]["historical_entries"], 18);
                    assert_eq!(output["counts"]["current_entries"], 24);
                    assert_eq!(output["counts"]["unique_qualified_symbols"], 20);
                    assert_eq!(output["counts"]["planned"], 8);
                    assert_eq!(output["counts"]["imported"], 7);
                    assert_eq!(output["counts"]["imported_file_count"], 7);
                    assert_eq!(output["counts"]["imported_qualified_methods"], 18);
                    assert_eq!(output["counts"]["missing_imports"], 1);
                    assert_eq!(output["active_ledger"]["path"], RECEIPT_V3_REL);
                    assert_eq!(output["active_ledger"]["version"], 3);
                    assert_eq!(output["records"].as_array().expect("records").len(), 13);
                    BTreeSet::from(["D", "E13", "E17", "E18", "PTIMER", "PTIMER_INSTANCE", "T"])
                }
                active if active.version == 4 => {
                    let active_ids: BTreeSet<&str> =
                        active.files.iter().map(|file| file.id.as_str()).collect();
                    assert_eq!(
                        active_ids,
                        BTreeSet::from([
                            "D",
                            "E13",
                            "E17",
                            "E18",
                            "PSCHED_GRAPH",
                            "PSCHED_HEAP",
                            "PSCHED_UTILS",
                            "PTIMER",
                            "PTIMER_INSTANCE",
                            "T",
                        ])
                    );
                    assert_eq!(active.files.len(), 10);
                    assert_eq!(
                        active
                            .files
                            .iter()
                            .map(|file| file.methods.len())
                            .sum::<usize>(),
                        51
                    );
                    assert_eq!(output["counts"]["historical_entries"], 18);
                    assert_eq!(output["counts"]["current_entries"], 57);
                    assert_eq!(output["counts"]["unique_qualified_symbols"], 53);
                    assert_eq!(output["counts"]["planned"], 11);
                    assert_eq!(output["counts"]["imported"], 10);
                    assert_eq!(output["counts"]["imported_file_count"], 10);
                    assert_eq!(output["counts"]["imported_qualified_methods"], 51);
                    assert_eq!(output["counts"]["missing_imports"], 1);
                    assert_eq!(output["active_ledger"]["path"], RECEIPT_V4_REL);
                    assert_eq!(output["active_ledger"]["version"], 4);
                    assert_eq!(output["records"].as_array().expect("records").len(), 16);
                    BTreeSet::from([
                        "D",
                        "E13",
                        "E17",
                        "E18",
                        "PSCHED_GRAPH",
                        "PSCHED_HEAP",
                        "PSCHED_UTILS",
                        "PTIMER",
                        "PTIMER_INSTANCE",
                        "T",
                    ])
                }
                active if active.version == 5 => {
                    let active_ids: BTreeSet<&str> =
                        active.files.iter().map(|file| file.id.as_str()).collect();
                    assert_eq!(
                        active_ids,
                        BTreeSet::from([
                            "D",
                            "E13",
                            "E17",
                            "E18",
                            "PEXEC_BASE",
                            "PEXEC_QUEUE",
                            "PEXEC_SYSTEM",
                            "PSCHED_GRAPH",
                            "PSCHED_HEAP",
                            "PSCHED_UTILS",
                            "PTIMER",
                            "PTIMER_INSTANCE",
                            "T",
                        ])
                    );
                    assert_eq!(active.files.len(), 13);
                    assert_eq!(
                        active
                            .files
                            .iter()
                            .map(|file| file.methods.len())
                            .sum::<usize>(),
                        102
                    );
                    assert_eq!(output["counts"]["historical_entries"], 18);
                    assert_eq!(output["counts"]["current_entries"], 108);
                    assert_eq!(output["counts"]["unique_qualified_symbols"], 104);
                    assert_eq!(output["counts"]["planned"], 14);
                    assert_eq!(output["counts"]["imported"], 13);
                    assert_eq!(output["counts"]["imported_file_count"], 13);
                    assert_eq!(output["counts"]["imported_qualified_methods"], 102);
                    assert_eq!(output["counts"]["missing_imports"], 1);
                    assert_eq!(output["active_ledger"]["path"], RECEIPT_V5_REL);
                    assert_eq!(output["active_ledger"]["version"], 5);
                    assert_eq!(output["records"].as_array().expect("records").len(), 19);
                    BTreeSet::from([
                        "D",
                        "E13",
                        "E17",
                        "E18",
                        "PEXEC_BASE",
                        "PEXEC_QUEUE",
                        "PEXEC_SYSTEM",
                        "PSCHED_GRAPH",
                        "PSCHED_HEAP",
                        "PSCHED_UTILS",
                        "PTIMER",
                        "PTIMER_INSTANCE",
                        "T",
                    ])
                }
                active if active.version == 6 => {
                    let active_ids: BTreeSet<&str> =
                        active.files.iter().map(|file| file.id.as_str()).collect();
                    let base_ids = BTreeSet::from([
                        "D",
                        "E13",
                        "E17",
                        "E18",
                        "PEXEC_BASE",
                        "PEXEC_QUEUE",
                        "PEXEC_SYSTEM",
                        "PSCHED_GRAPH",
                        "PSCHED_HEAP",
                        "PSCHED_UTILS",
                        "PTIMER",
                        "PTIMER_INSTANCE",
                        "T",
                    ]);
                    assert!(base_ids.iter().all(|id| active_ids.contains(id)));
                    assert_eq!(active.files.len(), active_ids.len());
                    assert_eq!(
                        output["counts"]["imported"].as_u64().expect("imported"),
                        active.files.len() as u64
                    );
                    assert_eq!(
                        output["counts"]["imported_file_count"]
                            .as_u64()
                            .expect("files"),
                        active.files.len() as u64
                    );
                    assert_eq!(
                        output["counts"]["imported_qualified_methods"]
                            .as_u64()
                            .expect("methods"),
                        active
                            .files
                            .iter()
                            .map(|file| file.methods.len() as u64)
                            .sum::<u64>()
                    );
                    assert_eq!(output["active_ledger"]["path"], INDEX_REL);
                    assert_eq!(output["active_ledger"]["version"], 6);
                    active_ids
                }
                active => panic!("unexpected active receipt version {}", active.version),
            };
            for record in output["records"].as_array().expect("records array") {
                let id = record["id"].as_str().expect("record id");
                if record["proposed_path"].is_string() {
                    let expected_status = if expected_active_ids.contains(id) {
                        "IMPORTED"
                    } else {
                        "NOT_IMPORTED"
                    };
                    assert_eq!(record["import_status"], expected_status);
                } else {
                    assert_eq!(record["import_status"], "NOT_APPLICABLE");
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            assert!(active.is_none());
            assert_eq!(output["counts"]["imported"], 0);
            assert_eq!(output["counts"]["imported_file_count"], 0);
            assert_eq!(output["counts"]["imported_qualified_methods"], 0);
            assert_eq!(output["counts"]["missing_imports"], 6);
            assert_eq!(output["active_ledger"], Value::Null);
            for record in output["records"].as_array().expect("records array") {
                if record["proposed_path"].is_string() {
                    assert_eq!(record["import_status"], "NOT_IMPORTED");
                } else {
                    assert_eq!(record["import_status"], "NOT_APPLICABLE");
                }
            }
        }
        Err(error) => panic!("receipt metadata inspection failed: {error}"),
    }
    assert!(
        output["findings"]
            .as_array()
            .expect("findings array")
            .is_empty()
    );
}

#[test]
fn positive_lookup_keeps_e17_and_h_current_historical_context() {
    let active = load_active_receipt(&curated_root())
        .expect("actual active receipt loads")
        .expect("actual active receipt exists");
    let (exit, output) = run(&["lookup", MANAGED_AUTHORITY_ID, "0x06000434", "--json"]);
    assert_eq!(exit, 0);
    match active.version {
        1 => {
            assert_eq!(output["active_ledger"]["path"], RECEIPT_REL);
            assert_eq!(output["active_ledger"]["version"], 1);
            assert_eq!(output["counts"]["imported"], 3);
            assert_eq!(output["counts"]["imported_file_count"], 3);
            assert_eq!(output["counts"]["imported_qualified_methods"], 3);
        }
        2 => {
            assert_eq!(output["active_ledger"]["path"], RECEIPT_V2_REL);
            assert_eq!(output["active_ledger"]["version"], 2);
            assert_eq!(output["counts"]["imported"], 5);
            assert_eq!(output["counts"]["imported_file_count"], 5);
            assert_eq!(output["counts"]["imported_qualified_methods"], 12);
        }
        3 => {
            assert_eq!(output["active_ledger"]["path"], RECEIPT_V3_REL);
            assert_eq!(output["active_ledger"]["version"], 3);
            assert_eq!(output["counts"]["imported"], 7);
            assert_eq!(output["counts"]["imported_file_count"], 7);
            assert_eq!(output["counts"]["imported_qualified_methods"], 18);
        }
        4 => {
            assert_eq!(output["active_ledger"]["path"], RECEIPT_V4_REL);
            assert_eq!(output["active_ledger"]["version"], 4);
            assert_eq!(output["counts"]["imported"], 10);
            assert_eq!(output["counts"]["imported_file_count"], 10);
            assert_eq!(output["counts"]["imported_qualified_methods"], 51);
        }
        5 => {
            assert_eq!(output["active_ledger"]["path"], RECEIPT_V5_REL);
            assert_eq!(output["active_ledger"]["version"], 5);
            assert_eq!(output["counts"]["imported"], 13);
            assert_eq!(output["counts"]["imported_file_count"], 13);
            assert_eq!(output["counts"]["imported_qualified_methods"], 102);
        }
        6 => {
            assert_eq!(output["active_ledger"]["path"], INDEX_REL);
            assert_eq!(output["active_ledger"]["version"], 6);
            assert_eq!(
                output["counts"]["imported"].as_u64().expect("imported"),
                active.files.len() as u64
            );
            assert_eq!(
                output["counts"]["imported_file_count"]
                    .as_u64()
                    .expect("files"),
                active.files.len() as u64
            );
            assert_eq!(
                output["counts"]["imported_qualified_methods"]
                    .as_u64()
                    .expect("methods"),
                active
                    .files
                    .iter()
                    .map(|file| file.methods.len() as u64)
                    .sum::<u64>()
            );
        }
        version => panic!("unexpected active receipt version {version}"),
    }
    let matches = output["matches"].as_array().expect("matches array");
    let ids: Vec<&str> = matches
        .iter()
        .map(|record| record["id"].as_str().expect("record id"))
        .collect();
    assert_eq!(ids, vec!["E17", "H"]);
    let historical = matches
        .iter()
        .find(|record| record["id"] == "H")
        .expect("historical H record");
    assert_eq!(historical["current"].as_array().expect("current").len(), 1);
    assert_eq!(
        historical["historical"]
            .as_array()
            .expect("historical")
            .len(),
        1
    );
    assert_eq!(historical["historical"][0]["era"], "HISTORICAL");
    assert_eq!(
        output["ambiguous"]
            .as_array()
            .expect("ambiguity list")
            .len(),
        1
    );
}

#[test]
fn positive_verify_without_root_is_incomplete_and_external_refs_not_checked() {
    let active = load_active_receipt(&curated_root())
        .expect("actual active receipt loads")
        .expect("actual active receipt exists");
    let (exit, output) = run(&["verify", "--json"]);
    assert_eq!(exit, 1);
    assert_eq!(output["ok"], false);
    match active.version {
        1 => {
            assert_eq!(output["active_ledger"]["path"], RECEIPT_REL);
            assert_eq!(output["active_ledger"]["version"], 1);
            assert_eq!(output["counts"]["imported"], 3);
            assert_eq!(output["counts"]["imported_file_count"], 3);
            assert_eq!(output["counts"]["imported_qualified_methods"], 3);
        }
        2 => {
            assert_eq!(output["active_ledger"]["path"], RECEIPT_V2_REL);
            assert_eq!(output["active_ledger"]["version"], 2);
            assert_eq!(output["counts"]["imported"], 5);
            assert_eq!(output["counts"]["imported_file_count"], 5);
            assert_eq!(output["counts"]["imported_qualified_methods"], 12);
        }
        3 => {
            assert_eq!(output["active_ledger"]["path"], RECEIPT_V3_REL);
            assert_eq!(output["active_ledger"]["version"], 3);
            assert_eq!(output["counts"]["imported"], 7);
            assert_eq!(output["counts"]["imported_file_count"], 7);
            assert_eq!(output["counts"]["imported_qualified_methods"], 18);
        }
        4 => {
            assert_eq!(output["active_ledger"]["path"], RECEIPT_V4_REL);
            assert_eq!(output["active_ledger"]["version"], 4);
            assert_eq!(output["counts"]["imported"], 10);
            assert_eq!(output["counts"]["imported_file_count"], 10);
            assert_eq!(output["counts"]["imported_qualified_methods"], 51);
        }
        5 => {
            assert_eq!(output["active_ledger"]["path"], RECEIPT_V5_REL);
            assert_eq!(output["active_ledger"]["version"], 5);
            assert_eq!(output["counts"]["imported"], 13);
            assert_eq!(output["counts"]["imported_file_count"], 13);
            assert_eq!(output["counts"]["imported_qualified_methods"], 102);
        }
        6 => {
            assert_eq!(output["active_ledger"]["path"], INDEX_REL);
            assert_eq!(output["active_ledger"]["version"], 6);
            assert_eq!(
                output["counts"]["imported"].as_u64().expect("imported"),
                active.files.len() as u64
            );
            assert_eq!(
                output["counts"]["imported_file_count"]
                    .as_u64()
                    .expect("files"),
                active.files.len() as u64
            );
            assert_eq!(
                output["counts"]["imported_qualified_methods"]
                    .as_u64()
                    .expect("methods"),
                active
                    .files
                    .iter()
                    .map(|file| file.methods.len() as u64)
                    .sum::<u64>()
            );
        }
        version => panic!("unexpected active receipt version {version}"),
    }
    assert!(
        output["findings"]
            .as_array()
            .expect("findings")
            .iter()
            .any(|finding| finding["code"] == "REQUIRED_ORIGINALS_NOT_CHECKED")
    );
    let not_checked = output["not_checked"].as_array().expect("not_checked");
    assert!(not_checked.iter().any(|item| {
        item["kind"] == "ORIGINAL_SOURCE" && item["id"] == "E18" && item["status"] == "NOT_CHECKED"
    }));
    assert!(
        not_checked
            .iter()
            .any(|item| item["kind"] == "EVIDENCE_REFERENCE" && item["status"] == "NOT_CHECKED")
    );
}

#[test]
fn strict_json_rejects_nested_escaped_duplicates_nonfinite_and_trailing_data() {
    assert_eq!(
        parse_strict_value(br#"{"x":{"a":1,"\u0061":2}}"#)
            .unwrap_err()
            .code,
        "DUPLICATE_JSON_KEY"
    );
    assert_eq!(
        parse_strict_value(br#"{"x":NaN}"#).unwrap_err().code,
        "MALFORMED_JSON"
    );
    assert_eq!(
        parse_strict_value(br#"{"x":1} {"y":2}"#).unwrap_err().code,
        "MALFORMED_JSON"
    );
}

#[test]
fn structural_rejects_duplicate_ids_hashes_mvid_tokens_and_unknown_enums() {
    let mut value = manifest_value();
    let duplicate = value["records"][1]["id"].clone();
    value["records"][0]["id"] = duplicate;
    assert_eq!(parse_mutated(value).code, "DUPLICATE_RECORD_ID");

    let mut value = manifest_value();
    value["records"][0]["original_sha256"] = Value::String("A".repeat(64));
    assert_eq!(parse_mutated(value).code, "INVALID_SOURCE_SHA256");

    let mut value = manifest_value();
    value["records"][0]["authority_mvid"] =
        Value::String("2D6D37CB-76E5-4075-8220-3EECA41471E3".to_owned());
    assert_eq!(parse_mutated(value).code, "MANAGED_AUTHORITY_INCOMPLETE");

    let mut value = manifest_value();
    record_mut(&mut value, "E18")["methods"]["current"][0]["methodDef"] =
        Value::String("0x0600042e".to_owned());
    record_mut(&mut value, "E18")["methods"]["current"][0]["symbol_key"]["methodDef"] =
        Value::String("0x0600042e".to_owned());
    assert_eq!(parse_mutated(value).code, "INVALID_METHODDEF");

    let mut value = manifest_value();
    record_mut(&mut value, "E18")["methods"]["current"][0]["methodDef"] =
        Value::String("0x0600042F".to_owned());
    assert_eq!(parse_mutated(value).code, "METHOD_KEY_MISMATCH");

    let mut value = manifest_value();
    record_mut(&mut value, "E18")["evidence_status"] = Value::String("NOPE".to_owned());
    assert_eq!(parse_mutated(value).code, "SCHEMA_INVALID");
}

#[test]
fn structural_rejects_missing_authority_and_unsafe_paths() {
    let mut value = manifest_value();
    record_mut(&mut value, "E18")["authority_id"] = Value::Null;
    assert_eq!(parse_mutated(value).code, "SCHEMA_INVALID");

    for path in [
        "C:/outside.cs",
        "/absolute.cs",
        "src/../outside.cs",
        "src/CON.cs",
        "src/file. ",
        "src/bad|name.cs",
        "src/naïve.cs",
    ] {
        assert!(
            validate_relative_path(path).is_err(),
            "unsafe path accepted: {path}"
        );
    }
}

#[test]
fn structural_rejects_destination_casefold_collision_and_qualified_conflict() {
    let mut value = manifest_value();
    let first_destination = value["records"][0]["proposed_path"].clone();
    value["records"][1]["proposed_path"] = first_destination;
    assert_eq!(parse_mutated(value).code, "DESTINATION_CASEFOLD_COLLISION");

    let mut value = manifest_value();
    let token = "0x06018865";
    record_mut(&mut value, "T")["methods"]["current"][0]["methodDef"] =
        Value::String(token.to_owned());
    record_mut(&mut value, "T")["methods"]["current"][0]["symbol_key"]["methodDef"] =
        Value::String(token.to_owned());
    record_mut(&mut value, "T")["methods"]["historical"][0]["symbol_key"]["methodDef"] =
        Value::String(token.to_owned());
    assert_eq!(parse_mutated(value).code, "CONFLICTING_QUALIFIED_OVERLAP");

    let mut value = manifest_value();
    record_mut(&mut value, "G")["relationship"] = Value::String("unrelated text".to_owned());
    assert_eq!(parse_mutated(value).code, "CONFLICTING_QUALIFIED_OVERLAP");
}

#[test]
fn structural_source_only_promotion_requires_accepted_load01_proof() {
    let mut value = manifest_value();
    record_mut(&mut value, "T")["methods"]["current"][0]["evidence_status"] =
        Value::String("DIRECT_CIL_FULL".to_owned());
    assert_eq!(parse_mutated(value).code, "SOURCE_ONLY_PROMOTION_UNPROVEN");
}

#[test]
fn receipt_entries_preserve_identity_hashes_scope_and_completeness_for_approved_batch() {
    let manifest = manifest();
    for id in ["E18", "E17", "E13"] {
        let fixture = format!("fixture-{id}");
        let bytes = fixture.as_bytes();
        let imported_sha256 = re_corpus::sha256_hex(bytes);
        let value = import_value(&manifest, id, &imported_sha256, bytes.len() as u64);
        let receipt = parse_receipt_bytes(
            &serde_json::to_vec(&value).expect("receipt JSON"),
            &manifest,
        )
        .expect("approved receipt entry validates");
        let entry = &receipt.imports[0];
        let record = manifest
            .records
            .iter()
            .find(|record| record.id == id)
            .expect("approved record");
        assert_eq!(entry.method_identity, record.methods.current[0].symbol_key);
        assert_eq!(entry.original_path, record.original_path);
        assert_eq!(entry.original_size, record.original_size);
        assert_eq!(entry.original_sha256, record.original_sha256);
        assert_eq!(entry.imported_size, bytes.len() as u64);
        assert_eq!(entry.imported_sha256, imported_sha256);
        assert_eq!(entry.implementation_scope, record.scope);
        assert_eq!(entry.source_completeness, record.completeness);
    }
}

#[test]
fn actual_v1_receipt_parses_with_pinned_identity_and_hashes() {
    let bytes = receipt_bytes();
    assert_eq!(
        re_corpus::sha256_hex(&bytes),
        re_corpus::BASELINE_RECEIPT_V1_SHA256
    );
    let receipt = historical_receipt();
    let manifest = manifest_value();
    assert_eq!(receipt.imports.len(), 3);
    for entry in &receipt.imports {
        let record = manifest_record_value(&manifest, &entry.id);
        assert_eq!(entry.authority_id, record["authority_id"]);
        assert_eq!(entry.original_path, record["original_path"]);
        assert_eq!(entry.original_size, record["original_size"]);
        assert_eq!(entry.original_sha256, record["original_sha256"]);
        assert_eq!(entry.imported_path, record["proposed_path"]);
        assert_eq!(entry.imported_size, record["original_size"]);
        assert_eq!(entry.imported_sha256, record["original_sha256"]);
    }
}

#[test]
fn v2_accepts_exact_five_entries_and_maps_twelve_methods() {
    let parsed = parse_v2_value(v2_value()).expect("accepted v2 fixture");
    assert_eq!(parsed.imports.len(), 5);
    let mut files = 0;
    let mut methods = 0;
    for entry in &parsed.imports {
        files += 1;
        match entry {
            ImportEntryV2::Event(_) => methods += 1,
            ImportEntryV2::Mapped(entry) => {
                methods += entry.methods.len();
                assert!(matches!(entry.id.as_str(), "D" | "T"));
            }
        }
    }
    assert_eq!(files, 5);
    assert_eq!(methods, 12);
    let (_fixture, active) = fixture_active(Some(&v2_value()));
    assert_eq!(active.version, 2);
    assert_eq!(active.files.len(), 5);
    assert_eq!(
        active
            .files
            .iter()
            .map(|file| file.methods.len())
            .sum::<usize>(),
        12
    );
    assert_eq!(active.ledger_path, RECEIPT_V2_REL);
}

#[test]
fn v2_method_reordering_is_identity_based_and_statuses_remain_mixed() {
    let mut value = v2_value();
    v2_entry_mut(&mut value, "D")["methods"]
        .as_array_mut()
        .expect("D methods")
        .reverse();
    value["imports"].as_array_mut().expect("imports").reverse();
    let parsed = parse_v2_value(value).expect("reordered v2 fixture");
    let d = parsed
        .imports
        .iter()
        .find_map(|entry| match entry {
            ImportEntryV2::Mapped(entry) if entry.id == "D" => Some(entry),
            _ => None,
        })
        .expect("D mapped entry");
    assert_eq!(d.methods.len(), 8);
    assert_eq!(
        d.methods
            .iter()
            .filter(|method| matches!(method.evidence_status, re_corpus::EvidenceStatus::Unknown))
            .count(),
        6
    );
    assert_eq!(
        d.methods
            .iter()
            .filter(|method| matches!(method.source_status, re_corpus::SourceStatus::Unknown))
            .count(),
        6
    );
    assert!(matches!(
        d.evidence_status,
        re_corpus::EvidenceStatus::Unknown
    ));
    assert!(matches!(
        d.source_status,
        re_corpus::SourceStatus::SourceOnly
    ));
    let t = parsed
        .imports
        .iter()
        .find_map(|entry| match entry {
            ImportEntryV2::Mapped(entry) if entry.id == "T" => Some(entry),
            _ => None,
        })
        .expect("T mapped entry");
    assert_eq!(t.methods.len(), 1);
    assert!(matches!(
        t.evidence_status,
        re_corpus::EvidenceStatus::SourceOnly
    ));
    assert!(matches!(
        t.source_status,
        re_corpus::SourceStatus::SourceOnly
    ));
}

#[test]
fn v2_scope_completeness_and_dependencies_remain_record_specific() {
    let value = v2_value();
    let manifest = manifest_value();
    let d = manifest_record_value(&manifest, "D");
    let t = manifest_record_value(&manifest, "T");
    assert_ne!(d["scope"], t["scope"]);
    assert_ne!(d["completeness"], t["completeness"]);
    assert_ne!(d["dependencies"], t["dependencies"]);
    parse_v2_value(value).expect("record-specific v2 scope fixture");
}

#[test]
fn active_loader_has_v1_fallback_and_keeps_v1_fixture_coverage() {
    let (_fixture, active) = fixture_active(None);
    assert_eq!(active.version, 1);
    assert_eq!(active.ledger_path, RECEIPT_REL);
    assert_eq!(active.files.len(), 3);
    assert_eq!(
        active
            .files
            .iter()
            .map(|file| file.methods.len())
            .sum::<usize>(),
        3
    );
}

#[test]
fn active_loader_prefers_v2_without_combining_counts() {
    let (_fixture, active) = fixture_active(Some(&v2_value()));
    assert_eq!(active.version, 2);
    assert_eq!(active.files.len(), 5);
    assert_eq!(
        active
            .files
            .iter()
            .map(|file| file.methods.len())
            .sum::<usize>(),
        12
    );
    assert_eq!(active.files.iter().filter(|file| file.id == "D").count(), 1);
}

#[test]
fn active_loader_rejects_invalid_v2_without_v1_fallback() {
    let mut invalid = v2_value();
    invalid["version"] = json!(1);
    let fixture = FixtureRoot::new();
    write_fixture_receipts(&fixture, Some(&invalid), &receipt_bytes());
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V2_ROOT_INVALID"
    );
}

#[test]
fn active_loader_rejects_historical_hash_conflict_and_event_conflict() {
    let mut altered_history = receipt_bytes();
    altered_history.push(b'\n');
    let fixture = FixtureRoot::new();
    write_fixture_receipts(&fixture, Some(&v2_value()), &altered_history);
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V2_HISTORICAL_RECEIPT_HASH_MISMATCH"
    );

    let mut conflicting = v2_value();
    v2_entry_mut(&mut conflicting, "E18")["imported_sha256"] = Value::String("f".repeat(64));
    let fixture = FixtureRoot::new();
    write_fixture_receipts(&fixture, Some(&conflicting), &receipt_bytes());
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V2_EVENT_ENTRY_MISMATCH"
    );
}

#[test]
fn active_loader_rejects_v2_directory_instead_of_falling_back() {
    let fixture = FixtureRoot::new();
    write_fixture_receipts(&fixture, None, &receipt_bytes());
    fs::create_dir(fixture.path.join(RECEIPT_V2_REL)).expect("create v2 directory");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V2_RECEIPT_NOT_REGULAR"
    );
}

#[test]
fn active_loader_rejects_v2_without_historical_v1_and_allows_no_receipt() {
    let fixture = FixtureRoot::new();
    write_fixture_receipts(&fixture, Some(&v2_value()), &receipt_bytes());
    fs::remove_file(fixture.path.join(RECEIPT_REL)).expect("remove owned historical receipt");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V2_HISTORICAL_RECEIPT_MISSING"
    );

    let fixture = FixtureRoot::new();
    let metadata = fixture.path.join("metadata");
    fs::create_dir_all(&metadata).expect("create fixture metadata");
    fs::write(metadata.join("migration.json"), manifest_bytes()).expect("write fixture manifest");
    assert!(
        load_active_receipt(&fixture.path)
            .expect("empty fixture loads")
            .is_none()
    );
}

#[test]
fn v2_rejects_exact_id_shape_identity_path_and_duplicate_contracts() {
    let mut duplicate = v2_value();
    let first = v2_entry_mut(&mut duplicate, "D")["methods"][0].clone();
    duplicate["imports"]
        .as_array_mut()
        .expect("imports")
        .iter_mut()
        .find(|entry| entry["id"] == "D")
        .expect("D")["methods"]
        .as_array_mut()
        .expect("D methods")
        .push(first);
    assert_eq!(
        parse_v2_value(duplicate).unwrap_err().code,
        "V2_DUPLICATE_QUALIFIED_METHOD"
    );

    let mut missing = v2_value();
    v2_entry_mut(&mut missing, "D")["methods"]
        .as_array_mut()
        .expect("D methods")
        .remove(0);
    assert_eq!(
        parse_v2_value(missing).unwrap_err().code,
        "V2_METHOD_SET_MISMATCH"
    );

    let mut extra = v2_value();
    let mut fabricated = v2_entry_mut(&mut extra, "D")["methods"][0].clone();
    fabricated["method_identity"]["methodDef"] = json!("0x0601FFFF");
    v2_entry_mut(&mut extra, "D")["methods"]
        .as_array_mut()
        .expect("D methods")
        .push(fabricated);
    assert_eq!(
        parse_v2_value(extra).unwrap_err().code,
        "V2_METHOD_SET_MISMATCH"
    );

    let mut bad_type = v2_value();
    v2_entry_mut(&mut bad_type, "D")["methods"][0]["method_identity"]["typeDef"] =
        json!("0x02002EB3");
    assert_eq!(
        parse_v2_value(bad_type).unwrap_err().code,
        "V2_METHOD_SET_MISMATCH"
    );

    let mut overlap = v2_value();
    let d_identity = v2_entry_mut(&mut overlap, "D")["methods"][0]["method_identity"].clone();
    v2_entry_mut(&mut overlap, "T")["methods"][0]["method_identity"] = d_identity;
    assert_eq!(
        parse_v2_value(overlap).unwrap_err().code,
        "V2_DUPLICATE_QUALIFIED_METHOD"
    );

    let mut unsafe_destination = v2_value();
    v2_entry_mut(&mut unsafe_destination, "D")["imported_path"] = json!("../escape.cs");
    assert_eq!(
        parse_v2_value(unsafe_destination).unwrap_err().code,
        "V2_PATH_INVALID"
    );

    let mut collision = v2_value();
    let d_path = v2_entry_mut(&mut collision, "D")["imported_path"].clone();
    v2_entry_mut(&mut collision, "T")["imported_path"] =
        Value::String(d_path.as_str().expect("D path").to_ascii_uppercase());
    assert_eq!(
        parse_v2_value(collision).unwrap_err().code,
        "V2_DESTINATION_CASEFOLD_COLLISION"
    );

    let mut malformed_token = v2_value();
    v2_entry_mut(&mut malformed_token, "D")["methods"][0]["method_identity"]["methodDef"] =
        json!("0x0601886e");
    assert_eq!(
        parse_v2_value(malformed_token).unwrap_err().code,
        "V2_METHOD_IDENTITY_INVALID"
    );

    let mut malformed_mvid = v2_value();
    v2_entry_mut(&mut malformed_mvid, "D")["methods"][0]["method_identity"]["mvid"] =
        json!("2d6d37cb-76e5-4075-8220-3eeca41471E3");
    assert_eq!(
        parse_v2_value(malformed_mvid).unwrap_err().code,
        "V2_METHOD_IDENTITY_INVALID"
    );

    let mut wrong_identity_hash = v2_value();
    v2_entry_mut(&mut wrong_identity_hash, "D")["methods"][0]["method_identity"]["full_authority_sha256"] =
        json!("f".repeat(64));
    assert_eq!(
        parse_v2_value(wrong_identity_hash).unwrap_err().code,
        "V2_METHOD_IDENTITY_INVALID"
    );

    let mut malformed_sha = v2_value();
    v2_entry_mut(&mut malformed_sha, "D")["original_sha256"] = json!("A".repeat(64));
    assert_eq!(
        parse_v2_value(malformed_sha).unwrap_err().code,
        "V2_HASH_INVALID"
    );

    for unauthorized in ["H", "P", "G", "L", "N", "I"] {
        let mut value = v2_value();
        value["imports"][0]["id"] = json!(unauthorized);
        assert_eq!(
            parse_v2_value(value).unwrap_err().code,
            "V2_UNAUTHORIZED_IMPORT_ID",
            "unauthorized id {unauthorized}"
        );
    }
}

#[test]
fn v2_rejects_record_correspondence_and_status_promotions() {
    let mut wrong_authority = v2_value();
    v2_entry_mut(&mut wrong_authority, "D")["authority_id"] = json!("MANAGED-CS-E3FC67839852");
    assert_eq!(
        parse_v2_value(wrong_authority).unwrap_err().code,
        "V2_ENTRY_CORRESPONDENCE_INVALID"
    );

    let mut wrong_scope = v2_value();
    v2_entry_mut(&mut wrong_scope, "D")["implementation_scope"] = json!("wrong scope");
    assert_eq!(
        parse_v2_value(wrong_scope).unwrap_err().code,
        "V2_ENTRY_CORRESPONDENCE_INVALID"
    );
    let manifest = manifest_value();
    let d_record = manifest_record_value(&manifest, "D");
    let d_completeness = d_record["completeness"]
        .as_str()
        .expect("D completeness")
        .to_owned();
    for completeness in [
        "wrong completeness".to_owned(),
        format!("{d_completeness} "),
        String::new(),
        " ".to_owned(),
    ] {
        let mut value = v2_value();
        v2_entry_mut(&mut value, "D")["source_completeness"] = json!(completeness);
        assert_eq!(
            parse_v2_value(value).unwrap_err().code,
            "V2_ENTRY_CORRESPONDENCE_INVALID"
        );
    }

    let mut swapped_scope = v2_value();
    v2_entry_mut(&mut swapped_scope, "D")["implementation_scope"] =
        d_record["completeness"].clone();
    assert_eq!(
        parse_v2_value(swapped_scope).unwrap_err().code,
        "V2_ENTRY_CORRESPONDENCE_INVALID"
    );

    let mut swapped_completeness = v2_value();
    v2_entry_mut(&mut swapped_completeness, "D")["source_completeness"] = d_record["scope"].clone();
    assert_eq!(
        parse_v2_value(swapped_completeness).unwrap_err().code,
        "V2_ENTRY_CORRESPONDENCE_INVALID"
    );

    let mut t_promotion = v2_value();
    v2_entry_mut(&mut t_promotion, "T")["methods"][0]["evidence_status"] = json!("DIRECT_CIL_FULL");
    assert_eq!(
        parse_v2_value(t_promotion).unwrap_err().code,
        "V2_METHOD_STATUS_MISMATCH"
    );

    let mut d_promotion = v2_value();
    v2_entry_mut(&mut d_promotion, "D")["methods"]
        .as_array_mut()
        .expect("D methods")
        .iter_mut()
        .find(|method| method["method_identity"]["methodDef"] == "0x06018868")
        .expect("D unknown method")["evidence_status"] = json!("DIRECT_CIL_FULL");
    assert_eq!(
        parse_v2_value(d_promotion).unwrap_err().code,
        "V2_METHOD_STATUS_MISMATCH"
    );

    let mut d_root_promotion = v2_value();
    v2_entry_mut(&mut d_root_promotion, "D")["evidence_status"] = json!("DIRECT_CIL_FULL");
    assert_eq!(
        parse_v2_value(d_root_promotion).unwrap_err().code,
        "V2_ARTIFACT_STATUS_MISMATCH"
    );

    let mut manifest_method_mismatch = v2_value();
    let mut t_method = v2_entry_mut(&mut manifest_method_mismatch, "T")["methods"][0].clone();
    t_method["method_identity"]["methodDef"] = json!("0x0601FFFE");
    v2_entry_mut(&mut manifest_method_mismatch, "D")["methods"] = json!([t_method]);
    assert_eq!(
        parse_v2_value(manifest_method_mismatch).unwrap_err().code,
        "V2_METHOD_SET_MISMATCH"
    );
}

#[test]
fn v2_rejects_root_entry_and_singular_plural_schema_drift() {
    let mut invalid_version = v2_value();
    invalid_version["version"] = json!(3);
    assert_eq!(
        parse_v2_value(invalid_version).unwrap_err().code,
        "V2_ROOT_INVALID"
    );

    let mut unknown_root = v2_value();
    unknown_root["unexpected"] = Value::Bool(true);
    assert_eq!(
        parse_v2_value(unknown_root).unwrap_err().code,
        "V2_SCHEMA_INVALID"
    );

    let mut unknown_entry = v2_value();
    v2_entry_mut(&mut unknown_entry, "D")["unexpected"] = Value::Bool(true);
    assert_eq!(
        parse_v2_value(unknown_entry).unwrap_err().code,
        "V2_SCHEMA_INVALID"
    );

    let mut singular_d = v2_value();
    let method = v2_entry_mut(&mut singular_d, "D")["methods"][0]["method_identity"].clone();
    v2_entry_mut(&mut singular_d, "D")["methods"] = Value::Null;
    v2_entry_mut(&mut singular_d, "D")
        .as_object_mut()
        .expect("D object")
        .remove("methods");
    v2_entry_mut(&mut singular_d, "D")
        .as_object_mut()
        .expect("D object")
        .remove("evidence_status");
    v2_entry_mut(&mut singular_d, "D")
        .as_object_mut()
        .expect("D object")
        .remove("source_status");
    v2_entry_mut(&mut singular_d, "D")["method_identity"] = method;
    assert_eq!(
        parse_v2_value(singular_d).unwrap_err().code,
        "V2_ENTRY_SHAPE_INVALID"
    );

    let mut combined_d = v2_value();
    let d_method_identity =
        v2_entry_mut(&mut combined_d, "D")["methods"][0]["method_identity"].clone();
    v2_entry_mut(&mut combined_d, "D")["method_identity"] = d_method_identity;
    assert_eq!(
        parse_v2_value(combined_d).unwrap_err().code,
        "V2_SCHEMA_INVALID"
    );

    let mut combined_d_null = v2_value();
    v2_entry_mut(&mut combined_d_null, "D")["method_identity"] = Value::Null;
    assert_eq!(
        parse_v2_value(combined_d_null).unwrap_err().code,
        "V2_SCHEMA_INVALID"
    );

    let mut plural_e = v2_value();
    let event_method = v2_entry_mut(&mut plural_e, "E18")["method_identity"].clone();
    let manifest = manifest_value();
    let event_record = manifest_record_value(&manifest, "E18");
    let method = json!({
        "method_identity": event_method,
        "evidence_status": event_record["evidence_status"].clone(),
        "source_status": event_record["source_status"].clone()
    });
    v2_entry_mut(&mut plural_e, "E18")
        .as_object_mut()
        .expect("E18 object")
        .remove("method_identity");
    v2_entry_mut(&mut plural_e, "E18")["methods"] = json!([method]);
    v2_entry_mut(&mut plural_e, "E18")["evidence_status"] = event_record["evidence_status"].clone();
    v2_entry_mut(&mut plural_e, "E18")["source_status"] = event_record["source_status"].clone();
    assert_eq!(
        parse_v2_value(plural_e).unwrap_err().code,
        "V2_ENTRY_SHAPE_INVALID"
    );

    let mut combined_e = v2_value();
    let event_method = v2_entry_mut(&mut combined_e, "E18")["method_identity"].clone();
    let manifest = manifest_value();
    let event_record = manifest_record_value(&manifest, "E18");
    v2_entry_mut(&mut combined_e, "E18")["methods"] = json!([{
        "method_identity": event_method,
        "evidence_status": event_record["evidence_status"].clone(),
        "source_status": event_record["source_status"].clone()
    }]);
    assert_eq!(
        parse_v2_value(combined_e).unwrap_err().code,
        "V2_SCHEMA_INVALID"
    );

    let mut combined_e_null = v2_value();
    v2_entry_mut(&mut combined_e_null, "E18")["methods"] = Value::Null;
    assert_eq!(
        parse_v2_value(combined_e_null).unwrap_err().code,
        "V2_SCHEMA_INVALID"
    );
}

#[test]
fn shared_imported_file_checker_rejects_wrong_content_and_missing_files() {
    let fixture = FixtureRoot::new();
    let manifest = manifest();
    let bytes = b"good bytes";
    let entry = import_entry(&manifest, "E18", bytes);
    let imported_path = fixture.path.join(&entry.imported_path);
    fs::create_dir_all(imported_path.parent().expect("import parent")).expect("import parent");
    fs::write(&imported_path, b"xxxx bytes").expect("wrong content");
    let view = ImportedFileView {
        id: entry.id.clone(),
        original_path: entry.original_path.clone(),
        imported: re_corpus::SourceArtifact {
            path: entry.imported_path.clone(),
            size: entry.imported_size,
            sha256: entry.imported_sha256.clone(),
        },
        methods: vec![entry.method_identity.clone()],
    };
    assert_eq!(
        check_import_file(&fixture.path, &entry).unwrap_err().code,
        "IMPORTED_HASH_MISMATCH"
    );
    assert_eq!(
        check_imported_file_view(&fixture.path, &view)
            .unwrap_err()
            .code,
        "IMPORTED_HASH_MISMATCH"
    );
    fs::remove_file(&imported_path).expect("remove owned fixture");
    assert_eq!(
        check_imported_file_view(&fixture.path, &view)
            .unwrap_err()
            .code,
        "MISSING_PATH"
    );
}

#[test]
fn receipt_schema_rejects_missing_unknown_and_inexact_source_completeness() {
    let manifest = manifest();
    let bytes = b"fixture";

    let mut missing = import_value(
        &manifest,
        "E18",
        &re_corpus::sha256_hex(bytes),
        bytes.len() as u64,
    );
    missing["imports"][0]
        .as_object_mut()
        .expect("import entry object")
        .remove("source_completeness");
    assert_eq!(
        parse_receipt_bytes(&serde_json::to_vec(&missing).unwrap(), &manifest)
            .unwrap_err()
            .code,
        "RECEIPT_SCHEMA_INVALID"
    );

    let e18_completeness = manifest
        .records
        .iter()
        .find(|record| record.id == "E18")
        .expect("E18 record")
        .completeness
        .clone();
    for completeness in [
        "invented completeness".to_owned(),
        String::new(),
        format!("{e18_completeness} "),
    ] {
        let mut invalid = import_value(
            &manifest,
            "E18",
            &re_corpus::sha256_hex(bytes),
            bytes.len() as u64,
        );
        invalid["imports"][0]["source_completeness"] = Value::String(completeness);
        assert_eq!(
            parse_receipt_bytes(&serde_json::to_vec(&invalid).unwrap(), &manifest)
                .unwrap_err()
                .code,
            "RECEIPT_CORRESPONDENCE_INVALID"
        );
    }

    let mut wrong_scope = import_value(
        &manifest,
        "E18",
        &re_corpus::sha256_hex(bytes),
        bytes.len() as u64,
    );
    wrong_scope["imports"][0]["implementation_scope"] = Value::String("wrong scope".to_owned());
    assert_eq!(
        parse_receipt_bytes(&serde_json::to_vec(&wrong_scope).unwrap(), &manifest)
            .unwrap_err()
            .code,
        "RECEIPT_CORRESPONDENCE_INVALID"
    );

    let mut unknown = import_value(
        &manifest,
        "E18",
        &re_corpus::sha256_hex(bytes),
        bytes.len() as u64,
    );
    unknown["imports"][0]["unexpected"] = Value::Bool(true);
    assert_eq!(
        parse_receipt_bytes(&serde_json::to_vec(&unknown).unwrap(), &manifest)
            .unwrap_err()
            .code,
        "RECEIPT_SCHEMA_INVALID"
    );
}

#[test]
fn receipt_schema_accepts_only_approved_subset_and_rejects_bad_claims() {
    let manifest = manifest();
    let bytes = b"fixture";
    let mut value = import_value(
        &manifest,
        "E18",
        &re_corpus::sha256_hex(bytes),
        bytes.len() as u64,
    );
    let receipt = parse_receipt_bytes(
        &serde_json::to_vec(&value).expect("receipt JSON"),
        &manifest,
    )
    .expect("approved receipt subset");
    assert_eq!(receipt.imports.len(), 1);
    assert_eq!(receipt.imports[0].id, "E18");

    value["imports"][0]["id"] = Value::String("H".to_owned());
    assert_eq!(
        parse_receipt_bytes(&serde_json::to_vec(&value).unwrap(), &manifest)
            .unwrap_err()
            .code,
        "UNAUTHORIZED_IMPORT_ID"
    );

    let mut duplicate = import_value(
        &manifest,
        "E18",
        &re_corpus::sha256_hex(bytes),
        bytes.len() as u64,
    );
    let entry = duplicate["imports"][0].clone();
    duplicate["imports"] = json!([entry.clone(), entry]);
    assert_eq!(
        parse_receipt_bytes(&serde_json::to_vec(&duplicate).unwrap(), &manifest)
            .unwrap_err()
            .code,
        "DUPLICATE_IMPORT_ID"
    );

    let mut bad_status = import_value(
        &manifest,
        "E18",
        &re_corpus::sha256_hex(bytes),
        bytes.len() as u64,
    );
    bad_status["imports"][0]["publication_status"] = Value::String("ENABLED".to_owned());
    assert_eq!(
        parse_receipt_bytes(&serde_json::to_vec(&bad_status).unwrap(), &manifest)
            .unwrap_err()
            .code,
        "RECEIPT_SCHEMA_INVALID"
    );
}

#[test]
fn imported_file_hash_and_missing_file_are_fail_closed() {
    let fixture = FixtureRoot::new();
    let manifest = manifest();
    let good = b"good bytes";
    let entry = import_entry(&manifest, "E18", good);
    let imported_path = entry.imported_path.clone();
    let absolute = fixture.path.join(&imported_path);
    fs::create_dir_all(absolute.parent().expect("import parent")).expect("fixture parent");
    fs::write(&absolute, b"changed bytes").expect("changed fixture");
    assert_eq!(
        check_import_file(&fixture.path, &entry).unwrap_err().code,
        "IMPORTED_SIZE_MISMATCH"
    );

    fs::remove_file(&absolute).expect("remove owned fixture");
    assert_eq!(
        check_import_file(&fixture.path, &entry).unwrap_err().code,
        "MISSING_PATH"
    );
}

#[test]
fn lookup_arguments_and_errors_are_structured_and_private() {
    let (exit, output) = run(&["lookup", MANAGED_AUTHORITY_ID, "0x0600042e", "--json"]);
    assert_eq!(exit, 2);
    assert_eq!(output["findings"][0]["code"], "MALFORMED_METHODDEF");

    let (exit, output) = run(&["lookup", "BLD-WUWA-0001", "0x06000434", "--json"]);
    assert_eq!(exit, 1);
    assert_eq!(output["findings"][0]["code"], "NOT_FOUND");

    let (exit, output) = run(&["lookup", "MANAGED-CS-E3FC67839851", "--json"]);
    assert_eq!(exit, 2);
    assert_eq!(output["findings"][0]["code"], "INVALID_ARGUMENTS");

    let (exit, output) = run(&["status", "--json", "--json"]);
    assert_eq!(exit, 2);
    assert_eq!(output["findings"][0]["code"], "DUPLICATE_JSON_OPTION");

    let private_path = r"C:\Users\Private\research\missing";
    let (exit, output) = run(&["verify", "--research-root", private_path, "--json"]);
    assert_eq!(exit, 1);
    let rendered = serde_json::to_string(&output).expect("output JSON");
    assert!(!rendered.contains(private_path));
}

#[test]
fn root_reparse_attribute_guard_is_explicit_and_fail_closed() {
    assert!(reparse_attributes_rejected(0x400));
    assert!(!reparse_attributes_rejected(0));
}

#[test]
fn scheduling_catalog_accepts_exact_three_records_and_33_methods() {
    let bytes = scheduling_catalog_bytes();
    assert_eq!(bytes.len(), 51985);
    assert_eq!(
        re_corpus::sha256_hex(&bytes),
        re_corpus::SCHEDULING_SOURCE_V1_SHA256
    );
    let catalog = scheduling_catalog();
    assert_eq!(catalog.schema, "wuwa-reconstruction-scheduling-source-v1");
    assert_eq!(catalog.version, 1);
    assert_eq!(catalog.manifest_sha256, re_corpus::BASELINE_MANIFEST_SHA256);
    assert_eq!(
        catalog.performance_source_sha256,
        re_corpus::PERFORMANCE_SOURCE_V1_SHA256
    );
    assert_eq!(catalog.records.len(), 3);
    let mut full = BTreeSet::new();
    let mut source_only = BTreeSet::new();
    for (id, root_type, methods) in [
        ("PSCHED_GRAPH", "0x020034D4", 12),
        ("PSCHED_UTILS", "0x020034D9", 6),
        ("PSCHED_HEAP", "0x0200006B", 15),
    ] {
        let wrapper = catalog
            .records
            .iter()
            .find(|wrapper| wrapper.record.id == id)
            .expect("scheduling record");
        let record = &wrapper.record;
        assert_eq!(record.type_def.as_deref(), Some(root_type));
        assert!(record.methods.historical.is_empty());
        assert_eq!(record.methods.current.len(), methods);
        assert!(matches!(
            record.evidence_status,
            re_corpus::EvidenceStatus::Unknown
        ));
        assert!(matches!(
            record.source_status,
            re_corpus::SourceStatus::SourceOnly
        ));
        assert!(matches!(
            record.curation_status,
            re_corpus::CurationStatus::NeedsEditorialReview
        ));
        assert!(matches!(
            record.export_status,
            re_corpus::ExportStatus::PublicationReviewRequired
        ));
        for method in &record.methods.current {
            if matches!(
                method.evidence_status,
                re_corpus::EvidenceStatus::DirectCilFull
            ) {
                full.insert(method.method_def.clone());
            } else {
                source_only.insert(method.method_def.clone());
            }
        }
    }
    assert_eq!(
        full,
        [
            "0x0601C917",
            "0x06000272",
            "0x06000273",
            "0x06000277",
            "0x06000278",
            "0x06000279",
            "0x0601C942",
            "0x0601C943",
            "0x0604A4F1",
            "0x0604A4F3",
            "0x0604A4F4",
            "0x060506E6",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
    );
    assert_eq!(source_only.len(), 21);
    for excluded in ["0x0604A4EF", "0x0604A4F5", "0x060506E7"] {
        assert!(!full.contains(excluded) && !source_only.contains(excluded));
    }
    let wrapper = catalog
        .records
        .iter()
        .find(|wrapper| wrapper.record.id == "PSCHED_GRAPH")
        .expect("graph record");
    assert_eq!(wrapper.associated_types.len(), 3);
    assert_eq!(
        wrapper
            .associated_types
            .iter()
            .map(|associated| (
                associated.type_def.as_str(),
                associated.parent_type_def.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("0x0200968E", "0x020034D4"),
            ("0x0200968F", "0x020034D4"),
            ("0x0200CE0E", "0x0200968E"),
        ]
    );
    for id in ["PSCHED_UTILS", "PSCHED_HEAP"] {
        let wrapper = catalog
            .records
            .iter()
            .find(|wrapper| wrapper.record.id == id)
            .expect("non-graph record");
        assert!(wrapper.associated_types.is_empty());
    }
}

#[test]
fn scheduling_catalog_rejects_associated_root_proof_and_identity_drift() {
    let mut value = scheduling_catalog_value();
    scheduling_record_mut(&mut value, "PSCHED_GRAPH")["record"]["typeDef"] = json!("0x0200968E");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_AUTHORITY_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_method_mut(&mut value, "PSCHED_GRAPH", "0x0604A4F1")["symbol_key"]["typeDef"] =
        json!("0x020034D4");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_METHOD_IDENTITY_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_method_mut(&mut value, "PSCHED_GRAPH", "0x0601C917")["rva"] = json!("0x88EC11");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_METHOD_IDENTITY_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_method_mut(&mut value, "PSCHED_GRAPH", "0x0601C916")["evidence_status"] =
        json!("DIRECT_CIL_FULL");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_METHOD_PROOF_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_method_mut(&mut value, "PSCHED_GRAPH", "0x0601C917")["evidence_status"] =
        json!("SOURCE_ONLY");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_METHOD_PROOF_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_record_mut(&mut value, "PSCHED_GRAPH")["associated_types"][2]["parent_typeDef"] =
        json!("0x020034D4");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_ASSOCIATED_TYPE_SET_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_record_mut(&mut value, "PSCHED_GRAPH")["associated_types"]
        .as_array_mut()
        .expect("associated types")
        .pop();
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_ASSOCIATED_TYPE_SET_INVALID"
    );

    let mut value = scheduling_catalog_value();
    let duplicate = scheduling_record_value(&value, "PSCHED_GRAPH")["associated_types"][0].clone();
    scheduling_record_mut(&mut value, "PSCHED_GRAPH")["associated_types"]
        .as_array_mut()
        .expect("associated types")
        .push(duplicate);
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_ASSOCIATED_TYPE_SET_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_record_mut(&mut value, "PSCHED_GRAPH")["associated_types"][0]["typeDef"] =
        json!("0x200968E");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_ASSOCIATED_TYPE_SET_INVALID"
    );

    let mut value = scheduling_catalog_value();
    let plumbing = scheduling_method_mut(&mut value, "PSCHED_GRAPH", "0x0604A4F0").clone();
    let mut plumbing = plumbing;
    plumbing["methodDef"] = json!("0x0604A4EF");
    plumbing["symbol_key"]["methodDef"] = json!("0x0604A4EF");
    scheduling_record_mut(&mut value, "PSCHED_GRAPH")["record"]["methods"]["current"]
        .as_array_mut()
        .expect("graph methods")
        .push(plumbing);
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_METHOD_SET_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_record_mut(&mut value, "PSCHED_GRAPH")["record"]["original_sha256"] =
        json!("f".repeat(64));
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_SOURCE_IDENTITY_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_record_mut(&mut value, "PSCHED_GRAPH")["record"]["proposed_path"] =
        json!("src/managed/cs-e3fc67839851/scheduling/Other.cs");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_DESTINATION_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_record_mut(&mut value, "PSCHED_GRAPH")["record"]["evidence_references"]
        .as_array_mut()
        .expect("record references")
        .retain(|reference| {
            reference["path"] != "evidence/performance/bounded_il_critical.cil.txt"
        });
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_EVIDENCE_REFERENCE_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_method_mut(&mut value, "PSCHED_GRAPH", "0x0601C917")["evidence_references"]
        .as_array_mut()
        .expect("method references")
        .pop();
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_EVIDENCE_REFERENCE_INVALID"
    );

    let mut value = scheduling_catalog_value();
    value["unexpected"] = Value::Bool(true);
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_CATALOG_SCHEMA_INVALID"
    );

    assert_eq!(
        parse_scheduling_source_bytes(
            br#"{"schema":"a","schema":"b"}"#,
            &manifest(),
            &performance_catalog()
        )
        .unwrap_err()
        .code,
        "DUPLICATE_JSON_KEY"
    );
}

#[test]
fn scheduling_catalog_rejects_enum_token_scope_and_dependency_drift() {
    let mut value = scheduling_catalog_value();
    scheduling_record_mut(&mut value, "PSCHED_GRAPH")["record"]["source_status"] =
        json!("NOT_A_STATUS");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_CATALOG_SCHEMA_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_method_mut(&mut value, "PSCHED_GRAPH", "0x0604A4F1")["symbol_key"]["mvid"] =
        json!("2D6D37CB-76E5-4075-8220-3EECA41471E3");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_METHOD_IDENTITY_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_method_mut(&mut value, "PSCHED_GRAPH", "0x0604A4F1")["symbol_key"]["typeDef"] =
        json!("0x0200968e");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_METHOD_IDENTITY_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_method_mut(&mut value, "PSCHED_GRAPH", "0x0601C917")["symbol_key"]["methodDef"] =
        json!("0x0601c917");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_METHOD_IDENTITY_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_record_mut(&mut value, "PSCHED_UTILS")["record"]["scope"] = json!("");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_INCOMPLETE_SCOPE"
    );

    let mut value = scheduling_catalog_value();
    scheduling_record_mut(&mut value, "PSCHED_UTILS")["record"]["dependencies"] = json!([]);
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_INCOMPLETE_RECORD"
    );

    let mut value = scheduling_catalog_value();
    scheduling_record_mut(&mut value, "PSCHED_GRAPH")["associated_types"][0]["parent_typeDef"] =
        json!("0x020034d4");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "SCHEDULING_ASSOCIATED_TYPE_SET_INVALID"
    );

    let mut value = scheduling_catalog_value();
    scheduling_method_mut(&mut value, "PSCHED_GRAPH", "0x0601C917")["evidence_references"][0]["locator"] =
        json!("");
    assert_eq!(
        parse_scheduling_value(value).unwrap_err().code,
        "EMPTY_EVIDENCE_LOCATOR"
    );
}

#[test]
fn v4_accepts_exact_ten_entries_and_maps_fifty_one_methods() {
    let parsed = parse_v4_value(v4_value()).expect("accepted v4 fixture");
    assert_eq!(parsed.imports.len(), 10);
    let historical = historical_receipt_v3();
    assert_eq!(&parsed.imports[..7], &historical.imports[..]);
    let mut methods = 0;
    let mut new_full = 0;
    let mut new_source_only = 0;
    for entry in &parsed.imports {
        match entry {
            ImportEntryV2::Event(_) => methods += 1,
            ImportEntryV2::Mapped(entry) => {
                methods += entry.methods.len();
                if matches!(
                    entry.id.as_str(),
                    "PSCHED_GRAPH" | "PSCHED_UTILS" | "PSCHED_HEAP"
                ) {
                    for method in &entry.methods {
                        if matches!(
                            method.evidence_status,
                            re_corpus::EvidenceStatus::DirectCilFull
                        ) {
                            new_full += 1;
                        } else {
                            new_source_only += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(methods, 51);
    assert_eq!(new_full, 12);
    assert_eq!(new_source_only, 21);
}

#[test]
fn v4_fixture_activates_ten_files_and_reports_combined_counts() {
    let (fixture, active) = fixture_v4_active();
    assert_eq!(active.version, 4);
    assert_eq!(active.ledger_path, RECEIPT_V4_REL);
    assert_eq!(active.files.len(), 10);
    assert_eq!(
        active
            .files
            .iter()
            .map(|file| file.methods.len())
            .sum::<usize>(),
        51
    );

    let (exit, output) = run_status_at(&fixture.path);
    assert_eq!(exit, 1);
    assert_eq!(output["counts"]["historical_entries"], 18);
    assert_eq!(output["counts"]["current_entries"], 57);
    assert_eq!(output["counts"]["unique_qualified_symbols"], 53);
    assert_eq!(output["counts"]["planned"], 11);
    assert_eq!(output["counts"]["imported"], 10);
    assert_eq!(output["counts"]["imported_file_count"], 10);
    assert_eq!(output["counts"]["imported_qualified_methods"], 51);
    assert_eq!(output["counts"]["missing_imports"], 1);
    assert_eq!(output["active_ledger"]["path"], RECEIPT_V4_REL);
    assert_eq!(output["active_ledger"]["version"], 4);
    let records = output["records"].as_array().expect("records");
    assert_eq!(records.len(), 16);
    for id in ["PSCHED_GRAPH", "PSCHED_UTILS", "PSCHED_HEAP"] {
        let record = records
            .iter()
            .find(|record| record["id"] == id)
            .expect("scheduling record in status");
        assert_eq!(record["import_status"], "IMPORTED");
    }
    assert_eq!(
        output["findings"]
            .as_array()
            .expect("findings")
            .iter()
            .filter(|finding| finding["code"] == "MISSING_IMPORTED_FILE")
            .count(),
        10
    );

    for (id, record_type, method_def, method_type) in [
        ("PSCHED_GRAPH", "0x020034D4", "0x0601C917", "0x020034D4"),
        ("PSCHED_GRAPH", "0x020034D4", "0x0604A4F1", "0x0200968E"),
        ("PSCHED_UTILS", "0x020034D9", "0x0601C943", "0x020034D9"),
        ("PSCHED_HEAP", "0x0200006B", "0x06000278", "0x0200006B"),
    ] {
        let (exit, output) = run_lookup_at(&fixture.path, MANAGED_AUTHORITY_ID, method_def);
        assert_eq!(exit, 0, "lookup {method_def}");
        let matches = output["matches"].as_array().expect("matches");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0]["id"], id);
        assert_eq!(matches[0]["typeDef"], record_type);
        assert_eq!(
            matches[0]["current"][0]["qualified_key"]["typeDef"],
            method_type
        );
        assert_eq!(output["counts"]["imported"], 10);
        assert_eq!(output["counts"]["imported_qualified_methods"], 51);
    }

    let (exit, output) = run_verify_at(&fixture.path, None);
    assert_eq!(exit, 1);
    assert_eq!(output["active_ledger"]["version"], 4);
    assert_eq!(output["counts"]["imported"], 10);
    assert_eq!(output["counts"]["imported_qualified_methods"], 51);
    assert_eq!(output["missing"].as_array().expect("missing").len(), 10);
    let not_checked = output["not_checked"].as_array().expect("not_checked");
    for id in ["PSCHED_GRAPH", "PSCHED_UTILS", "PSCHED_HEAP"] {
        assert!(not_checked.iter().any(|item| {
            item["kind"] == "ORIGINAL_SOURCE" && item["id"] == id && item["status"] == "NOT_CHECKED"
        }));
    }
}

#[test]
fn v4_rejects_old_entry_new_record_and_root_drift() {
    let mut value = v4_value();
    v4_entry_mut(&mut value, "E18")["imported_sha256"] = json!("f".repeat(64));
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_HISTORICAL_ENTRY_MISMATCH"
    );

    let mut value = v4_value();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["implementation_scope"] = json!("drifted scope");
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_ENTRY_CORRESPONDENCE_INVALID"
    );

    let mut value = v4_value();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["methods"][0]["evidence_status"] =
        json!("DIRECT_CIL_FULL");
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_ENTRY_CORRESPONDENCE_INVALID"
    );

    let mut value = v4_value();
    value["previous_receipt_sha256"] = json!("f".repeat(64));
    assert_eq!(parse_v4_value(value).unwrap_err().code, "V4_ROOT_INVALID");

    let mut value = v4_value();
    value["scheduling_source_sha256"] = json!("f".repeat(64));
    assert_eq!(parse_v4_value(value).unwrap_err().code, "V4_ROOT_INVALID");

    let mut value = v4_value();
    value["unexpected"] = Value::Bool(true);
    assert_eq!(parse_v4_value(value).unwrap_err().code, "V4_SCHEMA_INVALID");
}

#[test]
fn v4_rejects_duplicate_missing_extra_methods_ids_and_paths() {
    let mut value = v4_value();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["methods"]
        .as_array_mut()
        .expect("graph methods")
        .pop();
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_ENTRY_CORRESPONDENCE_INVALID"
    );

    let mut value = v4_value();
    let duplicate = v4_entry_mut(&mut value, "PSCHED_GRAPH")["methods"][0].clone();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["methods"]
        .as_array_mut()
        .expect("graph methods")
        .push(duplicate);
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_DUPLICATE_QUALIFIED_METHOD"
    );

    let mut value = v4_value();
    v4_entry_mut(&mut value, "PSCHED_UTILS")["id"] = json!("PSCHED_GRAPH");
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_DUPLICATE_IMPORT_ID"
    );

    let mut value = v4_value();
    v4_entry_mut(&mut value, "PSCHED_UTILS")["id"] = json!("P");
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_UNAUTHORIZED_IMPORT_ID"
    );

    let mut value = v4_value();
    value["imports"]
        .as_array_mut()
        .expect("v4 imports")
        .retain(|entry| entry["id"] != "PSCHED_HEAP");
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_IMPORT_ID_SET_INVALID"
    );

    let mut value = v4_value();
    let heap_path = v4_entry_mut(&mut value, "PSCHED_HEAP")["imported_path"].clone();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["imported_path"] =
        Value::String(heap_path.as_str().expect("heap path").to_ascii_uppercase());
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_DESTINATION_CASEFOLD_COLLISION"
    );

    let mut value = v4_value();
    let heap_source = v4_entry_mut(&mut value, "PSCHED_HEAP")["original_path"].clone();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["original_path"] = heap_source;
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_SOURCE_PATH_COLLISION"
    );

    let mut value = v4_value();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["imported_path"] = json!("../escape.cs");
    assert_eq!(parse_v4_value(value).unwrap_err().code, "V4_PATH_INVALID");

    let mut value = v4_value();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["original_sha256"] = json!("A".repeat(64));
    assert_eq!(parse_v4_value(value).unwrap_err().code, "V4_HASH_INVALID");

    let mut value = v4_value();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["methods"][0]["method_identity"]["mvid"] =
        json!("2D6D37CB-76E5-4075-8220-3EECA41471E3");
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_METHOD_IDENTITY_INVALID"
    );
}

#[test]
fn v4_rejects_singular_plural_null_and_combined_shapes() {
    let mut value = v4_value();
    let method = v4_entry_mut(&mut value, "PSCHED_GRAPH")["methods"][0]["method_identity"].clone();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["method_identity"] = method;
    assert_eq!(parse_v4_value(value).unwrap_err().code, "V4_SCHEMA_INVALID");

    let mut value = v4_value();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["methods"] = Value::Null;
    assert_eq!(parse_v4_value(value).unwrap_err().code, "V4_SCHEMA_INVALID");

    let mut value = v4_value();
    v4_entry_mut(&mut value, "E18")["methods"] = json!([]);
    assert_eq!(parse_v4_value(value).unwrap_err().code, "V4_SCHEMA_INVALID");

    let mut value = v4_value();
    v4_entry_mut(&mut value, "PSCHED_GRAPH")["evidence_status"] = json!("NOT_A_STATUS");
    assert_eq!(parse_v4_value(value).unwrap_err().code, "V4_SCHEMA_INVALID");

    let mut value = v4_value();
    let event_method = v4_entry_mut(&mut value, "E18")["method_identity"].clone();
    let manifest = manifest_value();
    let event_record = manifest_record_value(&manifest, "E18");
    v4_entry_mut(&mut value, "E18")
        .as_object_mut()
        .expect("E18 object")
        .remove("method_identity");
    v4_entry_mut(&mut value, "E18")["methods"] = json!([{
        "method_identity": event_method,
        "evidence_status": event_record["evidence_status"].clone(),
        "source_status": event_record["source_status"].clone()
    }]);
    v4_entry_mut(&mut value, "E18")["evidence_status"] = event_record["evidence_status"].clone();
    v4_entry_mut(&mut value, "E18")["source_status"] = event_record["source_status"].clone();
    assert_eq!(
        parse_v4_value(value).unwrap_err().code,
        "V4_ENTRY_SHAPE_INVALID"
    );
}

#[test]
fn v4_loader_fails_closed_on_history_catalogs_and_no_fallback() {
    let (fixture, _) = fixture_v4_active();
    fs::remove_file(fixture.path.join(RECEIPT_V3_REL)).expect("remove owned v3");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_V3_RECEIPT_MISSING"
    );

    let (fixture, _) = fixture_v4_active();
    let mut altered_v3 = receipt_v3_bytes();
    altered_v3.push(b'\n');
    fs::write(fixture.path.join(RECEIPT_V3_REL), altered_v3).expect("alter owned v3");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_V3_RECEIPT_HASH_MISMATCH"
    );

    let (fixture, _) = fixture_v4_active();
    let mut altered_v2 = receipt_v2_bytes();
    altered_v2.push(b'\n');
    fs::write(fixture.path.join(RECEIPT_V2_REL), altered_v2).expect("alter owned v2");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_PREVIOUS_RECEIPT_HASH_MISMATCH"
    );

    let (fixture, _) = fixture_v4_active();
    let mut altered_v1 = receipt_bytes();
    altered_v1.push(b'\n');
    fs::write(fixture.path.join(RECEIPT_REL), altered_v1).expect("alter owned v1");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_HISTORICAL_RECEIPT_HASH_MISMATCH"
    );

    let (fixture, _) = fixture_v4_active();
    fs::remove_file(fixture.path.join(RECEIPT_V2_REL)).expect("remove owned v2");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_PREVIOUS_RECEIPT_MISSING"
    );

    let (fixture, _) = fixture_v4_active();
    fs::remove_file(fixture.path.join(RECEIPT_REL)).expect("remove owned v1");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_HISTORICAL_RECEIPT_MISSING"
    );

    let (fixture, _) = fixture_v4_active();
    fs::remove_file(fixture.path.join(PERFORMANCE_SOURCE_REL)).expect("remove owned timer catalog");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_PERFORMANCE_SOURCE_MISSING"
    );

    let (fixture, _) = fixture_v4_active();
    fs::remove_file(fixture.path.join(SCHEDULING_SOURCE_REL))
        .expect("remove owned scheduling catalog");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_SCHEDULING_SOURCE_MISSING"
    );

    let (fixture, _) = fixture_v4_active();
    fs::remove_file(fixture.path.join(SCHEDULING_SOURCE_REL))
        .expect("remove owned scheduling catalog");
    fs::create_dir(fixture.path.join(SCHEDULING_SOURCE_REL))
        .expect("create scheduling catalog directory");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_SCHEDULING_SOURCE_NOT_REGULAR"
    );

    let (fixture, _) = fixture_v4_active();
    fs::remove_file(fixture.path.join(PERFORMANCE_SOURCE_REL)).expect("remove owned timer catalog");
    fs::create_dir(fixture.path.join(PERFORMANCE_SOURCE_REL))
        .expect("create timer catalog directory");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_PERFORMANCE_SOURCE_NOT_REGULAR"
    );

    let (fixture, _) = fixture_v4_active();
    let mut altered_scheduling = scheduling_catalog_bytes();
    altered_scheduling.push(b'\n');
    fs::write(fixture.path.join(SCHEDULING_SOURCE_REL), altered_scheduling)
        .expect("alter owned scheduling catalog");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "SCHEDULING_ANCHOR_MISMATCH"
    );

    let (fixture, _) = fixture_v4_active();
    fs::write(fixture.path.join(SCHEDULING_SOURCE_REL), b"{").expect("malform scheduling catalog");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "MALFORMED_JSON"
    );

    let (fixture, _) = fixture_v4_active();
    fs::write(fixture.path.join(RECEIPT_V4_REL), b"{").expect("malform v4");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "MALFORMED_JSON"
    );

    let (fixture, _) = fixture_v4_active();
    fs::remove_file(fixture.path.join(RECEIPT_V4_REL)).expect("remove owned v4");
    fs::create_dir(fixture.path.join(RECEIPT_V4_REL)).expect("create v4 directory");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_RECEIPT_NOT_REGULAR"
    );

    let mut invalid_v4 = v4_value();
    invalid_v4["version"] = json!(3);
    let fixture = FixtureRoot::new();
    write_fixture_v4(
        &fixture,
        Some(&invalid_v4),
        Some(&scheduling_catalog_bytes()),
        &catalog_bytes(),
        &receipt_v3_bytes(),
        &receipt_v2_bytes(),
        &receipt_bytes(),
    );
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V4_ROOT_INVALID"
    );
}

#[test]
fn v4_absent_keeps_v3_and_scheduling_catalog_does_not_inject_records() {
    let fixture = FixtureRoot::new();
    write_fixture_v4(
        &fixture,
        None,
        Some(&scheduling_catalog_bytes()),
        &catalog_bytes(),
        &receipt_v3_bytes(),
        &receipt_v2_bytes(),
        &receipt_bytes(),
    );
    let active = load_active_receipt(&fixture.path)
        .expect("v4-absent fixture loads")
        .expect("v3 becomes active");
    assert_eq!(active.version, 3);
    assert_eq!(active.ledger_path, RECEIPT_V3_REL);
    assert_eq!(active.files.len(), 7);
}

#[test]
fn shared_imported_file_checker_handles_multi_method_scheduling_view() {
    let fixture = FixtureRoot::new();
    let catalog = scheduling_catalog();
    let record = &catalog
        .records
        .iter()
        .find(|wrapper| wrapper.record.id == "PSCHED_GRAPH")
        .expect("graph record")
        .record;
    let methods = record
        .methods
        .current
        .iter()
        .map(|method| method.symbol_key.clone())
        .collect::<Vec<_>>();
    assert_eq!(methods.len(), 12);
    let bytes = b"synthetic multi-method scheduling view";
    let view = ImportedFileView {
        id: record.id.clone(),
        original_path: record.original_path.clone(),
        imported: re_corpus::SourceArtifact {
            path: record.proposed_path.clone().expect("proposed path"),
            size: bytes.len() as u64,
            sha256: re_corpus::sha256_hex(bytes),
        },
        methods,
    };
    let imported_path = fixture.path.join(&view.imported.path);
    fs::create_dir_all(imported_path.parent().expect("import parent")).expect("import parent");
    fs::write(&imported_path, bytes).expect("write owned fixture");
    assert_eq!(
        check_imported_file_view(&fixture.path, &view).expect("multi-method file validates"),
        re_corpus::VerificationResult::Pass
    );
    fs::write(&imported_path, b"changed multi-method scheduling view")
        .expect("change owned fixture");
    assert_eq!(
        check_imported_file_view(&fixture.path, &view)
            .unwrap_err()
            .code,
        "IMPORTED_SIZE_MISMATCH"
    );
    fs::remove_file(&imported_path).expect("remove owned fixture");
    assert_eq!(
        check_imported_file_view(&fixture.path, &view)
            .unwrap_err()
            .code,
        "MISSING_PATH"
    );
}

#[test]
fn task_catalog_accepts_exact_three_records_51_methods_and_abstract_metadata() {
    let bytes = task_catalog_bytes();
    assert_eq!(bytes.len(), 96117);
    assert_eq!(
        re_corpus::sha256_hex(&bytes),
        re_corpus::TASK_EXECUTION_SOURCE_V1_SHA256
    );
    let catalog = task_catalog();
    assert_eq!(
        catalog.schema,
        "wuwa-reconstruction-task-execution-source-v1"
    );
    assert_eq!(catalog.version, 1);
    assert_eq!(catalog.records.len(), 3);
    assert_eq!(
        catalog
            .records
            .iter()
            .map(|wrapper| wrapper.record.id.as_str())
            .collect::<Vec<_>>(),
        vec!["PEXEC_SYSTEM", "PEXEC_BASE", "PEXEC_QUEUE"]
    );
    let mut total = 0_usize;
    let mut full = 0_usize;
    let mut source_only = 0_usize;
    let mut metadata = 0_usize;
    for wrapper in &catalog.records {
        let record = &wrapper.record;
        assert!(record.methods.historical.is_empty());
        assert!(matches!(
            record.evidence_status,
            re_corpus::EvidenceStatus::Unknown
        ));
        assert!(matches!(
            record.source_status,
            re_corpus::SourceStatus::SourceOnly
        ));
        assert_eq!(record.representation, "MIXED");
        total += record.methods.current.len();
        for method in &record.methods.current {
            match method.evidence_status {
                re_corpus::EvidenceStatus::DirectCilFull => full += 1,
                re_corpus::EvidenceStatus::SourceOnly => source_only += 1,
                re_corpus::EvidenceStatus::MetadataOnly => metadata += 1,
                _ => panic!("unexpected proof status"),
            }
        }
    }
    assert_eq!((total, full, source_only, metadata), (51, 23, 27, 1));
    let base = catalog
        .records
        .iter()
        .find(|wrapper| wrapper.record.id == "PEXEC_BASE")
        .expect("base record");
    let abstract_method = base
        .record
        .methods
        .current
        .iter()
        .find(|method| method.method_def == "0x0601C913")
        .expect("abstract method");
    assert!(matches!(
        abstract_method.evidence_status,
        re_corpus::EvidenceStatus::MetadataOnly
    ));
    assert!(matches!(
        abstract_method.source_status,
        re_corpus::SourceStatus::MetadataOnly
    ));
    assert_eq!(abstract_method.rva.as_deref(), Some("0x0"));
    assert!(abstract_method.il_range.is_none());
    assert!(abstract_method.verification_basis.contains("no body"));
    assert_eq!(base.associated_types.len(), 1);
    assert_eq!(base.associated_types[0].type_def, "0x0200968D");
    let system = catalog
        .records
        .iter()
        .find(|wrapper| wrapper.record.id == "PEXEC_SYSTEM")
        .expect("system record");
    assert_eq!(system.associated_types.len(), 4);
    let queue = catalog
        .records
        .iter()
        .find(|wrapper| wrapper.record.id == "PEXEC_QUEUE")
        .expect("queue record");
    assert_eq!(queue.associated_types.len(), 1);
}

#[test]
fn task_catalog_rejects_association_proof_and_abstract_body_drift() {
    let mut value = task_catalog_value();
    task_method_mut(&mut value, "PEXEC_SYSTEM", "0x0604A4F8")["symbol_key"]["typeDef"] =
        json!("0x020034D8");
    assert_eq!(
        parse_task_value(value).unwrap_err().code,
        "TASK_METHOD_IDENTITY_INVALID"
    );

    let mut value = task_catalog_value();
    task_record_mut(&mut value, "PEXEC_SYSTEM")["associated_types"][1]["parent_typeDef"] =
        json!("0x02009691");
    assert_eq!(
        parse_task_value(value).unwrap_err().code,
        "TASK_ASSOCIATED_TYPE_SET_INVALID"
    );

    let mut value = task_catalog_value();
    task_record_mut(&mut value, "PEXEC_SYSTEM")["associated_types"]
        .as_array_mut()
        .expect("associated types")
        .pop();
    assert_eq!(
        parse_task_value(value).unwrap_err().code,
        "TASK_ASSOCIATED_TYPE_SET_INVALID"
    );

    let mut value = task_catalog_value();
    let duplicate = task_record_value(&value, "PEXEC_SYSTEM")["associated_types"][0].clone();
    task_record_mut(&mut value, "PEXEC_SYSTEM")["associated_types"]
        .as_array_mut()
        .expect("associated types")
        .push(duplicate);
    assert_eq!(
        parse_task_value(value).unwrap_err().code,
        "TASK_ASSOCIATED_TYPE_SET_INVALID"
    );

    let mut value = task_catalog_value();
    task_method_mut(&mut value, "PEXEC_SYSTEM", "0x0601C932")["evidence_status"] =
        json!("SOURCE_ONLY");
    assert_eq!(
        parse_task_value(value).unwrap_err().code,
        "TASK_METHOD_PROOF_INVALID"
    );

    let mut value = task_catalog_value();
    task_method_mut(&mut value, "PEXEC_BASE", "0x0601C913")["rva"] = json!("0x88EB00");
    assert_eq!(
        parse_task_value(value).unwrap_err().code,
        "TASK_METHOD_IDENTITY_INVALID"
    );

    let mut value = task_catalog_value();
    task_method_mut(&mut value, "PEXEC_BASE", "0x0601C913")["il_range"] = json!("[0x0000,0x0000)");
    assert_eq!(
        parse_task_value(value).unwrap_err().code,
        "TASK_METHOD_IDENTITY_INVALID"
    );

    let mut value = task_catalog_value();
    task_method_mut(&mut value, "PEXEC_BASE", "0x0601C913")["verification_basis"] =
        json!("abstract signature; metadata only.");
    assert_eq!(
        parse_task_value(value).unwrap_err().code,
        "TASK_METADATA_BODY_INVENTED"
    );

    let mut value = task_catalog_value();
    task_record_mut(&mut value, "PEXEC_SYSTEM")["record"]["representation"] = json!("SOURCE");
    assert_eq!(
        parse_task_value(value).unwrap_err().code,
        "TASK_STATUS_INVALID"
    );

    let mut value = task_catalog_value();
    let extra = task_method_mut(&mut value, "PEXEC_QUEUE", "0x060002A3").clone();
    let mut extra = extra;
    extra["methodDef"] = json!("0x060002B0");
    extra["symbol_key"]["methodDef"] = json!("0x060002B0");
    task_record_mut(&mut value, "PEXEC_QUEUE")["record"]["methods"]["current"]
        .as_array_mut()
        .expect("queue methods")
        .push(extra);
    assert_eq!(
        parse_task_value(value).unwrap_err().code,
        "TASK_METHOD_SET_INVALID"
    );

    let mut value = task_catalog_value();
    value["unexpected"] = Value::Bool(true);
    assert_eq!(
        parse_task_value(value).unwrap_err().code,
        "TASK_CATALOG_SCHEMA_INVALID"
    );

    assert_eq!(
        parse_task_execution_source_bytes(
            br#"{"schema":"a","schema":"b"}"#,
            &manifest(),
            &performance_catalog(),
            &scheduling_catalog()
        )
        .unwrap_err()
        .code,
        "DUPLICATE_JSON_KEY"
    );
}

#[test]
fn v5_accepts_exact_thirteen_entries_and_maps_102_methods() {
    let parsed = parse_v5_value(v5_value()).expect("accepted v5 fixture");
    assert_eq!(parsed.imports.len(), 13);
    let historical = historical_receipt_v4();
    assert_eq!(&parsed.imports[..10], &historical.imports[..]);
    let mut methods = 0;
    let mut new_full = 0;
    let mut new_source = 0;
    let mut new_metadata = 0;
    for entry in &parsed.imports {
        match entry {
            ImportEntryV2::Event(_) => methods += 1,
            ImportEntryV2::Mapped(entry) => {
                methods += entry.methods.len();
                if matches!(
                    entry.id.as_str(),
                    "PEXEC_SYSTEM" | "PEXEC_BASE" | "PEXEC_QUEUE"
                ) {
                    for method in &entry.methods {
                        match method.evidence_status {
                            re_corpus::EvidenceStatus::DirectCilFull => new_full += 1,
                            re_corpus::EvidenceStatus::SourceOnly => new_source += 1,
                            re_corpus::EvidenceStatus::MetadataOnly => new_metadata += 1,
                            _ => panic!("unexpected new status"),
                        }
                    }
                }
            }
        }
    }
    assert_eq!(methods, 102);
    assert_eq!(new_full, 23);
    assert_eq!(new_source, 27);
    assert_eq!(new_metadata, 1);
}

#[test]
fn v5_fixture_activates_thirteen_files_and_reports_combined_counts() {
    let (fixture, active) = fixture_v5_active();
    assert_eq!(active.version, 5);
    assert_eq!(active.ledger_path, RECEIPT_V5_REL);
    assert_eq!(active.files.len(), 13);
    assert_eq!(
        active
            .files
            .iter()
            .map(|file| file.methods.len())
            .sum::<usize>(),
        102
    );
    let (exit, output) = run_status_at(&fixture.path);
    assert_eq!(exit, 1);
    assert_eq!(output["counts"]["historical_entries"], 18);
    assert_eq!(output["counts"]["current_entries"], 108);
    assert_eq!(output["counts"]["unique_qualified_symbols"], 104);
    assert_eq!(output["counts"]["planned"], 14);
    assert_eq!(output["counts"]["imported"], 13);
    assert_eq!(output["counts"]["imported_file_count"], 13);
    assert_eq!(output["counts"]["imported_qualified_methods"], 102);
    assert_eq!(output["counts"]["missing_imports"], 1);
    assert_eq!(output["active_ledger"]["version"], 5);
    assert_eq!(output["records"].as_array().expect("records").len(), 19);
    assert_eq!(
        output["findings"]
            .as_array()
            .expect("findings")
            .iter()
            .filter(|finding| finding["code"] == "MISSING_IMPORTED_FILE")
            .count(),
        13
    );
    for (id, method_def, method_type) in [
        ("PEXEC_SYSTEM", "0x0601C933", "0x020034D8"),
        ("PEXEC_SYSTEM", "0x0604A4F8", "0x02009692"),
        ("PEXEC_BASE", "0x0601C913", "0x020034D3"),
        ("PEXEC_QUEUE", "0x060002A5", "0x02000071"),
    ] {
        let (exit, output) = run_lookup_at(&fixture.path, MANAGED_AUTHORITY_ID, method_def);
        assert_eq!(exit, 0);
        assert_eq!(output["matches"][0]["id"], id);
        assert_eq!(
            output["matches"][0]["current"][0]["qualified_key"]["typeDef"],
            method_type
        );
        assert_eq!(output["counts"]["imported"], 13);
        assert_eq!(output["counts"]["imported_qualified_methods"], 102);
    }
    let (exit, output) = run_verify_at(&fixture.path, None);
    assert_eq!(exit, 1);
    assert_eq!(output["active_ledger"]["version"], 5);
    assert_eq!(output["counts"]["imported"], 13);
    assert_eq!(output["counts"]["imported_qualified_methods"], 102);
    assert_eq!(output["missing"].as_array().expect("missing").len(), 13);
}

#[test]
fn v5_rejects_history_new_record_and_root_drift() {
    let mut value = v5_value();
    v5_entry_mut(&mut value, "E18")["imported_sha256"] = json!("f".repeat(64));
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_HISTORICAL_ENTRY_MISMATCH"
    );
    let mut value = v5_value();
    v5_entry_mut(&mut value, "PEXEC_SYSTEM")["implementation_scope"] = json!("drifted");
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_ENTRY_CORRESPONDENCE_INVALID"
    );
    let mut value = v5_value();
    v5_entry_mut(&mut value, "PEXEC_SYSTEM")["methods"][0]["evidence_status"] =
        json!("DIRECT_CIL_FULL");
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_ENTRY_CORRESPONDENCE_INVALID"
    );
    let mut value = v5_value();
    value["previous_receipt_sha256"] = json!("f".repeat(64));
    assert_eq!(parse_v5_value(value).unwrap_err().code, "V5_ROOT_INVALID");
    let mut value = v5_value();
    value["task_execution_source_sha256"] = json!("f".repeat(64));
    assert_eq!(parse_v5_value(value).unwrap_err().code, "V5_ROOT_INVALID");
    let mut value = v5_value();
    value["unexpected"] = Value::Bool(true);
    assert_eq!(parse_v5_value(value).unwrap_err().code, "V5_SCHEMA_INVALID");
}

#[test]
fn v5_rejects_duplicate_missing_extra_methods_ids_and_paths() {
    let mut value = v5_value();
    v5_entry_mut(&mut value, "PEXEC_QUEUE")["methods"]
        .as_array_mut()
        .expect("queue methods")
        .pop();
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_ENTRY_CORRESPONDENCE_INVALID"
    );
    let mut value = v5_value();
    let duplicate = v5_entry_mut(&mut value, "PEXEC_QUEUE")["methods"][0].clone();
    v5_entry_mut(&mut value, "PEXEC_QUEUE")["methods"]
        .as_array_mut()
        .expect("queue methods")
        .push(duplicate);
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_DUPLICATE_QUALIFIED_METHOD"
    );
    let mut value = v5_value();
    v5_entry_mut(&mut value, "PEXEC_BASE")["id"] = json!("PEXEC_SYSTEM");
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_DUPLICATE_IMPORT_ID"
    );
    let mut value = v5_value();
    v5_entry_mut(&mut value, "PEXEC_BASE")["id"] = json!("P");
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_UNAUTHORIZED_IMPORT_ID"
    );
    let mut value = v5_value();
    value["imports"]
        .as_array_mut()
        .expect("imports")
        .retain(|entry| entry["id"] != "PEXEC_QUEUE");
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_IMPORT_ID_SET_INVALID"
    );
    let mut value = v5_value();
    let queue_path = v5_entry_mut(&mut value, "PEXEC_QUEUE")["imported_path"].clone();
    v5_entry_mut(&mut value, "PEXEC_SYSTEM")["imported_path"] = Value::String(
        queue_path
            .as_str()
            .expect("queue path")
            .to_ascii_uppercase(),
    );
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_DESTINATION_CASEFOLD_COLLISION"
    );
    let mut value = v5_value();
    let queue_source = v5_entry_mut(&mut value, "PEXEC_QUEUE")["original_path"].clone();
    v5_entry_mut(&mut value, "PEXEC_SYSTEM")["original_path"] = queue_source;
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_SOURCE_PATH_COLLISION"
    );
    let mut value = v5_value();
    v5_entry_mut(&mut value, "PEXEC_SYSTEM")["imported_path"] = json!("../escape.cs");
    assert_eq!(parse_v5_value(value).unwrap_err().code, "V5_PATH_INVALID");
    let mut value = v5_value();
    v5_entry_mut(&mut value, "PEXEC_SYSTEM")["original_sha256"] = json!("A".repeat(64));
    assert_eq!(parse_v5_value(value).unwrap_err().code, "V5_HASH_INVALID");
    let mut value = v5_value();
    v5_entry_mut(&mut value, "PEXEC_SYSTEM")["methods"][0]["method_identity"]["mvid"] =
        json!("2D6D37CB-76E5-4075-8220-3EECA41471E3");
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_METHOD_IDENTITY_INVALID"
    );
}

#[test]
fn v5_loader_fails_closed_and_v5_absent_preserves_v4() {
    let (fixture, _) = fixture_v5_active();
    fs::remove_file(fixture.path.join(RECEIPT_V4_REL)).expect("remove owned v4");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V5_V4_RECEIPT_MISSING"
    );
    let (fixture, _) = fixture_v5_active();
    fs::remove_file(fixture.path.join(TASK_EXECUTION_SOURCE_REL))
        .expect("remove owned task catalog");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V5_TASK_SOURCE_MISSING"
    );
    let (fixture, _) = fixture_v5_active();
    let mut altered_task = task_catalog_bytes();
    altered_task.push(b'\n');
    fs::write(fixture.path.join(TASK_EXECUTION_SOURCE_REL), altered_task)
        .expect("alter owned task catalog");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "TASK_ANCHOR_MISMATCH"
    );
    let (fixture, _) = fixture_v5_active();
    fs::remove_file(fixture.path.join(RECEIPT_V5_REL)).expect("remove owned v5");
    fs::create_dir(fixture.path.join(RECEIPT_V5_REL)).expect("create v5 directory");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V5_RECEIPT_NOT_REGULAR"
    );
    let mut invalid_v5 = v5_value();
    invalid_v5["version"] = json!(4);
    let fixture = FixtureRoot::new();
    write_fixture_v5(
        &fixture,
        Some(&invalid_v5),
        Some(&task_catalog_bytes()),
        &scheduling_catalog_bytes(),
        &catalog_bytes(),
        &receipt_v4_bytes(),
        &receipt_v3_bytes(),
        &receipt_v2_bytes(),
        &receipt_bytes(),
    );
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V5_ROOT_INVALID"
    );
    let fixture = FixtureRoot::new();
    write_fixture_v5(
        &fixture,
        None,
        Some(&task_catalog_bytes()),
        &scheduling_catalog_bytes(),
        &catalog_bytes(),
        &receipt_v4_bytes(),
        &receipt_v3_bytes(),
        &receipt_v2_bytes(),
        &receipt_bytes(),
    );
    let active = load_active_receipt(&fixture.path)
        .expect("v5-absent fixture loads")
        .expect("v4 becomes active");
    assert_eq!(active.version, 4);
    assert_eq!(active.files.len(), 10);
}

#[cfg(windows)]
#[test]
fn active_receipt_junctions_are_reparse_rejected_without_fallback() {
    use std::os::windows::fs::MetadataExt;

    let (fixture, _) = fixture_v4_active();
    let v4_link = fixture.path.join(RECEIPT_V4_REL);
    fs::remove_file(&v4_link).expect("remove owned v4 before junction");
    let v4_target = FixtureRoot::new();
    let v4_target_ledger = v4_target.path.join("imports.v4.json");
    fs::write(&v4_target_ledger, receipt_v4_bytes()).expect("write target v4 bytes");
    fs::write(v4_target.path.join("marker.txt"), b"owned-v4-target")
        .expect("write target v4 marker");
    let before_marker = fs::read(v4_target.path.join("marker.txt")).expect("read marker");
    let (native, stdout, stderr) = create_test_junction(&v4_link, &v4_target.path);
    let mut case = JunctionCase::new(v4_link.clone(), fixture, v4_target, v4_link.exists());
    println!(
        "JUNCTION v4 native={native} stdout={stdout:?} stderr={stderr:?} link={} target={}",
        v4_link.display(),
        case.target_path().display()
    );
    assert_eq!(native, 0, "mklink /J v4 failed: {stderr}");
    let attr = fs::symlink_metadata(&v4_link)
        .expect("junction metadata")
        .file_attributes();
    assert_ne!(attr & 0x400, 0, "v4 junction is not marked reparse");
    assert_eq!(
        load_active_receipt(case.root_path()).unwrap_err().code,
        "UNSAFE_REPARSE_OR_SYMLINK"
    );
    let (exit, status) = run_status_at(case.root_path());
    assert_eq!(exit, 2);
    assert_eq!(status["findings"][0]["code"], "UNSAFE_REPARSE_OR_SYMLINK");
    assert_eq!(
        fs::read(case.target_path().join("marker.txt")).expect("target marker after"),
        before_marker
    );
    assert_eq!(
        fs::read(&v4_target_ledger).expect("target ledger after"),
        receipt_v4_bytes()
    );
    assert!(case.unlink(), "v4 junction unlink failed");
    assert!(!v4_link.exists(), "v4 junction remains after unlink");
    case.cleanup();

    let (fixture, _) = fixture_v5_active();
    let v5_link = fixture.path.join(RECEIPT_V5_REL);
    fs::remove_file(&v5_link).expect("remove owned v5 before junction");
    let v5_target = FixtureRoot::new();
    let v5_target_ledger = v5_target.path.join("imports.v5.json");
    fs::write(
        &v5_target_ledger,
        serde_json::to_vec(&v5_value()).expect("v5 json"),
    )
    .expect("write target v5 bytes");
    fs::write(v5_target.path.join("marker.txt"), b"owned-v5-target")
        .expect("write target v5 marker");
    let before_marker = fs::read(v5_target.path.join("marker.txt")).expect("read v5 marker");
    let (native, stdout, stderr) = create_test_junction(&v5_link, &v5_target.path);
    let mut case = JunctionCase::new(v5_link.clone(), fixture, v5_target, v5_link.exists());
    println!(
        "JUNCTION v5 native={native} stdout={stdout:?} stderr={stderr:?} link={} target={}",
        v5_link.display(),
        case.target_path().display()
    );
    assert_eq!(native, 0, "mklink /J v5 failed: {stderr}");
    let attr = fs::symlink_metadata(&v5_link)
        .expect("v5 junction metadata")
        .file_attributes();
    assert_ne!(attr & 0x400, 0, "v5 junction is not marked reparse");
    assert_eq!(
        load_active_receipt(case.root_path()).unwrap_err().code,
        "UNSAFE_REPARSE_OR_SYMLINK"
    );
    let (exit, status) = run_status_at(case.root_path());
    assert_eq!(exit, 2);
    assert_eq!(status["findings"][0]["code"], "UNSAFE_REPARSE_OR_SYMLINK");
    assert_eq!(
        fs::read(case.target_path().join("marker.txt")).expect("v5 target marker after"),
        before_marker
    );
    assert_eq!(
        fs::read(&v5_target_ledger).expect("v5 target ledger after"),
        serde_json::to_vec(&v5_value()).expect("v5 json after")
    );
    assert!(case.unlink(), "v5 junction unlink failed");
    assert!(!v5_link.exists(), "v5 junction remains after unlink");
    case.cleanup();
}

#[test]
fn v5_rejects_singular_plural_null_and_combined_shapes() {
    let mut value = v5_value();
    let method = v5_entry_mut(&mut value, "PEXEC_SYSTEM")["methods"][0]["method_identity"].clone();
    v5_entry_mut(&mut value, "PEXEC_SYSTEM")["method_identity"] = method;
    assert_eq!(parse_v5_value(value).unwrap_err().code, "V5_SCHEMA_INVALID");

    let mut value = v5_value();
    v5_entry_mut(&mut value, "PEXEC_SYSTEM")["methods"] = Value::Null;
    assert_eq!(parse_v5_value(value).unwrap_err().code, "V5_SCHEMA_INVALID");

    let mut value = v5_value();
    v5_entry_mut(&mut value, "E18")["methods"] = json!([]);
    assert_eq!(parse_v5_value(value).unwrap_err().code, "V5_SCHEMA_INVALID");

    let mut value = v5_value();
    let event_method = v5_entry_mut(&mut value, "E18")["method_identity"].clone();
    let manifest = manifest_value();
    let event_record = manifest_record_value(&manifest, "E18");
    v5_entry_mut(&mut value, "E18")
        .as_object_mut()
        .expect("E18 object")
        .remove("method_identity");
    v5_entry_mut(&mut value, "E18")["methods"] = json!([{
        "method_identity": event_method,
        "evidence_status": event_record["evidence_status"].clone(),
        "source_status": event_record["source_status"].clone()
    }]);
    v5_entry_mut(&mut value, "E18")["evidence_status"] = event_record["evidence_status"].clone();
    v5_entry_mut(&mut value, "E18")["source_status"] = event_record["source_status"].clone();
    assert_eq!(
        parse_v5_value(value).unwrap_err().code,
        "V5_ENTRY_SHAPE_INVALID"
    );
}

#[test]
fn approved_batch_is_exactly_three_records() {
    assert_eq!(REQUIRED_IMPORT_IDS, ["E13", "E17", "E18"]);
}

#[test]
fn performance_catalog_accepts_exact_two_records_and_six_methods() {
    let bytes = catalog_bytes();
    assert_eq!(bytes.len(), 17385);
    assert_eq!(
        re_corpus::sha256_hex(&bytes),
        re_corpus::PERFORMANCE_SOURCE_V1_SHA256
    );
    let catalog = performance_catalog();
    assert_eq!(catalog.schema, "wuwa-reconstruction-performance-source-v1");
    assert_eq!(catalog.version, 1);
    assert_eq!(
        catalog.base_manifest_sha256,
        re_corpus::BASELINE_MANIFEST_SHA256
    );
    assert_eq!(catalog.records.len(), 2);
    for (id, type_def, method_defs, rva) in [
        (
            "PTIMER",
            "0x02000BEC",
            vec!["0x06003267", "0x06003269"],
            vec!["0x226FC", "0x22898"],
        ),
        (
            "PTIMER_INSTANCE",
            "0x02000BED",
            vec!["0x06003271", "0x0600327E", "0x0600326B", "0x0600327F"],
            vec!["0x22AB8", "0x23140", "0x228C4", "0x23224"],
        ),
    ] {
        let record = catalog
            .records
            .iter()
            .find(|record| record.id == id)
            .expect("catalog record");
        assert_eq!(record.type_def.as_deref(), Some(type_def));
        assert_eq!(record.authority_id, MANAGED_AUTHORITY_ID);
        assert!(record.methods.historical.is_empty());
        assert!(matches!(
            record.evidence_status,
            re_corpus::EvidenceStatus::DirectCilFull
        ));
        assert!(matches!(
            record.source_status,
            re_corpus::SourceStatus::ReconstructedFromDirectCil
        ));
        assert!(matches!(
            record.curation_status,
            re_corpus::CurationStatus::NeedsEditorialReview
        ));
        assert!(matches!(
            record.export_status,
            re_corpus::ExportStatus::PublicationReviewRequired
        ));
        assert_eq!(record.methods.current.len(), method_defs.len());
        for (method, method_def) in record.methods.current.iter().zip(method_defs) {
            assert_eq!(method.method_def, method_def);
            assert_eq!(method.symbol_key.type_def, type_def);
            assert!(matches!(
                method.evidence_status,
                re_corpus::EvidenceStatus::DirectCilFull
            ));
            assert!(matches!(
                method.source_status,
                re_corpus::SourceStatus::ReconstructedFromDirectCil
            ));
        }
        assert_eq!(
            record
                .methods
                .current
                .iter()
                .map(|method| method.rva.clone().expect("rva"))
                .collect::<Vec<_>>(),
            rva
        );
    }
}

#[test]
fn performance_catalog_rejects_self_claimed_status_identity_and_shape() {
    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["original_sha256"] = json!("f".repeat(64));
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_SOURCE_IDENTITY_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["proposed_path"] =
        json!("src/managed/cs-e3fc67839851/timer/Other.cs");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_DESTINATION_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["typeDef"] = json!("0x02000BED");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_AUTHORITY_IDENTITY_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["evidence_status"] = json!("SOURCE_ONLY");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_STATUS_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["curation_status"] = json!("CURATED_READY");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_STATUS_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["export_status"] = json!("INTERNAL_ONLY");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_STATUS_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["validation"]["compilation"] = json!("PASS");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_VALIDATION_STATUS_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["evidence_references"][1]["artifact_sha256"] =
        json!("f".repeat(64));
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_EVIDENCE_REFERENCE_INVALID"
    );

    let mut value = catalog_value();
    let symbol =
        catalog_record_value(&value, "PTIMER")["methods"]["current"][0]["symbol_key"].clone();
    catalog_record_mut(&mut value, "PTIMER")["methods"]["historical"] = json!([{
        "symbol_key": symbol,
        "name": "Do",
        "coverage": "SOURCE_LIKE_PROJECTION"
    }]);
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_HISTORICAL_NOT_EMPTY"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["methods"]["current"][0]["evidence_status"] =
        json!("DIRECT_CIL_PARTIAL");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_METHOD_CORRESPONDENCE_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["methods"]["current"][0]["rva"] = json!("0x226FD");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_METHOD_CORRESPONDENCE_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["methods"]["current"]
        .as_array_mut()
        .expect("PTIMER methods")
        .pop();
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_METHOD_SET_INVALID"
    );

    let mut value = catalog_value();
    let mut extra = catalog_record_value(&value, "PTIMER")["methods"]["current"][0].clone();
    extra["methodDef"] = json!("0x060032FF");
    extra["symbol_key"]["methodDef"] = json!("0x060032FF");
    catalog_record_mut(&mut value, "PTIMER")["methods"]["current"]
        .as_array_mut()
        .expect("PTIMER methods")
        .push(extra);
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_METHOD_SET_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER_INSTANCE")["id"] = json!("PTIMER");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_RECORD_ID_INVALID"
    );

    let mut value = catalog_value();
    value["records"]
        .as_array_mut()
        .expect("catalog records")
        .pop();
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_RECORD_SET_INVALID"
    );

    let mut value = catalog_value();
    value["unexpected"] = Value::Bool(true);
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_SCHEMA_INVALID"
    );
}

#[test]
fn catalog_and_v3_reject_duplicate_keys_missing_fields_and_unknown_enums() {
    assert_eq!(
        parse_performance_source_bytes(br#"{"schema":"a","schema":"b"}"#, &manifest())
            .unwrap_err()
            .code,
        "DUPLICATE_JSON_KEY"
    );
    assert_eq!(
        parse_receipt_v3_bytes(
            br#"{"schema":"a","schema":"b"}"#,
            &manifest(),
            &historical_receipt(),
            &historical_receipt_v2(),
            &performance_catalog(),
        )
        .unwrap_err()
        .code,
        "DUPLICATE_JSON_KEY"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")
        .as_object_mut()
        .expect("PTIMER object")
        .remove("scope");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_SCHEMA_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["evidence_status"] = json!("NOPE");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "CATALOG_SCHEMA_INVALID"
    );

    let mut value = catalog_value();
    catalog_record_mut(&mut value, "PTIMER")["evidence_references"][1]["locator"] = json!("");
    assert_eq!(
        parse_catalog_value(value).unwrap_err().code,
        "EMPTY_EVIDENCE_LOCATOR"
    );

    let mut value = v3_value();
    value
        .as_object_mut()
        .expect("v3 object")
        .remove("performance_source_sha256");
    assert_eq!(parse_v3_value(value).unwrap_err().code, "V3_SCHEMA_INVALID");
}

#[test]
fn v3_accepts_exact_seven_entries_and_maps_eighteen_methods() {
    let parsed = parse_v3_value(v3_value()).expect("accepted v3 fixture");
    assert_eq!(parsed.imports.len(), 7);
    let mut files = 0;
    let mut methods = 0;
    for entry in &parsed.imports {
        files += 1;
        match entry {
            ImportEntryV2::Event(_) => methods += 1,
            ImportEntryV2::Mapped(entry) => {
                methods += entry.methods.len();
                let expected = match entry.id.as_str() {
                    "D" => 8,
                    "T" => 1,
                    "PTIMER" => 2,
                    "PTIMER_INSTANCE" => 4,
                    other => panic!("unexpected mapped id {other}"),
                };
                assert_eq!(entry.methods.len(), expected);
            }
        }
    }
    assert_eq!(files, 7);
    assert_eq!(methods, 18);
}

#[test]
fn v3_fixture_loader_activates_all_seven_and_reports_combined_counts() {
    let (fixture, active) = fixture_v3_active();
    assert_eq!(active.version, 3);
    assert_eq!(active.ledger_path, RECEIPT_V3_REL);
    assert_eq!(active.files.len(), 7);
    assert_eq!(
        active
            .files
            .iter()
            .map(|file| file.methods.len())
            .sum::<usize>(),
        18
    );
    assert_eq!(
        active
            .files
            .iter()
            .filter(|file| file.id == "PTIMER")
            .count(),
        1
    );
    assert_eq!(
        active
            .files
            .iter()
            .filter(|file| file.id == "PTIMER_INSTANCE")
            .count(),
        1
    );

    let (exit, output) = run_status_at(&fixture.path);
    assert_eq!(exit, 1);
    assert_eq!(output["counts"]["historical_entries"], 18);
    assert_eq!(output["counts"]["current_entries"], 24);
    assert_eq!(output["counts"]["unique_qualified_symbols"], 20);
    assert_eq!(output["counts"]["planned"], 8);
    assert_eq!(output["counts"]["imported"], 7);
    assert_eq!(output["counts"]["imported_file_count"], 7);
    assert_eq!(output["counts"]["imported_qualified_methods"], 18);
    assert_eq!(output["counts"]["missing_imports"], 1);
    assert_eq!(output["active_ledger"]["path"], RECEIPT_V3_REL);
    assert_eq!(output["active_ledger"]["version"], 3);
    let records = output["records"].as_array().expect("records");
    assert_eq!(records.len(), 13);
    for id in ["PTIMER", "PTIMER_INSTANCE"] {
        let record = records
            .iter()
            .find(|record| record["id"] == id)
            .expect("new record in status");
        assert_eq!(record["import_status"], "IMPORTED");
    }
    let p = records
        .iter()
        .find(|record| record["id"] == "P")
        .expect("P record");
    assert_eq!(p["import_status"], "NOT_IMPORTED");
    let findings = output["findings"].as_array().expect("findings");
    assert_eq!(findings.len(), 7);
    assert!(
        findings
            .iter()
            .all(|finding| finding["code"] == "MISSING_IMPORTED_FILE")
    );
}

#[test]
fn v3_fixture_lookup_resolves_both_new_types_and_methods() {
    let (fixture, _) = fixture_v3_active();
    for (id, type_def, method_def) in [
        ("PTIMER", "0x02000BEC", "0x06003267"),
        ("PTIMER", "0x02000BEC", "0x06003269"),
        ("PTIMER_INSTANCE", "0x02000BED", "0x06003271"),
        ("PTIMER_INSTANCE", "0x02000BED", "0x0600327E"),
        ("PTIMER_INSTANCE", "0x02000BED", "0x0600326B"),
        ("PTIMER_INSTANCE", "0x02000BED", "0x0600327F"),
    ] {
        let (exit, output) = run_lookup_at(&fixture.path, MANAGED_AUTHORITY_ID, method_def);
        assert_eq!(exit, 0, "lookup {method_def}");
        assert_eq!(output["counts"]["imported"], 7);
        assert_eq!(output["counts"]["imported_file_count"], 7);
        assert_eq!(output["counts"]["imported_qualified_methods"], 18);
        assert_eq!(output["active_ledger"]["path"], RECEIPT_V3_REL);
        assert_eq!(output["active_ledger"]["version"], 3);
        let matches = output["matches"].as_array().expect("matches");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0]["id"], id);
        assert_eq!(matches[0]["typeDef"], type_def);
        let current = matches[0]["current"].as_array().expect("current");
        assert_eq!(current.len(), 1);
        assert_eq!(current[0]["methodDef"], method_def);
        assert_eq!(output["ambiguous"].as_array().expect("ambiguous").len(), 1);
    }
    let (exit, output) = run_lookup_at(&fixture.path, MANAGED_AUTHORITY_ID, "0x06003268");
    assert_eq!(exit, 1);
    assert_eq!(output["findings"][0]["code"], "NOT_FOUND");
}

#[test]
fn v3_rejects_root_old_entry_and_new_record_drift() {
    let mut value = v3_value();
    v3_entry_mut(&mut value, "E18")["imported_sha256"] = json!("f".repeat(64));
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_HISTORICAL_ENTRY_MISMATCH"
    );

    let mut value = v3_value();
    v3_entry_mut(&mut value, "D")["implementation_scope"] = json!("drifted scope");
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_HISTORICAL_ENTRY_MISMATCH"
    );

    let mut value = v3_value();
    v3_entry_mut(&mut value, "PTIMER")["implementation_scope"] = json!("drifted scope");
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_ENTRY_CORRESPONDENCE_INVALID"
    );

    let mut value = v3_value();
    v3_entry_mut(&mut value, "PTIMER")["methods"][0]["evidence_status"] =
        json!("DIRECT_CIL_PARTIAL");
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_ENTRY_CORRESPONDENCE_INVALID"
    );

    let mut value = v3_value();
    value["previous_receipt_sha256"] = json!("f".repeat(64));
    assert_eq!(parse_v3_value(value).unwrap_err().code, "V3_ROOT_INVALID");

    let mut value = v3_value();
    value["performance_source_sha256"] = json!("f".repeat(64));
    assert_eq!(parse_v3_value(value).unwrap_err().code, "V3_ROOT_INVALID");

    let mut value = v3_value();
    value["unexpected"] = Value::Bool(true);
    assert_eq!(parse_v3_value(value).unwrap_err().code, "V3_SCHEMA_INVALID");
}

#[test]
fn v3_rejects_duplicate_missing_extra_methods_ids_and_paths() {
    let mut value = v3_value();
    let duplicate_method = v3_entry_mut(&mut value, "PTIMER")["methods"][0].clone();
    v3_entry_mut(&mut value, "PTIMER")["methods"]
        .as_array_mut()
        .expect("PTIMER methods")
        .push(duplicate_method);
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_DUPLICATE_QUALIFIED_METHOD"
    );

    let mut value = v3_value();
    v3_entry_mut(&mut value, "PTIMER")["methods"]
        .as_array_mut()
        .expect("PTIMER methods")
        .pop();
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_ENTRY_CORRESPONDENCE_INVALID"
    );

    let mut value = v3_value();
    let mut extra_method = v3_entry_mut(&mut value, "PTIMER")["methods"][0].clone();
    extra_method["method_identity"]["methodDef"] = json!("0x060032FF");
    v3_entry_mut(&mut value, "PTIMER")["methods"]
        .as_array_mut()
        .expect("PTIMER methods")
        .push(extra_method);
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_ENTRY_CORRESPONDENCE_INVALID"
    );

    let mut value = v3_value();
    v3_entry_mut(&mut value, "PTIMER_INSTANCE")["id"] = json!("PTIMER");
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_DUPLICATE_IMPORT_ID"
    );

    let mut value = v3_value();
    v3_entry_mut(&mut value, "PTIMER")["id"] = json!("P");
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_UNAUTHORIZED_IMPORT_ID"
    );

    let mut value = v3_value();
    value["imports"]
        .as_array_mut()
        .expect("v3 imports")
        .retain(|entry| entry["id"] != "PTIMER");
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_IMPORT_ID_SET_INVALID"
    );

    let mut value = v3_value();
    let instance_path = v3_entry_mut(&mut value, "PTIMER_INSTANCE")["imported_path"].clone();
    v3_entry_mut(&mut value, "PTIMER")["imported_path"] = Value::String(
        instance_path
            .as_str()
            .expect("instance path")
            .to_ascii_uppercase(),
    );
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_DESTINATION_CASEFOLD_COLLISION"
    );

    let mut value = v3_value();
    let instance_source = v3_entry_mut(&mut value, "PTIMER_INSTANCE")["original_path"].clone();
    v3_entry_mut(&mut value, "PTIMER")["original_path"] = instance_source;
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_SOURCE_PATH_COLLISION"
    );

    let mut value = v3_value();
    v3_entry_mut(&mut value, "PTIMER")["imported_path"] = json!("../escape.cs");
    assert_eq!(parse_v3_value(value).unwrap_err().code, "V3_PATH_INVALID");

    let mut value = v3_value();
    v3_entry_mut(&mut value, "PTIMER")["original_sha256"] = json!("A".repeat(64));
    assert_eq!(parse_v3_value(value).unwrap_err().code, "V3_HASH_INVALID");

    let mut value = v3_value();
    v3_entry_mut(&mut value, "PTIMER")["authority_id"] = json!("bad authority!");
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_AUTHORITY_INVALID"
    );

    let mut value = v3_value();
    v3_entry_mut(&mut value, "PTIMER")["methods"][0]["method_identity"]["mvid"] =
        json!("2D6D37CB-76E5-4075-8220-3EECA41471E3");
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_METHOD_IDENTITY_INVALID"
    );
}

#[test]
fn v3_rejects_singular_plural_null_and_combined_shapes() {
    let mut value = v3_value();
    let method = v3_entry_mut(&mut value, "PTIMER")["methods"][0]["method_identity"].clone();
    v3_entry_mut(&mut value, "PTIMER")["method_identity"] = method;
    assert_eq!(parse_v3_value(value).unwrap_err().code, "V3_SCHEMA_INVALID");

    let mut value = v3_value();
    v3_entry_mut(&mut value, "PTIMER")["methods"] = Value::Null;
    assert_eq!(parse_v3_value(value).unwrap_err().code, "V3_SCHEMA_INVALID");

    let mut value = v3_value();
    v3_entry_mut(&mut value, "E18")["methods"] = json!([]);
    assert_eq!(parse_v3_value(value).unwrap_err().code, "V3_SCHEMA_INVALID");

    let mut value = v3_value();
    let event_method = v3_entry_mut(&mut value, "E18")["method_identity"].clone();
    let manifest = manifest_value();
    let event_record = manifest_record_value(&manifest, "E18");
    v3_entry_mut(&mut value, "E18")
        .as_object_mut()
        .expect("E18 object")
        .remove("method_identity");
    v3_entry_mut(&mut value, "E18")["methods"] = json!([{
        "method_identity": event_method,
        "evidence_status": event_record["evidence_status"].clone(),
        "source_status": event_record["source_status"].clone()
    }]);
    v3_entry_mut(&mut value, "E18")["evidence_status"] = event_record["evidence_status"].clone();
    v3_entry_mut(&mut value, "E18")["source_status"] = event_record["source_status"].clone();
    assert_eq!(
        parse_v3_value(value).unwrap_err().code,
        "V3_ENTRY_SHAPE_INVALID"
    );

    let mut value = v3_value();
    v3_entry_mut(&mut value, "E18")["methods"] = Value::Null;
    assert_eq!(parse_v3_value(value).unwrap_err().code, "V3_SCHEMA_INVALID");
}

#[test]
fn v3_loader_fails_closed_on_invalid_or_missing_predecessors_and_catalog() {
    let v3 = v3_value();
    let catalog = catalog_bytes();
    let v1 = receipt_bytes();
    let v2 = receipt_v2_bytes();

    let fixture = FixtureRoot::new();
    write_fixture_v3(&fixture, Some(&v3), Some(&catalog), &v2, &v1);
    fs::remove_file(fixture.path.join(RECEIPT_V2_REL)).expect("remove owned v2");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V3_PREVIOUS_RECEIPT_MISSING"
    );

    let fixture = FixtureRoot::new();
    write_fixture_v3(&fixture, Some(&v3), Some(&catalog), &v2, &v1);
    fs::remove_file(fixture.path.join(RECEIPT_REL)).expect("remove owned v1");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V3_HISTORICAL_RECEIPT_MISSING"
    );

    let fixture = FixtureRoot::new();
    write_fixture_v3(&fixture, Some(&v3), None, &v2, &v1);
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V3_PERFORMANCE_SOURCE_MISSING"
    );

    let fixture = FixtureRoot::new();
    write_fixture_v3(&fixture, Some(&v3), None, &v2, &v1);
    fs::create_dir(fixture.path.join(PERFORMANCE_SOURCE_REL)).expect("create catalog directory");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V3_PERFORMANCE_SOURCE_NOT_REGULAR"
    );

    let fixture = FixtureRoot::new();
    let mut altered_v2 = v2.clone();
    altered_v2.push(b'\n');
    write_fixture_v3(&fixture, Some(&v3), Some(&catalog), &altered_v2, &v1);
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V3_PREVIOUS_RECEIPT_HASH_MISMATCH"
    );

    let fixture = FixtureRoot::new();
    let mut altered_v1 = v1.clone();
    altered_v1.push(b'\n');
    write_fixture_v3(&fixture, Some(&v3), Some(&catalog), &v2, &altered_v1);
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V3_HISTORICAL_RECEIPT_HASH_MISMATCH"
    );

    let fixture = FixtureRoot::new();
    let mut altered_catalog = catalog.clone();
    altered_catalog.push(b'\n');
    write_fixture_v3(&fixture, Some(&v3), Some(&altered_catalog), &v2, &v1);
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "CATALOG_ANCHOR_MISMATCH"
    );

    let fixture = FixtureRoot::new();
    write_fixture_v3(&fixture, None, Some(&catalog), &v2, &v1);
    fs::create_dir(fixture.path.join(RECEIPT_V3_REL)).expect("create v3 directory");
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V3_RECEIPT_NOT_REGULAR"
    );

    let mut invalid_v3 = v3_value();
    invalid_v3["version"] = json!(2);
    let fixture = FixtureRoot::new();
    write_fixture_v3(&fixture, Some(&invalid_v3), Some(&catalog), &v2, &v1);
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "V3_ROOT_INVALID"
    );

    let mut structurally_invalid_catalog = catalog_value();
    structurally_invalid_catalog["version"] = json!(2);
    let structurally_invalid_bytes =
        serde_json::to_vec(&structurally_invalid_catalog).expect("serialize altered catalog");
    let fixture = FixtureRoot::new();
    write_fixture_v3(
        &fixture,
        Some(&v3),
        Some(&structurally_invalid_bytes),
        &v2,
        &v1,
    );
    assert_eq!(
        load_active_receipt(&fixture.path).unwrap_err().code,
        "CATALOG_ROOT_INVALID"
    );
}

#[test]
fn v3_absent_preserves_v2_v1_and_empty_behavior() {
    let catalog = catalog_bytes();
    let v1 = receipt_bytes();
    let v2 = receipt_v2_bytes();

    let fixture = FixtureRoot::new();
    write_fixture_v3(&fixture, None, Some(&catalog), &v2, &v1);
    let active = load_active_receipt(&fixture.path)
        .expect("v3-absent fixture loads")
        .expect("v2 becomes active");
    assert_eq!(active.version, 2);
    assert_eq!(active.ledger_path, RECEIPT_V2_REL);

    let fixture = FixtureRoot::new();
    write_fixture_v3(&fixture, None, Some(b"not json"), &v2, &v1);
    let active = load_active_receipt(&fixture.path)
        .expect("v3-absent fixture ignores catalog")
        .expect("v2 becomes active");
    assert_eq!(active.version, 2);

    let fixture = FixtureRoot::new();
    let metadata = fixture.path.join("metadata");
    fs::create_dir_all(&metadata).expect("create fixture metadata");
    fs::write(metadata.join("migration.json"), manifest_bytes()).expect("write fixture manifest");
    assert!(
        load_active_receipt(&fixture.path)
            .expect("empty fixture loads")
            .is_none()
    );
}

#[test]
fn v3_fixture_verify_uses_eighteen_method_import_scope_without_promoting_externals() {
    let (fixture, _) = fixture_v3_active();
    let (exit, output) = run_verify_at(&fixture.path, None);
    assert_eq!(exit, 1);
    assert_eq!(output["active_ledger"]["path"], RECEIPT_V3_REL);
    assert_eq!(output["active_ledger"]["version"], 3);
    assert_eq!(output["counts"]["imported"], 7);
    assert_eq!(output["counts"]["imported_file_count"], 7);
    assert_eq!(output["counts"]["imported_qualified_methods"], 18);
    assert!(
        output["findings"]
            .as_array()
            .expect("findings")
            .iter()
            .any(|finding| finding["code"] == "REQUIRED_ORIGINALS_NOT_CHECKED")
    );
    let not_checked = output["not_checked"].as_array().expect("not_checked");
    for id in ["D", "PTIMER", "PTIMER_INSTANCE", "T"] {
        assert!(not_checked.iter().any(|item| {
            item["kind"] == "ORIGINAL_SOURCE" && item["id"] == id && item["status"] == "NOT_CHECKED"
        }));
    }
    assert!(
        not_checked
            .iter()
            .any(|item| item["kind"] == "EVIDENCE_REFERENCE" && item["status"] == "NOT_CHECKED")
    );
    let missing = output["missing"].as_array().expect("missing");
    assert_eq!(missing.len(), 7);
    assert!(
        missing
            .iter()
            .all(|item| item["kind"] == "IMPORTED_FILE" && item["status"] == "FAIL")
    );
    assert_eq!(
        output["findings"]
            .as_array()
            .expect("findings")
            .iter()
            .filter(|finding| finding["code"] == "MISSING_PATH")
            .count(),
        7
    );
}

fn cli_fixture(label: &str) -> PathBuf {
    let number = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/test-fixtures")
        .join(format!(
            "root-option-{label}-{}-{number}",
            std::process::id()
        ));
    fs::create_dir_all(&path).expect("create owned CLI fixture");
    path
}

fn run_binary(binary: &str, arguments: &[String], current_dir: &Path) -> (i32, Value) {
    let output = Command::new(binary)
        .args(arguments)
        .current_dir(current_dir)
        .output()
        .expect("spawn CLI binary");
    (
        output.status.code().unwrap_or(-1),
        serde_json::from_slice(&output.stdout).unwrap_or(Value::Null),
    )
}

#[test]
fn cli_explicit_root_works_from_foreign_cwd_and_wrong_root_never_falls_back() {
    let curated = curated_root();
    let curated_text = curated.to_string_lossy().to_string();
    let foreign = std::env::temp_dir();
    let (exit, output) = run_binary(
        env!("CARGO_BIN_EXE_re"),
        &[
            "status".to_string(),
            "--json".to_string(),
            "--curated-root".to_string(),
            curated_text.clone(),
        ],
        &foreign,
    );
    assert_eq!(exit, 0, "explicit root status: {output}");
    assert_eq!(output["ok"], true);

    let (exit, output) = run_binary(
        env!("CARGO_BIN_EXE_corpus-registry"),
        &[
            "source-status".to_string(),
            "--curated-root".to_string(),
            curated_text,
        ],
        &foreign,
    );
    assert_eq!(exit, 0, "explicit root source-status: {output}");

    let empty = cli_fixture("empty-root");
    let empty_text = empty.to_string_lossy().to_string();
    let (exit, output) = run_binary(
        env!("CARGO_BIN_EXE_re"),
        &[
            "status".to_string(),
            "--json".to_string(),
            "--curated-root".to_string(),
            empty_text.clone(),
        ],
        &foreign,
    );
    assert_eq!(exit, 2, "empty explicit root must fail: {output}");
    assert_eq!(output["ok"], false);

    let (exit, output) = run_binary(
        env!("CARGO_BIN_EXE_corpus-registry"),
        &[
            "source-status".to_string(),
            "--curated-root".to_string(),
            empty_text,
        ],
        &foreign,
    );
    assert_eq!(exit, 2, "empty explicit source root must fail: {output}");
    fs::remove_dir_all(&empty).expect("cleanup owned empty root fixture");
}

#[test]
fn cli_default_root_uses_nearest_metadata_ancestor_from_subdirectory() {
    let subdirectory = cli_fixture("ancestor cwd");
    let (exit, output) = run_binary(
        env!("CARGO_BIN_EXE_re"),
        &["status".to_string(), "--json".to_string()],
        &subdirectory,
    );
    assert_eq!(exit, 0, "ancestor-root status: {output}");
    assert_eq!(output["ok"], true);
    fs::remove_dir_all(&subdirectory).expect("cleanup owned ancestor fixture");
}

#[test]
fn cli_explicit_manifest_keeps_cwd_semantics_and_precedes_curated_default() {
    let curated = curated_root();
    let curated_text = curated.to_string_lossy().to_string();
    let manifest = curated.join("metadata/source-closure.v1.json");
    let manifest_text = manifest.to_string_lossy().to_string();
    let foreign = std::env::temp_dir();
    let (exit, output) = run_binary(
        env!("CARGO_BIN_EXE_corpus-registry"),
        &[
            "source-status".to_string(),
            "--curated-root".to_string(),
            curated_text.clone(),
            "--manifest".to_string(),
            manifest_text,
        ],
        &foreign,
    );
    assert_eq!(exit, 0, "explicit absolute manifest: {output}");

    let subdirectory = cli_fixture("relative manifest cwd");
    let (exit, output) = run_binary(
        env!("CARGO_BIN_EXE_corpus-registry"),
        &[
            "source-status".to_string(),
            "--curated-root".to_string(),
            curated_text,
            "--manifest".to_string(),
            "metadata/source-closure.v1.json".to_string(),
        ],
        &subdirectory,
    );
    assert_eq!(
        exit, 2,
        "relative explicit manifest must resolve from CWD, not curated root: {output}"
    );
    fs::remove_dir_all(&subdirectory).expect("cleanup owned relative manifest fixture");
}
