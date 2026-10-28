use super::super::path_filter::GlobMatcher;
use super::config::{catalog_names, Options, PlaywrightRoot, Root};
use super::root_files::{excludes_root, reject_parse_error, require_file};
use crate::codebase::dependencies::graph::{CallRoot, DepGraph, NodeId};
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

pub(crate) struct ExpandRequest<'a> {
    pub rule: &'a str,
    pub root: &'a Path,
    pub options: &'a Options,
    pub graph: &'a DepGraph,
    pub vitest: Option<&'a super::super::PreparedVitestProjectCatalog>,
    pub playwright: Option<&'a super::super::PreparedPlaywrightProjectCatalog>,
    pub graph_files: &'a [PathBuf],
    pub target_roots: &'a [PathBuf],
}

pub(crate) fn expand(input: ExpandRequest<'_>) -> Result<Vec<NodeId>> {
    expand_with_files(input).map(|(nodes, _)| nodes)
}

/// Expanded call roots plus the source files the selectors named, including
/// files that have no call site and so contribute no call node.
pub(crate) fn expand_with_files(input: ExpandRequest<'_>) -> Result<(Vec<NodeId>, Vec<PathBuf>)> {
    let exclude_roots = input
        .options
        .exclude_roots
        .as_ref()
        .map(|patterns| patterns.values())
        .unwrap_or_default();
    let exclude_roots = GlobMatcher::new(&exclude_roots, &format!("{} excludeRoots", input.rule))?;
    let mut call_roots = Vec::new();
    let mut skipped_unknown = false;
    for selector in &input.options.roots {
        if let Some(call_root) =
            expand_selector(&input, selector, &exclude_roots, &mut skipped_unknown)?
        {
            call_roots.push(call_root);
        }
    }
    if call_roots.is_empty() && skipped_unknown {
        return Ok((Vec::new(), Vec::new()));
    }
    if call_roots.is_empty() {
        bail!("{}: excludeRoots excluded all configured roots", input.rule);
    }
    let mut nodes = Vec::new();
    let mut files = Vec::new();
    for call_root in call_roots {
        match &call_root {
            CallRoot::File(file) | CallRoot::Module(file) => files.push(file.clone()),
            CallRoot::Function { file, .. } => files.push(file.clone()),
            CallRoot::Files { files: selected } => files.extend(selected.iter().cloned()),
        }
        let mut expanded = input
            .graph
            .expand_call_roots(std::slice::from_ref(&call_root));
        if expanded.is_empty() {
            bail!(
                "{}: a configured root resolves to no callable source",
                input.rule
            );
        }
        nodes.append(&mut expanded);
    }
    nodes.sort();
    nodes.dedup();
    files.sort();
    files.dedup();
    Ok((nodes, files))
}

fn expand_selector(
    input: &ExpandRequest<'_>,
    selector: &Root,
    exclude_roots: &GlobMatcher,
    skipped_unknown: &mut bool,
) -> Result<Option<CallRoot>> {
    match selector {
        Root::File(selector) => {
            let file = require_file(input.rule, input.root, &selector.file, "file root")?;
            if excludes_root(input.root, &file, exclude_roots) {
                return Ok(None);
            }
            reject_parse_error(input.rule, input.graph, input.root, &file)?;
            Ok(Some(CallRoot::File(file)))
        }
        Root::Module(selector) => {
            let file = require_file(input.rule, input.root, &selector.module, "module root")?;
            if excludes_root(input.root, &file, exclude_roots) {
                return Ok(None);
            }
            reject_parse_error(input.rule, input.graph, input.root, &file)?;
            Ok(Some(CallRoot::Module(file)))
        }
        Root::Function(selector) => {
            let file = require_file(
                input.rule,
                input.root,
                &selector.function.file,
                "function root",
            )?;
            if excludes_root(input.root, &file, exclude_roots) {
                return Ok(None);
            }
            reject_parse_error(input.rule, input.graph, input.root, &file)?;
            Ok(Some(CallRoot::Function {
                file,
                symbol: selector.function.symbol.clone(),
            }))
        }
        Root::Vitest(selector) => {
            let catalog = input.vitest.context(format!(
                "{} Vitest roots require a prepared Vitest project catalog",
                input.rule
            ))?;
            let files = catalog.matching_files(
                input.root,
                &catalog_names(&selector.vitest),
                input.graph_files,
            )?;
            collection_root(input, files, "Vitest root matched no files", exclude_roots)
        }
        Root::Playwright(PlaywrightRoot { playwright }) => {
            let catalog = input.playwright.context(format!(
                "{} Playwright roots require a prepared Playwright project catalog",
                input.rule
            ))?;
            let (files, unknown) = catalog.matching_files(
                input.root,
                &catalog_names(playwright),
                input.graph_files,
            )?;
            if files.is_empty() && unknown {
                *skipped_unknown = true;
                return Ok(None);
            }
            collection_root(
                input,
                files,
                "Playwright root matched no files",
                exclude_roots,
            )
        }
        Root::Glob(selector) => {
            let patterns = selector.glob.values();
            let mut files = super::super::matching_files(
                input.root,
                &patterns,
                input.graph_files,
                input.target_roots,
            )?;
            files.sort();
            files.dedup();
            collection_root(input, files, "glob root matched no files", exclude_roots)
        }
    }
}

fn collection_root(
    input: &ExpandRequest<'_>,
    mut files: Vec<PathBuf>,
    empty_message: &str,
    exclude_roots: &GlobMatcher,
) -> Result<Option<CallRoot>> {
    if files.is_empty() {
        bail!("{}: {empty_message}", input.rule);
    }
    files.retain(|file| !excludes_root(input.root, file, exclude_roots));
    if files.is_empty() {
        return Ok(None);
    }
    for file in &files {
        reject_parse_error(input.rule, input.graph, input.root, file)?;
    }
    Ok(Some(CallRoot::Files { files }))
}
