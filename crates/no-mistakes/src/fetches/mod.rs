mod analyze;
mod cli;
mod pipeline;
mod report;

pub use cli::FetchesArgs;
pub use report::types::FinalReport;

use anyhow::Result;
use std::path::Path;
use std::process::ExitCode;

pub fn analyze_with_base_root(base_root: &Path, args: &FetchesArgs) -> Result<FinalReport> {
    pipeline::run::run_with_base_root(base_root, args)
}

const _: for<'a, 'b> fn(&'a Path, &'b FetchesArgs) -> Result<FinalReport> = analyze_with_base_root;

pub fn run(args: FetchesArgs) -> Result<ExitCode> {
    cli::run(args)
}

const _: fn(FetchesArgs) -> Result<ExitCode> = run;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
