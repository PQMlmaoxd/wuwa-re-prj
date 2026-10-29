use std::env;
use std::panic::{AssertUnwindSafe, catch_unwind};

fn main() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        re_corpus::run_with_args(env::args_os().skip(1))
    }));
    let (exit, envelope) = result.unwrap_or_else(|_| {
        (
            2,
            serde_json::json!({
                "command": "unknown",
                "ok": false,
                "exit": 2,
                "counts": {},
                "records": [],
                "findings": [{"code": "EXECUTION_ERROR"}]
            }),
        )
    });
    match serde_json::to_string(&envelope) {
        Ok(output) => println!("{output}"),
        Err(_) => println!(
            "{}",
            "{\"command\":\"unknown\",\"ok\":false,\"exit\":2,\"counts\":{},\"records\":[],\"findings\":[{\"code\":\"OUTPUT_ERROR\"}]}"
        ),
    }
    std::process::exit(i32::from(exit));
}
