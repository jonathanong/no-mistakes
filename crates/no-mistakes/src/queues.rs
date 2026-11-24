use super::common_graph_args::GraphCommandArgs;
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use no_mistakes::cli::{resolve_root, root_scoped_edge_depth, Format};
use no_mistakes::queue::{analyze_project, analyze_project_indexed, RelatedDirection};
use std::process::ExitCode;

mod render;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

#[derive(Args)]
pub(crate) struct QueuesArgs {
    #[command(flatten)]
    common: GraphCommandArgs,
    #[command(subcommand)]
    pub(crate) command: QueuesCommand,
}

#[derive(Subcommand)]
pub(crate) enum QueuesCommand {
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
    /// Check producers, workers, cycles, waits, and mixed priorities.
    Check {
        /// Include queue analysis diagnostics alongside findings in structured output.
        #[arg(long)]
        include_diagnostics: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
pub(crate) enum QueueDirection {
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
    let root = resolve_root(&args.common.root, &base);
    let format = if args.common.json {
        Format::Json
    } else {
        args.common.format
    };
    let _ = args.common.timings;
    match &args.command {
        QueuesCommand::Edges { files } => {
            let report = trace_queue_analysis(|| {
                analyze_project_indexed(
                    &root,
                    args.common.tsconfig.as_deref(),
                    &args.common.filters,
                )
            })?;
            let depth = root_scoped_edge_depth(files, args.common.depth);
            let edges = report.edge_view(files, depth);
            render::print_edges(&edges, format)?;
            Ok(ExitCode::SUCCESS)
        }
        QueuesCommand::Related { files, direction } => {
            let report = trace_queue_analysis(|| {
                analyze_project_indexed(
                    &root,
                    args.common.tsconfig.as_deref(),
                    &args.common.filters,
                )
            })?;
            let edges = report.related(files, (*direction).into());
            render::print_related(files, &edges, format)?;
            Ok(ExitCode::SUCCESS)
        }
        QueuesCommand::Check {
            include_diagnostics,
        } => {
            let report = trace_queue_analysis(|| {
                analyze_project(&root, args.common.tsconfig.as_deref(), &args.common.filters)
            })?;
            render::print_diagnostics_stderr(&report.diagnostics, *include_diagnostics);
            render::print_check(
                &report.check,
                &report.diagnostics,
                format,
                *include_diagnostics,
            )?;
            Ok(
                if !report
                    .check
                    .iter()
                    .any(no_mistakes::queue::CheckFinding::is_blocking)
                {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                },
            )
        }
    }
}

fn trace_queue_analysis<T>(operation: impl FnOnce() -> Result<T>) -> Result<T> {
    match no_mistakes::diagnostics::current() {
        Some(observer) => observer.trace(
            "analysis.queues",
            no_mistakes::diagnostics::TimingKind::Serial,
            operation,
        ),
        None => operation(),
    }
}
