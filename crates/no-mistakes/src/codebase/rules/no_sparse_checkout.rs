use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "no-sparse-checkout";
const DEFAULT_INCLUDE: &[&str] = &[".github/workflows/**", ".github/actions/**"];
const FORBIDDEN_KEYS: &[&str] = &["sparse-checkout", "sparse-checkout-cone-mode"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
}
struct CompiledOptions {
    include: super::path_filter::GlobMatcher,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
}
pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(files);
    check_with_files_and_sources(root, config, files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_with_files_sources_and_deferred_suppression(root, config, all_files, sources, false)
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let results: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let opts = compile_options(rule.try_rule_options()?)?;
            let roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &roots))
                .filter(|path| {
                    matches!(
                        path.extension().and_then(|value| value.to_str()),
                        Some("yml" | "yaml")
                    )
                })
                .filter(|path| opts.include.is_match(&relative_slash_path(root, path)))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            Ok(files
                .par_iter()
                .flat_map(|path| scan::check_file(root, path, sources, defer_suppression))
                .collect())
        })
        .collect();
    let mut findings: Vec<RuleFinding> = results?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}
fn compile_options(options: Options) -> Result<CompiledOptions> {
    let include = if options.include.is_empty() {
        DEFAULT_INCLUDE
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    } else {
        options.include
    };
    Ok(CompiledOptions {
        include: super::path_filter::GlobMatcher::new(
            &include,
            &format!("{RULE_ID} options.include"),
        )?,
    })
}
#[cfg(test)]
mod tests;
