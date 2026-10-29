use re_corpus::bulk::apply_approved_cohort;
use std::env;
use std::path::Path;
use std::process::exit;

fn fail(message: &str) -> ! {
    let value = serde_json::json!({
        "command": "curate-batch",
        "ok": false,
        "exit": 2,
        "error": message
    });
    println!("{value}");
    exit(2);
}

fn main() {
    let mut args = env::args_os().skip(1);
    let Some(source_root) = args.next() else {
        fail("usage: curate-batch <sourceRoot> <curatedRoot> <cohort>");
    };
    let Some(curated_root) = args.next() else {
        fail("usage: curate-batch <sourceRoot> <curatedRoot> <cohort>");
    };
    let Some(cohort) = args.next() else {
        fail("usage: curate-batch <sourceRoot> <curatedRoot> <cohort>");
    };
    if args.next().is_some() {
        fail("usage: curate-batch <sourceRoot> <curatedRoot> <cohort>");
    }
    let source_root = match source_root.into_string() {
        Ok(value) if !value.is_empty() => value,
        _ => fail("sourceRoot must be a nonempty UTF-8 path"),
    };
    let curated_root = match curated_root.into_string() {
        Ok(value) if !value.is_empty() => value,
        _ => fail("curatedRoot must be a nonempty UTF-8 path"),
    };
    let cohort = match cohort.into_string() {
        Ok(value) if !value.is_empty() => value,
        _ => fail("cohort must be a nonempty UTF-8 value"),
    };
    match apply_approved_cohort(Path::new(&source_root), Path::new(&curated_root), &cohort) {
        Ok(receipt) => match serde_json::to_string(&receipt) {
            Ok(output) => {
                println!("{output}");
                exit(0);
            }
            Err(_) => fail("receipt serialization failed"),
        },
        Err(failure) => match serde_json::to_string(&failure) {
            Ok(output) => {
                println!("{output}");
                exit(1);
            }
            Err(_) => fail("failure serialization failed"),
        },
    }
}
