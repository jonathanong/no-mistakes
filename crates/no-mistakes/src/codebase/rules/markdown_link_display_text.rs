use super::RuleFinding;
use crate::codebase::ts_source::{byte_offset_to_line, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod link_target;
pub(crate) mod parser;
use parser::InlineLink;

pub const RULE_ID: &str = "markdown-link-display-text";

const DEFAULT_EXTENSIONS: &[&str] = &[".md"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) extensions: Vec<String>,
}

pub(crate) fn fact_candidate_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Vec<PathBuf> {
    let mut extensions = config
        .rule_applications(RULE_ID)
        .into_iter()
        .flat_map(|rule| {
            let opts: Options = rule.rule_options();
            if opts.extensions.is_empty() {
                DEFAULT_EXTENSIONS
                    .iter()
                    .map(|extension| (*extension).to_string())
                    .collect()
            } else {
                opts.extensions
            }
        })
        .collect::<Vec<_>>();
    extensions.sort();
    extensions.dedup();
    files
        .iter()
        .filter(|path| {
            let rel = relative_slash_path(root, path);
            extensions.iter().any(|extension| rel.ends_with(extension))
        })
        .cloned()
        .collect()
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let mut plan = super::markdown_facts::MarkdownFactPlan::default();
    plan.request_display_links(fact_candidate_files(root, config, all_files));
    let facts = super::markdown_facts::MarkdownFactMap::prepare(&plan, sources);
    check_with_files_sources_and_facts(root, config, all_files, &facts)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    facts: &super::markdown_facts::MarkdownFactMap,
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
            scan_with_facts(root, &opts, &files, facts)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan_with_facts(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    facts: &super::markdown_facts::MarkdownFactMap,
) -> Result<Vec<RuleFinding>> {
    let extensions = effective_extensions(opts);
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file_with_facts(root, path, &extensions, facts))
        .collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn effective_extensions(opts: &Options) -> Vec<&str> {
    if opts.extensions.is_empty() {
        DEFAULT_EXTENSIONS.to_vec()
    } else {
        opts.extensions.iter().map(String::as_str).collect()
    }
}

fn check_file_with_facts(
    root: &Path,
    path: &Path,
    extensions: &[&str],
    facts: &super::markdown_facts::MarkdownFactMap,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if !extensions.iter().any(|ext| rel.ends_with(ext)) {
        return Vec::new();
    }
    let Some(markdown) = facts.get(path) else {
        return Vec::new();
    };
    markdown
        .display_links
        .iter()
        .cloned()
        .filter_map(|link| link_target::finding_for_link(&rel, &markdown.source, link, extensions))
        .collect()
}

#[cfg(test)]
mod tests;
