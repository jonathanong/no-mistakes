use std::process::ExitCode;

fn main() -> ExitCode {
    match next_to_fetch::run_cli() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{e:#}");
            ExitCode::from(2)
        }
    }
}
