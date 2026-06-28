use anyhow::Result;
use clap::{Args, ValueEnum};
use no_mistakes::cli::Format;
use no_mistakes::codebase::dependencies::RelationshipArg;
use no_mistakes::flow_query::{self, FlowDirection, FlowOptions};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args)]
pub(crate) struct FlowArgs {
    /// File or file#symbol to summarize.
    target: String,
    /// Project root directory.
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Path to tsconfig.json for alias resolution.
    #[arg(long)]
    tsconfig: Option<PathBuf>,
    /// Path to no-mistakes config.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Traversal direction.
    #[arg(long, value_enum, default_value = "both")]
    direction: FlowDirectionArg,
    /// Maximum traversal depth.
    #[arg(long, default_value_t = 2)]
    depth: usize,
    /// Edge relationship filters.
    #[arg(long = "relationship", value_enum)]
    relationships: Vec<RelationshipArg>,
    /// Output format: json, yml, md, paths, human.
    #[arg(long, value_enum, default_value = "human", conflicts_with = "json")]
    format: Format,
    /// Shorthand for --format json.
    #[arg(long, conflicts_with = "format")]
    json: bool,
}

#[derive(ValueEnum, Clone, Copy)]
enum FlowDirectionArg {
    Deps,
    Dependents,
    Both,
}

pub(crate) fn run(args: FlowArgs) -> Result<ExitCode> {
    let format = if args.json { Format::Json } else { args.format };
    let options = FlowOptions {
        target: args.target,
        root: args.root,
        tsconfig: args.tsconfig,
        config: args.config,
        direction: match args.direction {
            FlowDirectionArg::Deps => FlowDirection::Deps,
            FlowDirectionArg::Dependents => FlowDirection::Dependents,
            FlowDirectionArg::Both => FlowDirection::Both,
        },
        depth: args.depth,
        relationships: args.relationships,
    };
    let report = flow_query::run(&options)?;
    print!("{}", render(&report, format)?);
    Ok(ExitCode::SUCCESS)
}

fn render(report: &flow_query::FlowReport, format: Format) -> Result<String> {
    Ok(match format {
        Format::Json => format!("{}\n", serde_json::to_string_pretty(report)?),
        Format::Yml => serde_yaml::to_string(report)?,
        Format::Paths => report
            .nodes
            .iter()
            .filter_map(|node| node.file.as_ref())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|path| format!("{path}\n"))
            .collect(),
        Format::Md => {
            let mut out = format!("# Flow `{}`\n\n", report.target);
            for edge in &report.edges {
                out.push_str(&format!(
                    "- `{}` -> `{}` ({})\n",
                    edge.from, edge.to, edge.kind
                ));
            }
            out
        }
        Format::Human => {
            let mut out = format!("{}\n", report.target);
            for edge in &report.edges {
                out.push_str(&format!("  {} -> {} ({})\n", edge.from, edge.to, edge.kind));
            }
            out
        }
    })
}
