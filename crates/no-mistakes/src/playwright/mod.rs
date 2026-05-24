mod analysis;
mod ast;
mod cli;
mod config;
mod fsutil;
mod matcher;
pub mod playwright_config;
pub(crate) mod playwright_tests;
pub mod playwright_urls;
mod routes;
mod rule_findings;
pub mod rules;
pub mod selectors;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
mod url;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub use analysis::cli_run::run;
pub use cli::PlaywrightArgs;

#[derive(Clone, Copy)]
pub(crate) enum PlaywrightReportKind {
    Check,
    Edges,
    Related,
    Tests,
}

pub(crate) struct PlaywrightReportOptions {
    pub(crate) root: PathBuf,
    pub(crate) config: Option<PathBuf>,
    pub(crate) playwright_config: Vec<PathBuf>,
    pub(crate) project: Option<String>,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) assert_conditional_tests: bool,
    pub(crate) allow_skipped_tests: bool,
    pub(crate) assert_unique_test_ids: bool,
    pub(crate) assert_unique_html_ids: bool,
    pub(crate) assert_unique_selectors: bool,
}

pub(crate) fn report_json(
    kind: PlaywrightReportKind,
    options: PlaywrightReportOptions,
) -> Result<String> {
    let root = fsutil::absolutize(&options.root).context("failed to resolve root")?;
    let config_path = options.config.as_deref();
    let configs = &options.playwright_config;
    let project = options.project.clone();
    let settings = report_settings(&root, config_path, configs, project)?;
    let analysis = analysis::pipeline::analyze_with_policy(
        &root,
        &settings,
        playwright_tests::TestPolicy {
            assert_conditional_tests: options.assert_conditional_tests,
            allow_skipped_tests: options.allow_skipped_tests,
        },
        analysis::types::UniqueSelectorPolicy {
            test_ids: options.assert_unique_test_ids || options.assert_unique_selectors,
            html_ids: options.assert_unique_html_ids
                || (options.assert_unique_selectors && settings.html_ids),
            aggregate: options.assert_unique_selectors,
            configured_html_id_selector: false,
        },
    )?;

    match kind {
        PlaywrightReportKind::Check => to_pretty_json(&analysis.coverage),
        PlaywrightReportKind::Edges => to_pretty_json(&analysis.edges),
        PlaywrightReportKind::Related => {
            require_files(&options.files)?;
            let related = analysis::output::build_related_report(
                &root,
                &analysis.edges.edges,
                &options.files,
            );
            to_pretty_json(&related)
        }
        PlaywrightReportKind::Tests => {
            let report = analysis::tests_report::build_tests_report(
                &analysis.edges.edges,
                &options.files,
                &root,
            );
            to_pretty_json(&report)
        }
    }
}

fn report_settings(
    root: &Path,
    config_path: Option<&std::path::Path>,
    playwright_configs: &[PathBuf],
    project: Option<String>,
) -> Result<config::Settings> {
    config::load_settings(root, config_path, playwright_configs, project)
}

fn require_files(files: &[PathBuf]) -> Result<()> {
    if files.is_empty() {
        anyhow::bail!("files must contain at least one file");
    }
    Ok(())
}

fn to_pretty_json<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_string_pretty(value).map_err(Into::into)
}
