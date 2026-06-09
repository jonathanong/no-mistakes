use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use no_mistakes::cli::{resolve_root, Format};
use no_mistakes::react_traits;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub(crate) struct ReactArgs {
    #[arg(long, default_value = ".", global = true)]
    pub(crate) root: PathBuf,
    #[arg(long, global = true)]
    pub(crate) config: Option<PathBuf>,
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
    #[command(subcommand)]
    pub(crate) command: ReactCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ReactCommand {
    /// Analyze component traits and print results.
    Analyze {
        #[arg(help = "Glob patterns for component files")]
        targets: Vec<String>,
    },
    /// Check for violations (e.g. assert-no-fetch).
    Check {
        #[arg(help = "Glob patterns for component files")]
        targets: Vec<String>,
        #[arg(long)]
        assert_no_fetch: bool,
    },
    /// Find JSX callsites, props, stories/tests, and prop types for a component.
    Usages {
        #[arg(help = "Target component: path/to/component.tsx or path/to/component.tsx#Symbol")]
        target: String,
        /// Glob patterns limiting which files are scanned for callsites.
        #[arg(long)]
        scan: Vec<String>,
        /// Sections to include: comma-separated subset of stories,tests,props (default: all).
        #[arg(long)]
        include: Option<String>,
    },
}

pub(crate) fn run(args: ReactArgs) -> Result<ExitCode> {
    let ReactArgs {
        root,
        config,
        format,
        json,
        command,
    } = args;
    let effective_format = if json { Format::Json } else { format };
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = resolve_root(&root, &cwd);
    match &command {
        ReactCommand::Analyze { targets } => {
            let results = react_traits::run_analyze(&root, config.as_deref(), targets, None)?;
            match effective_format {
                Format::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&results)
                            .expect("serialization of Rust structs never fails")
                    );
                }
                Format::Yml => {
                    println!(
                        "{}",
                        serde_yaml::to_string(&results)
                            .expect("serialization of Rust structs never fails")
                    );
                }
                Format::Md => react_traits::print_results_md(&results),
                Format::Paths => {
                    for r in &results {
                        println!("{}", r.file);
                    }
                }
                Format::Human => {
                    react_traits::print_results(&results, 0);
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        ReactCommand::Check {
            targets,
            assert_no_fetch,
        } => {
            let violations =
                react_traits::run_check(&root, config.as_deref(), targets, *assert_no_fetch)?;
            if violations.is_empty() {
                return Ok(ExitCode::SUCCESS);
            }
            match effective_format {
                Format::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&violations)
                            .expect("serialization of Rust structs never fails")
                    );
                }
                Format::Yml => {
                    println!(
                        "{}",
                        serde_yaml::to_string(&violations)
                            .expect("serialization of Rust structs never fails")
                    );
                }
                Format::Md => react_traits::print_violations_md(&violations),
                Format::Paths => {
                    for v in &violations {
                        println!("{}", v.file);
                    }
                }
                Format::Human => {
                    react_traits::print_violations(&violations);
                }
            }
            Ok(ExitCode::from(1))
        }
        ReactCommand::Usages {
            target,
            scan,
            include,
        } => {
            let include = react_traits::UsagesInclude::parse(include.as_deref())?;
            let report =
                react_traits::run_usages(&root, config.as_deref(), target, scan, &include)?;
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
                        serde_yaml::to_string(&report)
                            .expect("serialization of Rust structs never fails")
                    );
                }
                Format::Md => react_traits::print_usages_md(&report),
                Format::Paths => {
                    for path in usages_paths(&report) {
                        println!("{path}");
                    }
                }
                Format::Human => react_traits::print_usages(&report),
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// Deduplicated, sorted union of callsite and importer file paths for
/// `--format paths` command substitution.
fn usages_paths(report: &react_traits::UsagesReport) -> Vec<String> {
    let mut paths: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for callsite in &report.callsites {
        paths.insert(callsite.file.clone());
    }
    for files in [&report.stories, &report.tests].into_iter().flatten() {
        paths.extend(files.iter().cloned());
    }
    paths.into_iter().collect()
}
