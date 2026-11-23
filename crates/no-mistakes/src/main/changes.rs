use anyhow::Result;
use clap::{Args, Subcommand};
use no_mistakes::changes::{ChangesDiffArgs, ChangesReport};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args)]
pub(super) struct ChangesArgs {
    #[command(subcommand)]
    command: ChangesCommand,
}

#[derive(Subcommand)]
enum ChangesCommand {
    /// Compare categorized changes in two local directory trees, without Git or network access.
    Diff(DiffArgs),
}

#[derive(Args)]
struct DiffArgs {
    #[arg(long)]
    base_root: PathBuf,
    #[arg(long)]
    head_root: PathBuf,
    #[arg(long)]
    config: Option<PathBuf>,
    /// Output format (JSON).
    #[arg(long, default_value = "json", value_parser = ["json"])]
    format: String,
}

pub(super) fn run(args: ChangesArgs) -> Result<ExitCode> {
    let ChangesCommand::Diff(args) = args.command;
    let report: ChangesReport = no_mistakes::changes::diff(&ChangesDiffArgs {
        base_root: args.base_root,
        head_root: args.head_root,
        config: args.config,
    })?;
    no_mistakes::cli::print_json(&report);
    Ok(if report.complete {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
