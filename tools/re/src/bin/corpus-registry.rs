fn main() {
    let result =
        std::panic::catch_unwind(|| re_corpus::registry::run_cli(std::env::args_os().skip(1)));
    let exit = match result {
        Ok(exit) => exit,
        Err(_) => {
            println!(
                "{}",
                "{\"command\":\"unknown\",\"ok\":false,\"exit\":2,\"status\":\"ERROR\",\"findings\":[{\"code\":\"EXECUTION_ERROR\"}]}"
            );
            2
        }
    };
    std::process::exit(i32::from(exit));
}
