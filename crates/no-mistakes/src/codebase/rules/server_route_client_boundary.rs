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
mod execution;
mod paths;
use execution::{check_files, BorrowedFactItem};

pub const RULE_ID: &str = "server-route-client-boundary";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FileFacts {
    has_server_route_shape: bool,
    client_call_lines: Vec<usize>,
}

pub(crate) fn extract_program(
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
) -> FileFacts {
    FileFacts {
        has_server_route_shape: ast::has_server_like_route_call_from_program(path, source, program),
        client_call_lines: if has_disable_file_comment(source, RULE_ID) {
            Vec::new()
        } else {
            ast::client_call_lines_from_program(source, program)
        },
    }
}

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
    check_with_optional_inferred(root, config, shared, None)
}

pub(crate) fn check_with_facts_and_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    inferred_roots: &crate::codebase::config::InferredRoots,
) -> Result<Vec<RuleFinding>> {
    check_with_optional_inferred(root, config, shared, Some(inferred_roots))
}

fn check_with_optional_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut facts = Vec::new();
    for path in shared.files() {
        let Some(file) = shared.ts.get(path) else {
            continue;
        };
        let Some(boundary) = file.server_route_client_boundary.as_ref() else {
            bail!("{} requires boundary facts for {}", RULE_ID, path.display());
        };
        facts.push(BorrowedFactItem::new(path.as_path(), boundary));
    }
    check_items(
        &root,
        config,
        &facts,
        |item| item.path,
        |item| item.facts,
        inferred_roots,
    )
}

pub(super) fn check_items<T>(
    root: &Path,
    config: &NoMistakesConfig,
    items: &[T],
    path_for: impl Fn(&T) -> &Path + Sync,
    facts_for: impl Fn(&T) -> &FileFacts + Sync,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>>
where
    T: Sync,
{
    let mut findings = Vec::new();
    let mut inferred_roots = inferred_roots.cloned().unwrap_or_default();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let exclude_matcher = ExcludeMatcher::new(&opts.excludes);
        let filter = super::path_filter::RulePathFilter::new_with_inferred(
            root,
            config,
            rule,
            &mut inferred_roots,
        )?;
        let Some(route_globset) = route_globset_for_rule(config, rule) else {
            continue;
        };
        let route_dirs: HashSet<PathBuf> = items
            .par_iter()
            .filter_map(|item| {
                let path = path_for(item);
                (route_globset.is_match(relative_path(root, path))
                    && !exclude_matcher.is_match(root, path)
                    && filter.is_match(path))
                .then(|| {
                    facts_for(item)
                        .has_server_route_shape
                        .then(|| path.parent().map(Path::to_path_buf))
                        .flatten()
                })
                .flatten()
            })
            .collect();
        if route_dirs.is_empty() {
            continue;
        }
        findings.extend(
            items
                .par_iter()
                .filter(|item| {
                    let path = path_for(item);
                    !exclude_matcher.is_match(root, path)
                        && filter.is_match(path)
                        && path.ancestors().any(|dir| route_dirs.contains(dir))
                })
                .flat_map(|item| {
                    let path = path_for(item);
                    client_findings_for_file(root, path, facts_for(item))
                })
                .collect::<Vec<_>>(),
        );
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn client_findings_for_file(root: &Path, path: &Path, facts: &FileFacts) -> Vec<RuleFinding> {
    let file = relative_slash_path(root, path);
    facts
        .client_call_lines
        .iter()
        .copied()
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
