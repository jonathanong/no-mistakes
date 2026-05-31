use crate::playwright::analysis::output::{
    build_related_report, print_coverage_text, print_edges_text, print_related_text,
};
use crate::playwright::analysis::pipeline::analyze_with_policy;
use crate::playwright::analysis::tests_report::{build_tests_report, print_tests_text};
use crate::playwright::analysis::types::UniqueSelectorPolicy;
use crate::playwright::cli::{Command, PlaywrightArgs};
use crate::playwright::config;
use crate::playwright::fsutil::absolutize;
use crate::playwright::playwright_tests;
use anyhow::{Context, Result};
use std::process::ExitCode;

pub fn run(cli: PlaywrightArgs) -> Result<ExitCode> {
    let root = absolutize(&cli.root).context("failed to resolve --root")?;
    let settings = config::load_settings(
        &root,
        cli.config.as_deref(),
        &cli.playwright_config,
        cli.project.clone(),
    )?;
    let analysis = analyze_with_policy(
        &root,
        &settings,
        playwright_tests::TestPolicy {
            assert_conditional_tests: cli.assert_conditional_tests,
            allow_skipped_tests: cli.allow_skipped_tests,
        },
        UniqueSelectorPolicy {
            test_ids: cli.assert_unique_test_ids,
            html_ids: cli.assert_unique_html_ids,
            aggregate: false,
            configured_html_id_selector: false,
        },
    )?;
    match cli.command {
        Command::Check => {
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&analysis.coverage)?);
            } else {
                print_coverage_text(&analysis.coverage);
            }
            if analysis.coverage.summary.uncovered_routes > 0
                || analysis.coverage.summary.uncovered_selectors > 0
                || analysis.coverage.summary.duplicate_selectors > 0
            {
                Ok(ExitCode::from(1))
            } else {
                Ok(ExitCode::SUCCESS)
            }
        }
        Command::Edges => {
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&analysis.edges)?);
            } else {
                print_edges_text(&analysis.edges);
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Related { files } => {
            let related = build_related_report(&root, &analysis.edges.edges, &files);
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&related)?);
            } else {
                print_related_text(&related);
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Tests { files } => {
            let report = build_tests_report(&analysis.edges.edges, &files, &root);
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_tests_text(&report);
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}
