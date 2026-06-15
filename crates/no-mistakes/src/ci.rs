//! `no-mistakes ci` — GitHub Actions workflow-graph commands.

use crate::cli::{resolve_format, Format};
use crate::codebase::ci_graph::env_query::{CiEnvReport, EnvLocationKind, EnvScope};
use crate::codebase::ci_graph::impact::CiImpactReport;
use crate::codebase::ci_graph::{analyze_env, analyze_impact, relative_slash, WorkflowSet};
use crate::config::v2::load_v2_config;
use anyhow::Result;
use clap::{Args, Subcommand};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Args)]
pub struct CiArgs {
    #[command(subcommand)]
    command: CiCommand,
}

#[derive(Subcommand)]
enum CiCommand {
    /// Show which workflows the given changed file(s) trigger and the
    /// permissions each job requires.
    Impact(CiImpactArgs),
    /// Find every workflow definition and `${{ env.VAR }}` reference of an
    /// environment variable.
    Env(CiEnvArgs),
}

#[derive(Args)]
struct CiImpactArgs {
    /// Changed file path(s), relative to `--root` (or absolute).
    #[arg(required = true, value_name = "FILE")]
    files: Vec<PathBuf>,
    /// Project root directory.
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Path to config file.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Output format: json, md, yml, paths, human.
    #[arg(long, value_enum, conflicts_with = "json")]
    format: Option<Format>,
    /// Shorthand for --format json.
    #[arg(long, default_value_t = false, conflicts_with = "format")]
    json: bool,
}

#[derive(Args)]
struct CiEnvArgs {
    /// Environment variable name (case-sensitive).
    #[arg(value_name = "VAR")]
    var: String,
    /// Project root directory.
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Path to config file.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Output format: json, md, yml, paths, human.
    #[arg(long, value_enum, conflicts_with = "json")]
    format: Option<Format>,
    /// Shorthand for --format json.
    #[arg(long, default_value_t = false, conflicts_with = "format")]
    json: bool,
}

pub fn run(args: CiArgs) -> Result<ExitCode> {
    match args.command {
        CiCommand::Impact(sub) => run_impact(sub),
        CiCommand::Env(sub) => run_env(sub),
    }
}

const _: fn(CiArgs) -> Result<ExitCode> = run;

fn run_impact(args: CiImpactArgs) -> Result<ExitCode> {
    let report = impact_report(&args.root, args.config.as_deref(), &args.files)?;
    let format = resolve_format(args.json, args.format, std::io::stdout().is_terminal());
    print!("{}", render::render_impact(&report, format)?);
    Ok(ExitCode::SUCCESS)
}

fn run_env(args: CiEnvArgs) -> Result<ExitCode> {
    let report = env_report(&args.root, args.config.as_deref(), &args.var)?;
    let format = resolve_format(args.json, args.format, std::io::stdout().is_terminal());
    print!("{}", render::render_env(&report, format)?);
    Ok(ExitCode::SUCCESS)
}

/// Compute the impact report for changed files (shared by CLI and N-API).
pub fn impact_report(
    root: &Path,
    config: Option<&Path>,
    files: &[PathBuf],
) -> Result<CiImpactReport> {
    let root = resolve_root(root)?;
    let config = load_v2_config(&root, config)?;
    let set = WorkflowSet::load(&root, &config.ci);
    let changed: Vec<String> = files.iter().map(|file| changed_rel(&root, file)).collect();
    Ok(analyze_impact(&set, &changed))
}

/// Compute the env report for a variable (shared by CLI and N-API).
pub fn env_report(root: &Path, config: Option<&Path>, var: &str) -> Result<CiEnvReport> {
    let root = resolve_root(root)?;
    let config = load_v2_config(&root, config)?;
    Ok(analyze_env(&root, &config.ci, var))
}

fn resolve_root(root: &Path) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let root = crate::cli::resolve_optional_root(Some(root), &cwd);
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    Ok(root.canonicalize().unwrap_or(root))
}

fn changed_rel(root: &Path, file: &Path) -> String {
    let abs = if file.is_absolute() {
        crate::codebase::ts_resolver::normalize_path(file)
    } else {
        crate::codebase::ts_resolver::normalize_path(&root.join(file))
    };
    relative_slash(root, &abs)
}

mod render;

#[cfg(test)]
mod tests;
