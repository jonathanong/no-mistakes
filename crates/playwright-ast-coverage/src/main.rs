use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    match playwright_ast_coverage::run(playwright_ast_coverage::Cli::parse()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::from(2)
        }
    }
}
