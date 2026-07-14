use super::RuleFinding;
use crate::codebase::ts_source::{byte_offset_to_line, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod link_target;
mod parser;
use parser::{markdown_links_outside_code, InlineLink};

pub const RULE_ID: &str = "markdown-link-display-text";

const DEFAULT_EXTENSIONS: &[&str] = &[".md"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) extensions: Vec<String>,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan_with_sources(root, &opts, &files, sources)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan_with_sources(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let extensions = effective_extensions(opts);
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file_with_sources(root, path, &extensions, sources))
        .collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn effective_extensions(opts: &Options) -> Vec<&str> {
    if opts.extensions.is_empty() {
        DEFAULT_EXTENSIONS.to_vec()
    } else {
        opts.extensions.iter().map(String::as_str).collect()
    }
}

fn check_file_with_sources(
    root: &Path,
    path: &Path,
    extensions: &[&str],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if !extensions.iter().any(|ext| rel.ends_with(ext)) {
        return Vec::new();
    }
    let Some(source) = super::read_source(sources, path) else {
        return Vec::new();
    };
    markdown_links_outside_code(&source)
        .into_iter()
        .filter_map(|link| link_target::finding_for_link(&rel, &source, link, extensions))
        .collect()
}

#[cfg(test)]
mod tests;
