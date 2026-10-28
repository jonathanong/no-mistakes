use clap::{Args, Parser, Subcommand, ValueEnum};
use std::process::ExitCode;

#[derive(Parser, Clone)]
pub struct LockArgs {
    #[command(subcommand)]
    command: LockCommand,
}

#[derive(Subcommand, Clone)]
enum LockCommand {
    /// Report whether another invocation holds the lock, and who (read-only, never waits).
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
        StatusFormat::Text => match &report.holder {
            Some(holder) => println!("held by {}", holder.describe()),
            None if report.state == no_mistakes::invocation::LockState::Held => {
                println!("held by another no-mistakes invocation (holder details unavailable)")
            }
            None => println!("free"),
        },
    }
    Ok(ExitCode::SUCCESS)
}
