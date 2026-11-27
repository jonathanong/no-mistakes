use clap::{Args, Parser, Subcommand, ValueEnum};
use no_mistakes::invocation::LockState;
use std::process::ExitCode;

#[derive(Parser, Clone)]
pub struct LockArgs {
    #[command(subcommand)]
    command: LockCommand,
}

#[derive(Subcommand, Clone)]
enum LockCommand {
    /// Report whether another invocation holds the lock, and who (never waits).
    Status(StatusArgs),
}

#[derive(Copy, Clone, ValueEnum)]
enum StatusFormat {
    Text,
    Json,
}

#[derive(Args, Clone)]
struct StatusArgs {
    #[arg(long, value_enum, default_value_t = StatusFormat::Text)]
    format: StatusFormat,
}

pub fn run(args: LockArgs) -> anyhow::Result<ExitCode> {
    let LockCommand::Status(status) = args.command;
    let report = no_mistakes::invocation::lock_status()?;
    match status.format {
        StatusFormat::Json => println!("{}", serde_json::to_string(&report)?),
        StatusFormat::Text => {
            let state = match (&report.state, &report.holder) {
                (LockState::Held, Some(holder)) => format!("held by {}", holder.describe()),
                (LockState::Held, None) => {
                    "held by another no-mistakes invocation (holder details unavailable)".into()
                }
                (LockState::Free, _) => "free".into(),
                (LockState::Unavailable, _) => "unavailable (no invocation lock selected)".into(),
            };
            println!("{state}");
            if let Some(path) = report.lock_file {
                println!("lock file: {path}");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
