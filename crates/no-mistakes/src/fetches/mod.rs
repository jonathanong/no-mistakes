mod analyze;
mod cli;
mod pipeline;
mod report;

pub(crate) use cli::FetchesArgs;
pub(crate) use report::types::FinalReport;

use anyhow::Result;
use std::path::Path;
use std::process::ExitCode;

pub(crate) fn analyze_with_base_root(base_root: &Path, args: &FetchesArgs) -> Result<FinalReport> {
    pipeline::run::run_with_base_root(base_root, args)
}

const _: for<'a, 'b> fn(&'a Path, &'b FetchesArgs) -> Result<FinalReport> = analyze_with_base_root;

pub(crate) fn run(args: FetchesArgs) -> Result<ExitCode> {
    cli::run(args)
}

const _: fn(FetchesArgs) -> Result<ExitCode> = run;

#[cfg(test)]
mod tests;
