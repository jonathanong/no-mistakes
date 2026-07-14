use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use no_mistakes::cli::{resolve_root, root_scoped_edge_depth, Format};
use no_mistakes::queue::{
    analyze_project, analyze_project_indexed, CheckFinding, Edge, RelatedDirection,
};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args)]
pub(crate) struct QueuesArgs {
    /// Project root directory.
    #[arg(long, default_value = ".", global = true)]
    root: PathBuf,
    /// Path to tsconfig.json for path alias resolution.
    #[arg(long, global = true)]
    tsconfig: Option<PathBuf>,
    /// Filter to files matching this glob. Can be repeated.
    #[arg(long = "filter", global = true)]
    filters: Vec<String>,
    /// Maximum edge traversal depth for the edges command when roots are provided.
    /// Defaults to 1 when roots are provided, and unlimited otherwise.
    #[arg(long, alias = "max-depth", global = true)]
    depth: Option<usize>,
    /// Output format: json, yml, md, paths, human.
    #[arg(
        long,
        value_enum,
        default_value = "human",
        global = true,
        conflicts_with = "json"
    )]
    format: Format,
    /// Alias for --format json; cannot be combined with --format.
    #[arg(long, global = true, conflicts_with = "format")]
    json: bool,
    /// Emit phase timings to stderr.
    #[arg(long, global = true)]
    timings: bool,
    #[command(subcommand)]
    command: QueuesCommand,
}

#[derive(Subcommand)]
enum QueuesCommand {
    /// Print queue dependency edges.
    Edges {
        /// Only show edges whose source exactly matches these files/nodes.
        files: Vec<String>,
    },
    /// Print files/nodes related to the given files/nodes.
    Related {
        #[arg(required = true)]
        files: Vec<String>,
        #[arg(long, value_enum, default_value = "both")]
        direction: QueueDirection,
    },
    /// Check for unmatched producers and workers.
    Check,
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum QueueDirection {
    Deps,
    Dependents,
    Both,
}

impl From<QueueDirection> for RelatedDirection {
    fn from(d: QueueDirection) -> Self {
        match d {
            QueueDirection::Deps => RelatedDirection::Deps,
            QueueDirection::Dependents => RelatedDirection::Dependents,
            QueueDirection::Both => RelatedDirection::Both,
        }
    }
}

pub(crate) fn run(args: QueuesArgs) -> Result<ExitCode> {
    let base = std::env::current_dir().context("cwd must be accessible")?;
    let root = resolve_root(&args.root, &base);
    let started = std::time::Instant::now();
    let format = if args.json { Format::Json } else { args.format };
    match &args.command {
        QueuesCommand::Edges { files } => {
            let report = analyze_project_indexed(&root, args.tsconfig.as_deref(), &args.filters)?;
            print_analysis_timing(args.timings, started);
            let depth = root_scoped_edge_depth(files, args.depth);
            let edges = report.edge_view(files, depth);
            print_edges(&edges, format)?;
            Ok(ExitCode::SUCCESS)
        }
        QueuesCommand::Related { files, direction } => {
            let report = analyze_project_indexed(&root, args.tsconfig.as_deref(), &args.filters)?;
            print_analysis_timing(args.timings, started);
            let edges = report.related(files, (*direction).into());
            print_related(files, &edges, format)?;
            Ok(ExitCode::SUCCESS)
        }
        QueuesCommand::Check => {
            let report = analyze_project(&root, args.tsconfig.as_deref(), &args.filters)?;
            print_analysis_timing(args.timings, started);
            print_check(&report.check, format)?;
            Ok(if report.check.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
    }
}

fn print_analysis_timing(enabled: bool, started: std::time::Instant) {
    if enabled {
        eprintln!(
            "analysis: {:.3}ms",
            started.elapsed().as_secs_f64() * 1000.0
        );
    }
}

fn print_edges(edges: &[Edge], format: Format) -> Result<()> {
    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(edges)?),
        Format::Yml => println!("{}", serde_yaml::to_string(edges)?),
        Format::Md => {
            println!("# Queue edges");
            for edge in edges {
                println!("- `{}` -> `{}` ({})", edge.from, edge.to, edge.kind);
            }
        }
        Format::Paths => print_edge_paths(&edges),
        Format::Human => {
            for edge in edges {
                println!("{} -> {}", edge.from, edge.to);
            }
        }
    }
    Ok(())
}

fn print_related(roots: &[String], edges: &[Edge], format: Format) -> Result<()> {
    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(edges)?),
        Format::Yml => println!("{}", serde_yaml::to_string(edges)?),
        Format::Md => {
            println!("# Related queue files");
            for edge in edges {
                println!("- `{}` -> `{}`", edge.from, edge.to);
            }
        }
        Format::Paths => print_edge_paths(edges),
        Format::Human => {
            println!("{}", roots.join(", "));
            for edge in edges {
                println!("  {} -> {}", edge.from, edge.to);
            }
        }
    }
    Ok(())
}

fn print_edge_paths(edges: &[Edge]) {
    let paths: BTreeSet<&str> = edges
        .iter()
        .flat_map(|e| [e.from.as_str(), e.to.as_str()])
        .collect();
    for p in paths {
        println!("{p}");
    }
}

fn print_check(findings: &[CheckFinding], format: Format) -> Result<()> {
    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(findings)?),
        Format::Yml => println!("{}", serde_yaml::to_string(findings)?),
        Format::Md => {
            println!("# Queue check");
            for f in findings {
                println!("- `{}`:{} {}", f.file, f.line, f.message);
            }
        }
        Format::Paths => {
            for f in findings {
                println!("{}:{}", f.file, f.line);
            }
        }
        Format::Human => {
            for f in findings {
                println!(
                    "{}[{}] {}:{} {}",
                    f.kind,
                    f.job.as_deref().unwrap_or("*"),
                    f.file,
                    f.line,
                    f.message
                );
            }
        }
    }
    Ok(())
}
