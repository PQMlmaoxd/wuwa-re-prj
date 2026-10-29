use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    dir: PathBuf,
    cleanup: bool,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/test-fixtures");
        fs::create_dir_all(&base).unwrap();
        let dir = base.join(format!(
            "{label}-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&dir).unwrap();
        Self { dir, cleanup: true }
    }
    fn path(&self, relative: &str) -> PathBuf {
        self.dir.join(relative)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.cleanup && !std::thread::panicking() {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }
}

fn text(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn run(args: &[String]) -> (i32, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_corpus-registry"))
        .args(args)
        .output()
        .expect("corpus-registry binary should run");
    (
        output.status.code().unwrap_or(-1),
        serde_json::from_slice(&output.stdout).unwrap_or(Value::Null),
    )
}

fn scan(config: &Path, output: &Path) -> (i32, Value) {
    run(&[
        "scan".to_string(),
        "--config".to_string(),
        text(config),
        "--output".to_string(),
        text(output),
    ])
}

fn verify(config: &Path, snapshot: &Path) -> (i32, Value) {
    run(&[
        "verify".to_string(),
        "--config".to_string(),
        text(config),
        "--snapshot".to_string(),
        text(snapshot),
    ])
}

fn copy_snapshot(source: &Path, destination: &Path) {
    fs::create_dir(destination).unwrap();
    for name in ["artifacts.jsonl", "snapshot.json"] {
        fs::copy(source.join(name), destination.join(name)).unwrap();
    }
}

fn rewrite_snapshot(
    snapshot: &Path,
    mutate_rows: impl FnOnce(&mut Vec<Value>),
    mutate_manifest: impl FnOnce(&mut Value),
) {
    let artifacts_path = snapshot.join("artifacts.jsonl");
    let mut rows = fs::read_to_string(&artifacts_path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    mutate_rows(&mut rows);
    let mut bytes = Vec::new();
    for row in &rows {
        bytes.extend_from_slice(serde_json::to_string(row).unwrap().as_bytes());
        bytes.push(b'\n');
    }
    fs::write(&artifacts_path, &bytes).unwrap();
    let sha = {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(&bytes);
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let manifest_path = snapshot.join("snapshot.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["snapshot_id"] = Value::String(sha);
    manifest["artifacts_bytes"] = Value::from(bytes.len() as u64);
    manifest["artifact_entries"] = Value::from(rows.len() as u64);
    mutate_manifest(&mut manifest);
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
}

fn finding_has(value: &Value, code: &str) -> bool {
    value["findings"]
        .as_array()
        .map(|rows| rows.iter().any(|row| row["code"] == code))
        .unwrap_or(false)
}

#[test]
fn verify_rejects_forged_kind_even_with_recomputed_snapshot_id() {
    let fixture = Fixture::new("forged-kind");
    let root = fixture.path("root");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a.rs"), b"hello").unwrap();
    let config = fixture.path("config.json");
    fs::write(
        &config,
        serde_json::to_vec(&json!({
            "schema": "wuwa-reconstruction-corpus-registry-config-v1",
            "roots": [{"id": "RESEARCH_ROOT", "path": text(&root)}],
            "exclude_paths": [],
            "exclude_directory_names": []
        }))
        .unwrap(),
    )
    .unwrap();
    let base = fixture.path("base");
    assert_eq!(scan(&config, &base).0, 0);
    let snapshot = fixture.path("forged");
    copy_snapshot(&base, &snapshot);
    rewrite_snapshot(
        &snapshot,
        |rows| {
            let row = rows.iter_mut().find(|row| row["path"] == "a.rs").unwrap();
            row["kind"] = Value::String("opaque".to_string());
        },
        |_| {},
    );
    let (code, value) = verify(&config, &snapshot);
    assert_eq!(code, 1, "forged derived field must fail verify: {value}");
    assert!(finding_has(&value, "SNAPSHOT_ROW_SEMANTIC_MISMATCH"));
}

#[test]
fn verify_rejects_forged_manifest_metadata() {
    let fixture = Fixture::new("forged-manifest");
    let root = fixture.path("root");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a.rs"), b"hello").unwrap();
    let config = fixture.path("config.json");
    fs::write(
        &config,
        serde_json::to_vec(&json!({
            "schema": "wuwa-reconstruction-corpus-registry-config-v1",
            "roots": [{"id": "RESEARCH_ROOT", "path": text(&root)}],
            "exclude_paths": [],
            "exclude_directory_names": []
        }))
        .unwrap(),
    )
    .unwrap();
    let base = fixture.path("base");
    assert_eq!(scan(&config, &base).0, 0);
    let snapshot = fixture.path("forged");
    copy_snapshot(&base, &snapshot);
    let manifest_path = snapshot.join("snapshot.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["observed"]["kinds"]["source"] = Value::from(999u64);
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let (code, value) = verify(&config, &snapshot);
    assert_eq!(
        code, 1,
        "forged manifest metadata must fail verify: {value}"
    );
    assert!(finding_has(&value, "SNAPSHOT_MANIFEST_SEMANTIC_MISMATCH"));
}

#[test]
fn verify_rejects_invalid_backup_coverage_and_unknown_row_fields() {
    let fixture = Fixture::new("forged-enums");
    let root = fixture.path("root");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a.txt"), b"hello").unwrap();
    let config = fixture.path("config.json");
    fs::write(
        &config,
        serde_json::to_vec(&json!({
            "schema": "wuwa-reconstruction-corpus-registry-config-v1",
            "roots": [{"id": "RESEARCH_ROOT", "path": text(&root)}],
            "exclude_paths": [],
            "exclude_directory_names": []
        }))
        .unwrap(),
    )
    .unwrap();
    let base = fixture.path("base");
    assert_eq!(scan(&config, &base).0, 0);

    let bad_backup = fixture.path("bad-backup");
    copy_snapshot(&base, &bad_backup);
    rewrite_snapshot(
        &bad_backup,
        |rows| {
            rows[0]["backup_coverage"] =
                Value::String("COPY_PRESENT_NOT_INDEPENDENT_BACKUP".to_string());
        },
        |_| {},
    );
    let (code, value) = verify(&config, &bad_backup);
    assert_eq!(code, 2, "invalid backup_coverage must fail closed: {value}");
    assert!(finding_has(&value, "SNAPSHOT_ROW_INVALID"));

    let unknown = fixture.path("unknown-field");
    copy_snapshot(&base, &unknown);
    rewrite_snapshot(
        &unknown,
        |rows| {
            rows[0]["injected"] = Value::Bool(true);
        },
        |_| {},
    );
    let (code, value) = verify(&config, &unknown);
    assert_eq!(code, 2, "unknown row field must fail closed: {value}");
    assert!(finding_has(&value, "SNAPSHOT_ARTIFACTS_INVALID"));

    let duplicate = fixture.path("duplicate");
    copy_snapshot(&base, &duplicate);
    rewrite_snapshot(
        &duplicate,
        |rows| {
            rows.push(rows[0].clone());
        },
        |_| {},
    );
    let (code, value) = verify(&config, &duplicate);
    assert_eq!(code, 1, "duplicate logical key must fail verify: {value}");
    assert!(
        finding_has(&value, "SNAPSHOT_DUPLICATE_KEY")
            || finding_has(&value, "NON_CANONICAL_ORDER")
            || finding_has(&value, "SNAPSHOT_ROW_SEMANTIC_MISMATCH")
    );
}

#[cfg(windows)]
struct JunctionGuard {
    junction: PathBuf,
    fixture: PathBuf,
}

#[cfg(windows)]
impl Drop for JunctionGuard {
    fn drop(&mut self) {
        if fs::remove_dir(&self.junction).is_ok() {
            let _ = fs::remove_dir_all(&self.fixture);
        } else {
            eprintln!(
                "preserving member-junction fixture because unlink failed: {}",
                self.junction.display()
            );
        }
    }
}

#[cfg(windows)]
#[test]
fn verify_rejects_snapshot_member_junction_before_reading_it() {
    let mut fixture = Fixture::new("member-junction");
    let root = fixture.path("root");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a.txt"), b"hello").unwrap();
    let config = fixture.path("config.json");
    fs::write(
        &config,
        serde_json::to_vec(&json!({
            "schema": "wuwa-reconstruction-corpus-registry-config-v1",
            "roots": [{"id": "RESEARCH_ROOT", "path": text(&root)}],
            "exclude_paths": [],
            "exclude_directory_names": []
        }))
        .unwrap(),
    )
    .unwrap();
    let base = fixture.path("base");
    assert_eq!(scan(&config, &base).0, 0);
    let snapshot = fixture.path("member-junction-snapshot");
    fs::create_dir(&snapshot).unwrap();
    fs::copy(base.join("snapshot.json"), snapshot.join("snapshot.json")).unwrap();
    let target = fixture.path("junction-target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("sentinel.txt"), b"sentinel").unwrap();
    let junction = snapshot.join("artifacts.jsonl");
    let created = Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            text(&junction).as_str(),
            text(&target).as_str(),
        ])
        .output()
        .expect("cmd should run");
    if !created.status.success() {
        eprintln!(
            "NOT_TESTED: mklink /J member failed: stdout={:?} stderr={:?}",
            String::from_utf8_lossy(&created.stdout),
            String::from_utf8_lossy(&created.stderr)
        );
        return;
    }
    // Fixture is cleaned by the guard only after the junction is unlinked.
    fixture.cleanup = false;
    let _guard = JunctionGuard {
        junction: junction.clone(),
        fixture: fixture.dir.clone(),
    };
    let before = fs::read(target.join("sentinel.txt")).unwrap();
    let (code, value) = verify(&config, &snapshot);
    assert_eq!(code, 2, "member junction must fail closed: {value}");
    assert!(finding_has(&value, "UNSAFE_REPARSE_OR_SYMLINK"));
    let after = fs::read(target.join("sentinel.txt")).unwrap();
    assert_eq!(before, after, "junction target bytes must be untouched");
}
