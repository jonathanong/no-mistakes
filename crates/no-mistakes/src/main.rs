mod check;
mod queues;
mod react;
mod server;

use anyhow::Result;
use clap::{Parser, Subcommand};
use no_mistakes_core::cli::{init_rayon_threads, JobsArg};
use no_mistakes_core::codebase::dependencies::{self, Direction, TraverseArgs};
use no_mistakes_core::codebase::symbols::{self, SymbolsArgs};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, ExitCode, Stdio};

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    after_help = "External subcommands: unknown commands are proxied to matching no-mistakes-* executables on PATH, for example `no-mistakes rust-no-inline-tests` -> `no-mistakes-rust-no-inline-tests`."
)]
struct Cli {
    #[command(flatten)]
    jobs: JobsArg,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Find files that the given files depend on.
    Dependencies(TraverseArgs),
    /// Find files that depend on the given files.
    Dependents(TraverseArgs),
    /// Find files that depend on the given files (alias for `dependents`).
    Related(TraverseArgs),
    /// Dump named exports and imports of TS/JS files.
    Symbols(SymbolsArgs),
    /// Analyze React component traits.
    React(react::ReactArgs),
    /// Analyze queue producer/worker relationships (BullMQ, glide-mq).
    Queues(queues::QueuesArgs),
    /// Analyze server route graphs (Express, Hono, Koa).
    Server(server::ServerArgs),
    /// Run all checks across configured projects (react + queues).
    Check(check::CheckArgs),
    /// Proxy to a matching no-mistakes-* executable on PATH.
    #[command(external_subcommand)]
    External(Vec<OsString>),
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    init_rayon_threads(cli.jobs);
    match cli.command {
        Command::Dependencies(args) => {
            dependencies::run(args, Direction::Deps)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Dependents(args) | Command::Related(args) => {
            dependencies::run(args, Direction::Dependents)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Symbols(args) => {
            symbols::run(args)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::React(args) => react::run(args),
        Command::Queues(args) => queues::run(args),
        Command::Server(args) => server::run(args),
        Command::Check(args) => check::run(args),
        Command::External(args) => proxy_external(args),
    }
}

fn proxy_external(args: Vec<OsString>) -> Result<ExitCode> {
    let Some((subcommand, forwarded)) = args.split_first() else {
        anyhow::bail!("missing external subcommand");
    };
    let subcommand = subcommand
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("external subcommand is not valid UTF-8"))?;
    let executable = format!("no-mistakes-{subcommand}");
    let executable_path = find_in_path(&executable).ok_or_else(|| {
        anyhow::anyhow!("unknown command `{subcommand}` and `{executable}` was not found on PATH")
    })?;

    let status = ProcessCommand::new(executable_path)
        .args(forwarded)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Ok(match status.code() {
        Some(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        None => ExitCode::from(1),
    })
}

fn find_in_path(executable: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(executable);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let candidate = dir.join(format!("{executable}.exe"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
