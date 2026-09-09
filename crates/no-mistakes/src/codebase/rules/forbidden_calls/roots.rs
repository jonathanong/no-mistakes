use super::config::{Options, Root, VitestSelector};
use super::RULE_ID;
use crate::codebase::dependencies::graph::{CallRoot, DepGraph, NodeId};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

pub(super) fn expand(
    root: &Path,
    options: &Options,
    graph: &DepGraph,
    catalog: Option<&super::super::PreparedVitestProjectCatalog>,
    graph_files: &[PathBuf],
) -> Result<Vec<NodeId>> {
    let mut call_roots = Vec::new();
    for selector in &options.roots {
        match selector {
            Root::File(selector) => call_roots.push(CallRoot::File(require_parsed_file(
                root,
                graph,
                &selector.file,
                "file root",
            )?)),
            Root::Module(selector) => call_roots.push(CallRoot::Module(require_parsed_file(
                root,
                graph,
                &selector.module,
                "module root",
            )?)),
            Root::Function(selector) => call_roots.push(CallRoot::Function {
                file: require_parsed_file(root, graph, &selector.function.file, "function root")?,
                symbol: selector.function.symbol.clone(),
            }),
            Root::Vitest(selector) => {
                let catalog = catalog.context(
                    "forbidden-calls Vitest roots require a prepared Vitest project catalog",
                )?;
                let names = match &selector.vitest {
                    VitestSelector::All(true) => Vec::new(),
                    VitestSelector::All(false) => unreachable!("validated options"),
                    VitestSelector::Projects(names) => names.clone(),
                };
                let files = catalog.matching_files(root, &names, graph_files)?;
                for file in &files {
                    reject_parse_error(graph, root, file)?;
                }
                call_roots.push(CallRoot::Vitest { files });
            }
        }
    }
    let mut nodes = Vec::new();
    for call_root in call_roots {
        let mut expanded = graph.expand_call_roots(std::slice::from_ref(&call_root));
        if expanded.is_empty() {
            bail!("{RULE_ID}: a configured root resolves to no callable source");
        }
        nodes.append(&mut expanded);
    }
    nodes.sort();
    nodes.dedup();
    Ok(nodes)
}

fn require_parsed_file(
    root: &Path,
    graph: &DepGraph,
    configured: &str,
    kind: &str,
) -> Result<PathBuf> {
    let path = require_file(root, configured, kind)?;
    reject_parse_error(graph, root, &path)?;
    Ok(path)
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

fn require_file(root: &Path, configured: &str, kind: &str) -> Result<PathBuf> {
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
