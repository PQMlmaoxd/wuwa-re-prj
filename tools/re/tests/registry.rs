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
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = base.join(format!(
            "{label}-{}-{nanos}-{}",
            std::process::id(),
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
    let code = output.status.code().unwrap_or(-1);
    let value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (code, value)
}

fn write_config(
    path: &Path,
    roots: &[(&str, &Path)],
    excludes: &[(&str, &str, &str)],
    directory_names: &[&str],
) {
    let roots = roots
        .iter()
        .map(|(id, root)| json!({"id": id, "path": text(root)}))
        .collect::<Vec<_>>();
    let excludes = excludes
        .iter()
        .map(|(root_id, rel, reason)| json!({"root_id": root_id, "path": rel, "reason": reason}))
        .collect::<Vec<_>>();
    let config = json!({
        "schema": "wuwa-reconstruction-corpus-registry-config-v1",
        "roots": roots,
        "exclude_paths": excludes,
        "exclude_directory_names": directory_names
    });
    fs::write(path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
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

fn read_artifacts(snapshot: &Path) -> Vec<Value> {
    let text = fs::read_to_string(snapshot.join("artifacts.jsonl")).unwrap();
    text.lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect()
}

fn artifact<'a>(rows: &'a [Value], path: &str) -> &'a Value {
    rows.iter()
        .find(|row| row["path"].as_str() == Some(path))
        .unwrap_or_else(|| panic!("artifact {path} should be present"))
}

fn error_codes(snapshot: &Path) -> Vec<String> {
    let errors: Vec<Value> =
        serde_json::from_slice(&fs::read(snapshot.join("errors.json")).unwrap()).unwrap();
    errors
        .iter()
        .map(|error| error["code"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn scan_hashes_known_bytes_and_empty_file_and_applies_exclusions() {
    let fixture = Fixture::new("scan-known");
    let root = fixture.path("root");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a.txt"), b"hello").unwrap();
    fs::write(root.join("empty.bin"), b"").unwrap();
    fs::create_dir(root.join("sub")).unwrap();
    fs::write(root.join("sub/b.bin"), [0u8, 1, 2, 3]).unwrap();
    fs::create_dir(root.join("skip")).unwrap();
    fs::write(root.join("skip/ignored.bin"), b"ignored").unwrap();
    fs::create_dir(root.join("__pycache__")).unwrap();
    fs::write(root.join("__pycache__/x.pyc"), b"bytecode").unwrap();
    fs::create_dir(root.join("task-root")).unwrap();
    fs::write(root.join("task-root/secret.bin"), b"secret").unwrap();

    let config = fixture.path("config.json");
    write_config(
        &config,
        &[("RESEARCH_ROOT", &root)],
        &[
            ("RESEARCH_ROOT", "skip", "TEST_EXCLUDE"),
            ("RESEARCH_ROOT", "task-root", "TASK_ROOT_RECURSION_GUARD"),
        ],
        &["__pycache__"],
    );
    let output = fixture.path("out-1");
    let (code, value) = scan(&config, &output);
    assert_eq!(code, 0, "scan should pass: {value}");
    assert_eq!(value["status"], "PASS");

    let rows = read_artifacts(&output);
    assert_eq!(rows.len(), 3);
    let a = artifact(&rows, "a.txt");
    assert_eq!(a["size"], 5);
    assert_eq!(
        a["sha256"].as_str().unwrap(),
        re_corpus::sha256_hex(b"hello")
    );
    assert_eq!(a["status"], "OK");
    assert_eq!(a["backup_coverage"], "UNKNOWN");
    assert_eq!(
        a["artifact_id"].as_str().unwrap(),
        re_corpus::sha256_hex(format!("RESEARCH_ROOT\0a.txt").as_bytes())
    );
    let empty = artifact(&rows, "empty.bin");
    assert_eq!(empty["size"], 0);
    assert_eq!(
        empty["sha256"].as_str().unwrap(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    let sub = artifact(&rows, "sub/b.bin");
    assert_eq!(sub["size"], 4);
    assert_eq!(
        sub["sha256"].as_str().unwrap(),
        re_corpus::sha256_hex(&[0u8, 1, 2, 3])
    );
    for row in &rows {
        let path = row["path"].as_str().unwrap();
        assert!(!path.contains('\\'));
        assert!(!path.contains(':'));
        assert!(!path.starts_with("skip/"));
        assert!(!path.starts_with("__pycache__/"));
    }

    let manifest: Value =
        serde_json::from_slice(&fs::read(output.join("snapshot.json")).unwrap()).unwrap();
    assert_eq!(manifest["artifact_entries"], 3);
    let artifacts_bytes = fs::read(output.join("artifacts.jsonl")).unwrap();
    assert_eq!(
        manifest["snapshot_id"].as_str().unwrap(),
        re_corpus::sha256_hex(&artifacts_bytes)
    );
    let exclusions = manifest["exclusions"].as_array().unwrap();
    let skip = exclusions
        .iter()
        .find(|entry| entry["path"].as_str() == Some("skip"))
        .unwrap();
    assert_eq!(skip["files"], 1);
    assert_eq!(skip["bytes"], 7);
    let pycache = exclusions
        .iter()
        .find(|entry| entry["path"].as_str() == Some("__pycache__"))
        .unwrap();
    assert_eq!(pycache["files"], 1);
    let task_root = exclusions
        .iter()
        .find(|entry| entry["path"].as_str() == Some("task-root"))
        .unwrap();
    assert_eq!(task_root["measured"], false);
    assert!(task_root["files"].is_null());
    assert!(error_codes(&output).is_empty());
}

#[test]
fn verify_detects_changed_missing_and_new_files() {
    let fixture = Fixture::new("verify");
    let root = fixture.path("root");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a.txt"), b"hello").unwrap();
    fs::create_dir(root.join("sub")).unwrap();
    fs::write(root.join("sub/b.bin"), b"bravo").unwrap();
    let config = fixture.path("config.json");
    write_config(&config, &[("RESEARCH_ROOT", &root)], &[], &[]);
    let output = fixture.path("out-1");
    assert_eq!(scan(&config, &output).0, 0);

    let (code, value) = verify(&config, &output);
    assert_eq!(code, 0, "unchanged snapshot should verify: {value}");
    assert_eq!(value["status"], "EXACT_MATCH");

    fs::write(root.join("a.txt"), b"world").unwrap();
    fs::remove_file(root.join("sub/b.bin")).unwrap();
    fs::write(root.join("new.bin"), b"new").unwrap();
    let (code, value) = verify(&config, &output);
    assert_eq!(code, 1, "changed corpus must not verify: {value}");
    assert_eq!(value["counts"]["changed"], 1);
    assert_eq!(value["counts"]["missing"], 1);
    assert_eq!(value["counts"]["new"], 1);
    let codes = value["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|finding| finding["code"].as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"CHANGED"));
    assert!(codes.contains(&"MISSING"));
    assert!(codes.contains(&"NEW"));
}

#[test]
fn repeated_scan_is_deterministic_except_execution_receipt() {
    let fixture = Fixture::new("deterministic");
    let root = fixture.path("root");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a.txt"), b"hello").unwrap();
    fs::write(root.join("b.bin"), b"bytes").unwrap();
    let config = fixture.path("config.json");
    write_config(&config, &[("RESEARCH_ROOT", &root)], &[], &[]);
    let out_one = fixture.path("out-1");
    let out_two = fixture.path("out-2");
    assert_eq!(scan(&config, &out_one).0, 0);
    assert_eq!(scan(&config, &out_two).0, 0);
    for name in ["artifacts.jsonl", "snapshot.json", "errors.json"] {
        assert_eq!(
            fs::read(out_one.join(name)).unwrap(),
            fs::read(out_two.join(name)).unwrap(),
            "{name} must be byte-identical for the same input"
        );
    }
    let execution: Value =
        serde_json::from_slice(&fs::read(out_one.join("execution.json")).unwrap()).unwrap();
    assert_eq!(execution["command"], "scan");
    assert_eq!(execution["status"], "PASS");
}

#[test]
fn output_exists_is_rejected_without_overwrite() {
    let fixture = Fixture::new("output-exists");
    let root = fixture.path("root");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a.txt"), b"hello").unwrap();
    let config = fixture.path("config.json");
    write_config(&config, &[("RESEARCH_ROOT", &root)], &[], &[]);
    let output = fixture.path("out");
    fs::create_dir(&output).unwrap();
    fs::write(output.join("sentinel.txt"), b"keep").unwrap();
    let (code, value) = scan(&config, &output);
    assert_eq!(code, 2, "existing output must be refused: {value}");
    assert_eq!(value["findings"][0]["code"], "OUTPUT_ALREADY_EXISTS");
    assert_eq!(fs::read(output.join("sentinel.txt")).unwrap(), b"keep");
    assert!(!output.join("artifacts.jsonl").exists());
}

#[test]
fn unsafe_configs_are_rejected_without_creating_output() {
    let fixture = Fixture::new("unsafe-config");
    let root = fixture.path("root");
    let sub = root.join("sub");
    let other = fixture.path("other");
    fs::create_dir(&root).unwrap();
    fs::create_dir(&sub).unwrap();
    fs::create_dir(&other).unwrap();
    let schema = "wuwa-reconstruction-corpus-registry-config-v1";

    let cases: Vec<(&str, Value)> = vec![
        (
            "unknown-root",
            json!({"schema": schema, "roots": [{"id": "OTHER_ROOT", "path": text(&root)}]}),
        ),
        (
            "duplicate-root-id",
            json!({"schema": schema, "roots": [
                {"id": "RESEARCH_ROOT", "path": text(&root)},
                {"id": "RESEARCH_ROOT", "path": text(&other)}
            ]}),
        ),
        (
            "same-path-two-ids",
            json!({"schema": schema, "roots": [
                {"id": "RESEARCH_ROOT", "path": text(&root)},
                {"id": "CURATED_ROOT", "path": text(&root)}
            ]}),
        ),
        (
            "reversed-overlap",
            json!({"schema": schema, "roots": [
                {"id": "RESEARCH_ROOT", "path": text(&sub)},
                {"id": "CURATED_ROOT", "path": text(&root)}
            ]}),
        ),
        (
            "casefold-overlap",
            json!({"schema": schema, "roots": [
                {"id": "RESEARCH_ROOT", "path": text(&root)},
                {"id": "CURATED_ROOT", "path": text(&root).to_uppercase()}
            ]}),
        ),
        (
            "unsafe-exclude",
            json!({"schema": schema, "roots": [{"id": "RESEARCH_ROOT", "path": text(&root)}],
                "exclude_paths": [{"root_id": "RESEARCH_ROOT", "path": "../escape", "reason": "TEST"}]}),
        ),
        (
            "bad-schema",
            json!({"schema": "not-the-schema", "roots": [{"id": "RESEARCH_ROOT", "path": text(&root)}]}),
        ),
    ];

    for (label, config_value) in cases {
        let config_path = fixture.path(&format!("config-{label}.json"));
        fs::write(
            &config_path,
            serde_json::to_vec_pretty(&config_value).unwrap(),
        )
        .unwrap();
        let output = fixture.path(&format!("out-{label}"));
        let (code, value) = scan(&config_path, &output);
        assert_eq!(code, 2, "{label} must be rejected: {value}");
        assert!(!output.exists(), "{label} must not create output");
    }

    let config_path = fixture.path("config-valid.json");
    write_config(&config_path, &[("RESEARCH_ROOT", &root)], &[], &[]);
    let outside = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/test-fixtures")
        .join(format!(
            "outside-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
    let (code, value) = scan(&config_path, &outside);
    assert_eq!(
        code, 2,
        "output outside the config directory must be rejected: {value}"
    );
    assert!(!outside.exists());
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
                "preserving junction fixture because unlink failed: {}",
                self.junction.display()
            );
        }
    }
}

#[cfg(windows)]
#[test]
fn reparse_junction_is_rejected_before_traversal() {
    let mut fixture = Fixture::new("junction");
    let root = fixture.path("root");
    let target = fixture.path("target-data");
    let junction = root.join("jump");
    fs::create_dir(&root).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(root.join("normal.txt"), b"normal").unwrap();
    fs::write(target.join("sentinel.txt"), b"sentinel").unwrap();

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
            "NOT_TESTED: mklink /J failed: stdout={:?} stderr={:?}",
            String::from_utf8_lossy(&created.stdout),
            String::from_utf8_lossy(&created.stderr)
        );
        return;
    }
    fixture.cleanup = false;
    let _guard = JunctionGuard {
        junction: junction.clone(),
        fixture: fixture.dir.clone(),
    };

    let before = fs::read(target.join("sentinel.txt")).unwrap();
    let config = fixture.path("config.json");
    write_config(&config, &[("RESEARCH_ROOT", &root)], &[], &[]);
    let output = fixture.path("out");
    let (code, value) = scan(&config, &output);
    assert_eq!(code, 1, "junction must make the scan incomplete: {value}");
    let rows = read_artifacts(&output);
    assert!(
        rows.iter()
            .any(|row| row["path"].as_str() == Some("normal.txt"))
    );
    assert!(
        rows.iter()
            .all(|row| !row["path"].as_str().unwrap().starts_with("jump/")),
        "junction target must not be traversed"
    );
    let errors = error_codes(&output);
    assert!(
        errors.iter().any(|code| code == "REPARSE_OBJECT_REJECTED"),
        "reparse rejection must be recorded: {errors:?}"
    );
    let after = fs::read(target.join("sentinel.txt")).unwrap();
    assert_eq!(before, after, "junction target bytes must remain unchanged");
}
