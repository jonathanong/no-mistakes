use super::analysis_policy::AnalysisDiagnostics;
use super::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "github-actions-no-insta-snapshot-updates";
const NAME: &str = "INSTA_UPDATE";

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Options {}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources, false)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, all_files, sources, defer_suppression)
        .map(|output| output.findings)
}

fn resolve_unanalyzable(
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
) -> Result<Unanalyzable> {
    let explicit = rule.options.get("unanalyzable").is_some() || config.unanalyzable.is_some();
    if explicit {
        Ok(config.effective_rule_analysis(rule)?.unanalyzable)
    } else {
        Ok(Unanalyzable::Advise)
    }
}

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let _opts: Options = rule.try_rule_options()?;
        let unanalyzable = resolve_unanalyzable(config, rule)?;
        let roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &roots))
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        let chunks: Vec<AnalysisDiagnostics> = files
            .par_iter()
            .filter_map(|path| {
                scan::check_workflow(root, path, sources, defer_suppression, unanalyzable)
            })
            .collect();
        for chunk in chunks {
            output.append(chunk);
        }
    }
    super::sort_findings(&mut output.findings);
    super::sort_findings(&mut output.advisories);
    Ok(output)
}
