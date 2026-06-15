use anyhow::{Context, Result};
use clap::Args;
use no_mistakes::cli::{resolve_root, Format};
use no_mistakes::effects_query::{self, EffectsReport};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub(crate) struct EffectsArgs {
    /// Effect kind to resolve (a key under `effects:` in config).
    pub(crate) kind: String,
    /// Entry file whose transitive imports are scanned.
    #[arg(long)]
    pub(crate) entry: PathBuf,
    #[arg(long, default_value = ".", global = true)]
    pub(crate) root: PathBuf,
    #[arg(long, global = true)]
    pub(crate) tsconfig: Option<PathBuf>,
    #[arg(long, global = true)]
    pub(crate) config: Option<PathBuf>,
    /// Restrict to one or more configured categories.
    #[arg(long = "category", global = true)]
    pub(crate) categories: Vec<String>,
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

pub(crate) fn run(args: EffectsArgs) -> Result<ExitCode> {
    let EffectsArgs {
        kind,
        entry,
        root,
        tsconfig,
        config,
        categories,
        depth,
        format,
        json,
    } = args;
    let effective_format = if json { Format::Json } else { format };
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = resolve_root(&root, &cwd);
    let report = effects_query::run(
        &root,
        config.as_deref(),
        tsconfig.as_deref(),
        &kind,
        &entry,
        &categories,
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

fn print_human(report: &EffectsReport) {
    println!("{} effects reachable from {}", report.kind, report.entry);
    for site in &report.call_sites {
        let category = site.category.as_deref().unwrap_or("-");
        let caller = site.caller.as_deref().unwrap_or("-");
        println!(
            "  {}:{} {} [{}] caller={} (depth {})",
            site.file, site.line, site.callee, category, caller, site.depth
        );
    }
}

fn print_md(report: &EffectsReport) {
    println!("# effects `{}` from `{}`", report.kind, report.entry);
    for site in &report.call_sites {
        let category = site.category.as_deref().unwrap_or("-");
        let caller = site.caller.as_deref().unwrap_or("-");
        println!(
            "- `{}:{}` `{}` ({}) caller={} depth {}",
            site.file, site.line, site.callee, category, caller, site.depth
        );
    }
}
