use anyhow::{Context, Result};
use clap::Args;
use no_mistakes::cli::{resolve_root, Format};
use no_mistakes::rsc_callers_query::{self, RscCallersReport};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub(crate) struct RscCallersArgs {
    /// Component file to find RSC callers of.
    pub(crate) component: PathBuf,
    #[arg(long, default_value = ".", global = true)]
    pub(crate) root: PathBuf,
    #[arg(long, global = true)]
    pub(crate) tsconfig: Option<PathBuf>,
    #[arg(long, global = true)]
    pub(crate) config: Option<PathBuf>,
    /// Maximum traversal depth (default: unlimited).
    #[arg(long, global = true)]
    pub(crate) depth: Option<usize>,
    /// Output format: json, yml, md, paths, human.
    #[arg(
        long,
        value_enum,
        default_value = "human",
        global = true,
        conflicts_with = "json"
    )]
    pub(crate) format: Format,
    /// Shorthand for --format json.
    #[arg(long, global = true, conflicts_with = "format")]
    pub(crate) json: bool,
}

pub(crate) fn run(args: RscCallersArgs) -> Result<ExitCode> {
    let RscCallersArgs {
        component,
        root,
        tsconfig,
        config,
        depth,
        format,
        json,
    } = args;
    let effective_format = if json { Format::Json } else { format };
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = resolve_root(&root, &cwd);
    let report = rsc_callers_query::run(
        &root,
        config.as_deref(),
        tsconfig.as_deref(),
        &component,
        depth,
    )?;
    match effective_format {
        Format::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .expect("serialization of Rust structs never fails")
            );
        }
        Format::Yml => {
            println!(
                "{}",
                serde_yaml::to_string(&report).expect("serialization of Rust structs never fails")
            );
        }
        Format::Md => print_md(&report),
        Format::Paths => {
            for path in report.paths() {
                println!("{path}");
            }
        }
        Format::Human => print_human(&report),
    }
    Ok(ExitCode::SUCCESS)
}

fn print_human(report: &RscCallersReport) {
    println!("RSC callers of {}", report.component);
    for caller in &report.callers {
        println!(
            "  {} [{:?}/{:?}] (depth {})",
            caller.file, caller.kind, caller.environment, caller.depth
        );
    }
}

fn print_md(report: &RscCallersReport) {
    println!("# RSC callers of `{}`", report.component);
    for caller in &report.callers {
        println!(
            "- `{}` ({:?}, {:?})",
            caller.file, caller.kind, caller.environment
        );
    }
}
