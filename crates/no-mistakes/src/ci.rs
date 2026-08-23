//! `no-mistakes ci` — GitHub Actions workflow-graph commands.

use crate::cli::{resolve_format, Format};
use crate::codebase::ci_graph::env_query::{CiEnvReport, EnvLocationKind, EnvScope};
use crate::codebase::ci_graph::impact::CiImpactReport;
use crate::codebase::ci_graph::{
    analyze_env_from_snapshot, analyze_impact, relative_slash, WorkflowSet,
};
use crate::codebase::workflow_topology::load_workflow_topology_from_snapshot;
use crate::codebase::workflow_topology::model::WorkflowTopology;
use crate::config::v2::load_v2_config_from_visible;
use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
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
    /// Parse `.github/workflows` into a typed graph of workflows, jobs, and
    /// `needs`/reusable-call/`workflow_run` edges, with diagnostics for
    /// malformed, dangling, cyclic, or contract-violating definitions.
    Topology(CiTopologyArgs),
}

#[derive(Clone, Copy, ValueEnum)]
enum TopologyFormat {
    Json,
    Mermaid,
}

#[derive(Args)]
struct CiTopologyArgs {
    /// Restrict output to this workflow (basename, e.g. `ci.yml`, or a path
    /// inside `.github/workflows`) plus its transitive local
    /// reusable-workflow callees. Repeatable; defaults to every workflow.
    #[arg(long = "workflow", value_name = "PATH")]
    workflows: Vec<String>,
    /// Project root directory.
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Path to config file.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Output format: json or mermaid.
    #[arg(long, value_enum, conflicts_with = "json")]
    format: Option<TopologyFormat>,
    /// Shorthand for --format json.
    #[arg(long, default_value_t = false, conflicts_with = "format")]
    json: bool,
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
        CiCommand::Topology(sub) => run_topology(sub),
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

fn run_topology(args: CiTopologyArgs) -> Result<ExitCode> {
    let report = topology_report(&args.root, args.config.as_deref(), &args.workflows)?;
    // Matches the original engine's CLI: any error diagnostic means nothing
    // is written to stdout — the graph is printed only once it's clean.
    if !report.diagnostics.is_empty() {
        for diagnostic in &report.diagnostics {
            eprintln!("{}", render::format_topology_diagnostic(diagnostic));
        }
        return Ok(ExitCode::FAILURE);
    }
    let format = if args.json {
        TopologyFormat::Json
    } else {
        args.format.unwrap_or(TopologyFormat::Json)
    };
    print!("{}", render::render_topology(&report, format)?);
    Ok(ExitCode::SUCCESS)
}

/// Compute the impact report for changed files (shared by CLI and N-API).
pub fn impact_report(
    root: &Path,
    config: Option<&Path>,
    files: &[PathBuf],
) -> Result<CiImpactReport> {
    let root = resolve_root(root)?;
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    let config = load_v2_config_from_visible(&root, config, &visible_paths)?;
    let set = WorkflowSet::load_from_snapshot(&root, &config.ci, &snapshot);
    let changed: Vec<String> = files.iter().map(|file| changed_rel(&root, file)).collect();
    Ok(analyze_impact(&set, &changed))
}

/// Compute the env report for a variable (shared by CLI and N-API).
pub fn env_report(root: &Path, config: Option<&Path>, var: &str) -> Result<CiEnvReport> {
    let root = resolve_root(root)?;
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    let config = load_v2_config_from_visible(&root, config, &visible_paths)?;
    Ok(analyze_env_from_snapshot(&root, &config.ci, var, &snapshot))
}

/// Compute the workflow topology graph (shared by CLI and N-API). Reuses
/// the same visibility snapshot for config loading and graph discovery —
/// one file-universe discovery pass per invocation, matching the sibling
/// `impact_report`/`env_report` shared entrypoints above.
pub fn topology_report(
    root: &Path,
    config: Option<&Path>,
    workflows: &[String],
) -> Result<WorkflowTopology> {
    let root = resolve_root(root)?;
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    let config = load_v2_config_from_visible(&root, config, &visible_paths)?;
    Ok(load_workflow_topology_from_snapshot(
        &root, &config.ci, &snapshot, workflows,
    ))
}

fn resolve_root(root: &Path) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let root = crate::cli::resolve_optional_root(Some(root), &cwd);
    // Lexical normalization only — never `canonicalize`, so we don't resolve
    // symlinks (GitHub matches the literal repo path) and root/abs stay
    // prefix-comparable on every platform (no Windows `\\?\` mismatch).
    Ok(crate::codebase::ts_resolver::normalize_path(&root))
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
