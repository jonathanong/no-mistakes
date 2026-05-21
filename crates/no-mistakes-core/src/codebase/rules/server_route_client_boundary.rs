use super::RuleFinding;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path};
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::{bail, Result};
use paths::{relative_path, route_globset_for_rule, ExcludeMatcher};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod ast;
mod paths;

pub const RULE_ID: &str = "server-route-client-boundary";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    excludes: Vec<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let files =
        crate::codebase::ts_source::discover_files(&root, &config.filesystem.skip_directories);
    let files: Vec<_> = files
        .into_iter()
        .filter(|path| is_indexable(path))
        .collect();
    check_files(&root, config, &files)
}

pub(crate) fn check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut sources = Vec::new();
    for path in shared.files() {
        let Some(facts) = shared.ts.get(path) else {
            continue;
        };
        let Some(source) = facts.source.as_ref() else {
            bail!("{} requires source facts for {}", RULE_ID, path.display());
        };
        sources.push(SourceItem::new(path.as_path(), source.as_str()));
    }
    check_sources(&root, config, &sources)
}

struct SourceItem<'a> {
    path: &'a Path,
    source: &'a str,
}

impl<'a> SourceItem<'a> {
    fn new(path: &'a Path, source: &'a str) -> Self {
        Self { path, source }
    }
}

fn check_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let exclude_matcher = ExcludeMatcher::new(&opts.excludes);
        let Some(route_globset) = route_globset_for_rule(config, rule) else {
            continue;
        };
        let route_dirs: HashSet<PathBuf> = files
            .par_iter()
            .filter(|path| route_globset.is_match(relative_path(root, path)))
            .filter(|path| !exclude_matcher.is_match(root, path))
            .filter_map(|path| {
                std::fs::read_to_string(path).ok().and_then(|source| {
                    ast::has_server_like_route_call(path, &source)
                        .then(|| path.parent().map(Path::to_path_buf))
                        .flatten()
                })
            })
            .collect();
        if route_dirs.is_empty() {
            continue;
        }
        findings.extend(
            files
                .par_iter()
                .filter(|path| !exclude_matcher.is_match(root, path))
                .filter(|path| path.ancestors().any(|dir| route_dirs.contains(dir)))
                .flat_map(|path| {
                    std::fs::read_to_string(path)
                        .ok()
                        .map(|source| client_findings_for_file(root, path, &source))
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>(),
        );
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn check_sources(
    root: &Path,
    config: &NoMistakesConfig,
    sources: &[SourceItem<'_>],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let exclude_matcher = ExcludeMatcher::new(&opts.excludes);
        let Some(route_globset) = route_globset_for_rule(config, rule) else {
            continue;
        };
        let route_dirs: HashSet<PathBuf> = sources
            .par_iter()
            .filter(|item| {
                let path = item.path;
                route_globset.is_match(relative_path(root, path))
                    && !exclude_matcher.is_match(root, path)
                    && ast::has_server_like_route_call(path, item.source)
            })
            .filter_map(|item| item.path.parent().map(Path::to_path_buf))
            .collect();
        if route_dirs.is_empty() {
            continue;
        }
        findings.extend(
            sources
                .par_iter()
                .filter(|item| !exclude_matcher.is_match(root, item.path))
                .filter(|item| item.path.ancestors().any(|dir| route_dirs.contains(dir)))
                .flat_map(|item| client_findings_for_file(root, item.path, item.source))
                .collect::<Vec<_>>(),
        );
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn client_findings_for_file(root: &Path, path: &Path, source: &str) -> Vec<RuleFinding> {
    if has_disable_file_comment(source, RULE_ID) {
        return Vec::new();
    }
    let file = relative_slash_path(root, path);
    ast::client_call_lines(path, source)
        .into_iter()
        .map(|line| RuleFinding {
            rule: RULE_ID.to_string(),
            file: file.clone(),
            line,
            message: "client HTTP call is in a server route folder; move request clients out of route definition folders or narrow server route globs so AST route extraction stays unambiguous".to_string(),
            import: None,
            target: None,
        })
        .collect()
}

#[cfg(test)]
mod tests;
