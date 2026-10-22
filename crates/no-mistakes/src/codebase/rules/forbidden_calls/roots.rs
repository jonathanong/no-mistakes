use super::super::path_filter::GlobMatcher;
use super::config::{catalog_names, Options, PlaywrightRoot, Root};
use super::RULE_ID;
use crate::codebase::dependencies::graph::{CallRoot, DepGraph, NodeId};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

pub(super) struct ExpandRequest<'a> {
    pub root: &'a Path,
    pub options: &'a Options,
    pub graph: &'a DepGraph,
    pub vitest: Option<&'a super::super::PreparedVitestProjectCatalog>,
    pub playwright: Option<&'a super::super::PreparedPlaywrightProjectCatalog>,
    pub graph_files: &'a [PathBuf],
    pub target_roots: &'a [PathBuf],
}

pub(super) fn expand(input: ExpandRequest<'_>) -> Result<Vec<NodeId>> {
    let exclude_roots = input
        .options
        .exclude_roots
        .as_ref()
        .map(|patterns| patterns.values())
        .unwrap_or_default();
    let exclude_roots = GlobMatcher::new(&exclude_roots, "forbidden-calls excludeRoots")?;
    let mut call_roots = Vec::new();
    for selector in &input.options.roots {
        if let Some(call_root) = expand_selector(&input, selector, &exclude_roots)? {
            call_roots.push(call_root);
        }
    }
    if call_roots.is_empty() {
        bail!("{RULE_ID}: excludeRoots excluded all configured roots");
    }
    let mut nodes = Vec::new();
    for call_root in call_roots {
        let mut expanded = input
            .graph
            .expand_call_roots(std::slice::from_ref(&call_root));
        if expanded.is_empty() {
            bail!("{RULE_ID}: a configured root resolves to no callable source");
        }
        nodes.append(&mut expanded);
    }
    nodes.sort();
    nodes.dedup();
    Ok(nodes)
}

fn expand_selector(
    input: &ExpandRequest<'_>,
    selector: &Root,
    exclude_roots: &GlobMatcher,
) -> Result<Option<CallRoot>> {
    match selector {
        Root::File(selector) => {
            let file = require_file(input.root, &selector.file, "file root")?;
            if excludes_root(input.root, &file, exclude_roots) {
                return Ok(None);
            }
            reject_parse_error(input.graph, input.root, &file)?;
            Ok(Some(CallRoot::File(file)))
        }
        Root::Module(selector) => {
            let file = require_file(input.root, &selector.module, "module root")?;
            if excludes_root(input.root, &file, exclude_roots) {
                return Ok(None);
            }
            reject_parse_error(input.graph, input.root, &file)?;
            Ok(Some(CallRoot::Module(file)))
        }
        Root::Function(selector) => {
            let file = require_file(input.root, &selector.function.file, "function root")?;
            if excludes_root(input.root, &file, exclude_roots) {
                return Ok(None);
            }
            reject_parse_error(input.graph, input.root, &file)?;
            Ok(Some(CallRoot::Function {
                file,
                symbol: selector.function.symbol.clone(),
            }))
        }
        Root::Vitest(selector) => {
            let catalog = input.vitest.context(
                "forbidden-calls Vitest roots require a prepared Vitest project catalog",
            )?;
            let files = catalog.matching_files(
                input.root,
                &catalog_names(&selector.vitest),
                input.graph_files,
            )?;
            collection_root(input, files, "Vitest root matched no files", exclude_roots)
        }
        Root::Playwright(PlaywrightRoot { playwright }) => {
            let catalog = input.playwright.context(
                "forbidden-calls Playwright roots require a prepared Playwright project catalog",
            )?;
            let files = catalog.matching_files(
                input.root,
                &catalog_names(playwright),
                input.graph_files,
            )?;
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
        bail!("{RULE_ID}: {empty_message}");
    }
    files.retain(|file| !excludes_root(input.root, file, exclude_roots));
    if files.is_empty() {
        return Ok(None);
    }
    for file in &files {
        reject_parse_error(input.graph, input.root, file)?;
    }
    Ok(Some(CallRoot::Files { files }))
}

pub(super) fn excludes_root(root: &Path, file: &Path, exclude_roots: &GlobMatcher) -> bool {
    exclude_roots.is_match(&relative_slash_path(root, file))
}

fn reject_parse_error(graph: &DepGraph, root: &Path, path: &Path) -> Result<()> {
    let Some(error) = graph.parse_error(path) else {
        return Ok(());
    };
    bail!(
        "{RULE_ID}: configured root `{}` failed to parse: {error}",
        relative_slash_path(root, path)
    );
}

pub(super) fn require_file(root: &Path, configured: &str, kind: &str) -> Result<PathBuf> {
    let path = Path::new(configured);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    if !path.is_file() {
        bail!("{RULE_ID}: {kind} `{configured}` is not a file");
    }
    Ok(crate::codebase::ts_resolver::normalize_path(&path))
}
