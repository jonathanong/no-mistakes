use super::RuleFinding;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path};
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::Path;

mod ast;
mod execution;
mod selection;
use selection::{check_items, CheckContext};
mod paths;
use execution::{check_files, BorrowedFactItem};

pub const RULE_ID: &str = "server-route-client-boundary";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FileFacts {
    has_server_route_shape: bool,
    client_call_lines: Vec<usize>,
    disabled: bool,
}

pub(crate) fn extract_program(
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
) -> FileFacts {
    FileFacts {
        has_server_route_shape: ast::has_server_like_route_call_from_program(path, source, program),
        client_call_lines: ast::client_call_lines_from_program(source, program),
        disabled: has_disable_file_comment(source, RULE_ID),
    }
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    excludes: Vec<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::check_indexed_standalone(root, config, check_files)
}

pub(crate) fn diagnostics_with_facts_for_aggregate(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    sources: &crate::codebase::ts_source::SourceStore,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    defer_suppression: bool,
) -> Result<super::analysis_policy::AnalysisDiagnostics> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut facts = Vec::new();
    for path in shared.files() {
        if !crate::codebase::dependencies::extract::is_indexable(path) {
            continue;
        }
        let Some(file) = shared.ts.get(path) else {
            continue;
        };
        let Some(boundary) = file.server_route_client_boundary.as_ref() else {
            bail!("{} requires boundary facts for {}", RULE_ID, path.display());
        };
        facts.push(BorrowedFactItem::new(
            path.as_path(),
            boundary,
            file.ts
                .operational_error
                .as_deref()
                .or(file.parse_error.as_deref()),
        ));
    }
    check_items(
        CheckContext {
            root: &root,
            config,
            sources,
            inferred_roots,
            defer_suppression,
        },
        &facts,
        |item| item.path,
        |item| (item.facts, item.error),
    )
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
#[cfg(test)]
mod tests_support;
