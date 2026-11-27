use anyhow::Result;
use clap::Args;
use std::{path::PathBuf, process::ExitCode};

#[derive(Args)]
pub(super) struct LockInventoryArgs {
    #[arg(long, default_value = ".")]
    root: PathBuf,
    #[arg(long)]
    config: Option<PathBuf>,
    /// Initialize a missing baseline or tighten an existing one; never add sites to it.
    #[arg(long)]
    write_baseline: bool,
}

pub(super) fn run(args: LockInventoryArgs) -> Result<ExitCode> {
    let report = no_mistakes::postgres_lock_inventory::run(
        &no_mistakes::postgres_lock_inventory::PostgresLockInventoryOptions {
            root: args.root,
            config: args.config,
            write_baseline: args.write_baseline,
        },
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(if report.is_clean() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}
