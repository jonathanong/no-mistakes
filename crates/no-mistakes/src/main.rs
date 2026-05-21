mod check;
mod check_discovery;
mod check_parallel;
mod check_runner;
mod check_tasks;
#[cfg(all(test, unix))]
mod executable_file_tests;
mod queues;
mod react;
mod server;
mod tests;

use anyhow::Result;
use clap::{Parser, Subcommand};
use no_mistakes_core::cli::{init_rayon_threads, JobsArg};
use no_mistakes_core::codebase::dependencies::{self, Direction, TraverseArgs};
use no_mistakes_core::codebase::symbols::{self, SymbolsArgs};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
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
    /// Run configured project checks.
    Check(check::CheckArgs),
    /// Plan, explain, and visualize test impacts based on changed files.
    Tests(tests::TestsArgs),
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
        Command::Tests(args) => tests::run(args),
        Command::External(args) => proxy_external(args),
    }
}

fn proxy_external(args: Vec<OsString>) -> Result<ExitCode> {
    let (subcommand, forwarded) = args
        .split_first()
        .expect("clap external subcommands include a command");
    let subcommand = subcommand.to_string_lossy();

    if subcommand.is_empty()
        || !subcommand
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        anyhow::bail!(
            "invalid command name `{subcommand}`; only ASCII letters, digits, hyphens, and underscores are allowed"
        );
    }

    let executable = format!("no-mistakes-{subcommand}");
    let executable_path = find_in_path(&executable).ok_or_else(|| {
        anyhow::anyhow!("unknown command `{subcommand}`; `{executable}` was not found on PATH")
    })?;

    let status = ProcessCommand::new(executable_path)
        .args(forwarded)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    #[cfg(windows)]
    std::process::exit(status.code().unwrap_or(1));

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        let code = status
            .code()
            .or_else(|| status.signal().map(|signal| 128 + signal))
            .and_then(|code| u8::try_from(code).ok())
            .unwrap_or(1);

        Ok(ExitCode::from(code))
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(ExitCode::from(
            status
                .code()
                .and_then(|code| u8::try_from(code).ok())
                .unwrap_or(1),
        ))
    }
}

fn find_in_path(executable: &str) -> Option<PathBuf> {
    let executable_path = Path::new(executable);
    if executable_path.is_absolute() || executable_path.components().count() > 1 {
        return is_executable_file(executable_path).then(|| executable_path.to_path_buf());
    }

    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        if !dir.is_absolute() {
            continue;
        }
        let candidate = dir.join(executable);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let pathext =
                std::env::var_os("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
            for extension in std::env::split_paths(&pathext) {
                let Some(extension) = extension.to_str() else {
                    continue;
                };
                let candidate = dir.join(format!("{executable}{extension}"));
                if is_executable_file(&candidate) {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
        && rustix::fs::access(path, rustix::fs::Access::EXEC_OK).is_ok()
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}
