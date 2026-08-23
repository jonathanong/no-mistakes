use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod links;
mod scan;

pub const RULE_ID: &str = "markdown-child-links";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) groups: Vec<Group>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Group {
    pub(crate) parents: Vec<String>,
    pub(crate) children: Vec<String>,
    pub(crate) require_whole_file: bool,
    pub(crate) count_canonical_html_list_items: bool,
}

pub(crate) struct CompiledGroup {
    parents: GlobMatcher,
    children: GlobMatcher,
    require_whole_file: bool,
    count_canonical_html_list_items: bool,
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    facts: &super::markdown_facts::MarkdownFactMap,
) -> Result<Vec<RuleFinding>> {
    let markdown = super::markdown_scope::markdown_files(all_files);
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.rule_options();
        let groups = compile_groups(&options)?;
        let files = super::path_filter::filter_markdown_rule_files(root, config, rule, &markdown)?;
        findings.extend(scan::scan(root, &files, facts, &groups)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_groups(options: &Options) -> Result<Vec<CompiledGroup>> {
    options
        .groups
        .iter()
        .map(|group| {
            Ok(CompiledGroup {
                parents: GlobMatcher::new(&group.parents, &format!("{RULE_ID} parents"))?,
                children: GlobMatcher::new(&group.children, &format!("{RULE_ID} children"))?,
                require_whole_file: group.require_whole_file,
                count_canonical_html_list_items: group.count_canonical_html_list_items,
            })
        })
        .collect()
}

fn matches_group(matcher: &GlobMatcher, rel: &str) -> bool {
    !matcher.is_empty() && matcher.is_match(rel)
}

fn relative(root: &Path, path: &Path) -> String {
    relative_slash_path(root, path)
}

#[cfg(test)]
mod tests;
