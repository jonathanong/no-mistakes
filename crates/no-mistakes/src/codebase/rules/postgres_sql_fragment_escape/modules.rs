use super::fragments::Catalog;
use super::source_text::file_text;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId, TsFactLookup};
use std::path::{Path, PathBuf};

pub(super) fn module_file(
    graph: &DepGraph,
    facts: &CheckFactMap,
    catalog: &Catalog<'_>,
    importer: &Path,
    specifier: &str,
) -> Option<PathBuf> {
    if let Some(path) = relative_module(facts, importer, specifier) {
        return Some(path);
    }
    let mut hits = neighbor_hits(graph, importer, specifier);
    if hits.len() != 1 && specifier.starts_with('.') {
        hits.extend(relative_hits(facts, importer, specifier));
    }
    if hits.is_empty() {
        hits.extend(ambient_hits(facts, catalog, specifier));
    }
    hits.sort();
    hits.dedup();
    (hits.len() == 1).then(|| hits.pop()).flatten()
}

fn neighbor_hits(graph: &DepGraph, importer: &Path, specifier: &str) -> Vec<PathBuf> {
    let stem = file_stem(specifier);
    if stem.is_empty() {
        return Vec::new();
    }
    let Some(neighbors) = graph.dependencies_of_node(&NodeId::file(importer)) else {
        return Vec::new();
    };
    neighbors
        .iter()
        .filter_map(|(target, kind)| {
            matches!(
                kind,
                EdgeKind::Import
                    | EdgeKind::Require
                    | EdgeKind::DynamicImport
                    | EdgeKind::ConditionalDynamicImport
            )
            .then_some(target.as_file())
            .flatten()
        })
        .filter(|path| path_matches(path, stem))
        .map(Path::to_path_buf)
        .collect()
}

fn relative_module(facts: &CheckFactMap, importer: &Path, specifier: &str) -> Option<PathBuf> {
    if !specifier.starts_with('.') {
        return None;
    }
    let mut hits = relative_hits(facts, importer, specifier);
    (hits.len() == 1).then(|| hits.pop()).flatten()
}

fn relative_hits(facts: &CheckFactMap, importer: &Path, specifier: &str) -> Vec<PathBuf> {
    let Some(dir) = importer.parent() else {
        return Vec::new();
    };
    let logical = crate::codebase::ts_resolver::normalize_path(&dir.join(specifier));
    let Some(stem) = logical.file_stem().and_then(|stem| stem.to_str()) else {
        return Vec::new();
    };
    let Some(parent) = logical.parent() else {
        return Vec::new();
    };
    facts
        .graph_file_universe()
        .iter()
        .filter(|file| path_in_parent(file, parent, stem))
        .cloned()
        .collect()
}

fn ambient_hits(facts: &CheckFactMap, catalog: &Catalog<'_>, specifier: &str) -> Vec<PathBuf> {
    let double = format!("declare module \"{specifier}\"");
    let single = format!("declare module '{specifier}'");
    facts
        .graph_file_universe()
        .iter()
        .filter(|file| {
            facts
                .get_ts_facts(file)
                .and_then(|ts| file_text(ts, catalog.sources, file))
                .is_some_and(|source| source.contains(&double) || source.contains(&single))
        })
        .cloned()
        .collect()
}

fn path_in_parent(file: &Path, parent: &Path, stem: &str) -> bool {
    let file_stem = file.file_stem().and_then(|stem| stem.to_str());
    (file.parent() == Some(parent) && file_stem == Some(stem))
        || (file_stem == Some("index")
            && file.parent().is_some_and(|dir| {
                dir.file_name().and_then(|name| name.to_str()) == Some(stem)
                    && dir.parent() == Some(parent)
            }))
}

fn path_matches(path: &Path, stem: &str) -> bool {
    let file_stem = path.file_stem().and_then(|stem| stem.to_str());
    file_stem == Some(stem)
        || (file_stem == Some("index")
            && path
                .parent()
                .and_then(|dir| dir.file_name())
                .and_then(|name| name.to_str())
                == Some(stem))
}

fn file_stem(specifier: &str) -> &str {
    let segment = specifier.rsplit(['/', '\\']).next().unwrap_or(specifier);
    for ext in [".mts", ".cts", ".tsx", ".jsx", ".mjs", ".cjs", ".ts", ".js"] {
        if let Some(stem) = segment.strip_suffix(ext) {
            return stem;
        }
    }
    segment
}
