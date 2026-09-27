use super::is_named;
use crate::codebase::md_links;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Component, Path, PathBuf};

pub(super) fn link_graph(
    root: &Path,
    markdown: &[PathBuf],
    facts: &super::super::markdown_facts::MarkdownFactMap,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Result<BTreeMap<PathBuf, Vec<PathBuf>>> {
    let known = markdown.iter().cloned().collect::<BTreeSet<_>>();
    markdown
        .iter()
        .map(|path| -> Result<_> {
            let facts = facts.get_for_rule(path, super::RULE_ID)?;
            let links = extract_local_links(root, path, &facts.link_destinations, &known, remapper);
            Ok((path.clone(), links))
        })
        .collect()
}

fn extract_local_links(
    root: &Path,
    source: &Path,
    destinations: &[String],
    known: &BTreeSet<PathBuf>,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Vec<PathBuf> {
    let mut paths = BTreeSet::new();
    for destination in destinations {
        let destination = destination
            .as_str()
            .split(['#', '?'])
            .next()
            .unwrap_or_default();
        if destination.is_empty() || md_links::is_external(destination) {
            continue;
        }
        let Some(destination) = md_links::decode_local_path(destination) else {
            continue;
        };
        let base = if destination.starts_with('/') {
            root.to_path_buf()
        } else {
            source.parent().unwrap_or(root).to_path_buf()
        };
        if let Some(path) = normalize_inside(root, &base.join(destination.trim_start_matches('/')))
        {
            if let Some(path) = remapper.remap(&path).filter(|path| known.contains(path)) {
                paths.insert(path);
            }
        }
    }
    paths.into_iter().collect()
}

pub(super) fn normalize_inside(root: &Path, path: &Path) -> Option<PathBuf> {
    let mut relative = PathBuf::new();
    for component in path.strip_prefix(root).ok()?.components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            Component::ParentDir if !relative.pop() => return None,
            Component::ParentDir => {}
            _ => return None,
        }
    }
    Some(root.join(relative))
}

pub(super) fn index_reachable(
    roots: &BTreeSet<String>,
    indexes: &BTreeSet<String>,
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
    max_depth: usize,
) -> BTreeSet<PathBuf> {
    let mut queue = graph
        .keys()
        .filter(|path| is_named(path, roots))
        .cloned()
        .map(|path| (path, 0usize))
        .collect::<VecDeque<_>>();
    let mut expanded = BTreeSet::new();
    let mut reachable = BTreeSet::new();
    while let Some((current, depth)) = queue.pop_front() {
        if !expanded.insert(current.clone()) || depth >= max_depth {
            continue;
        }
        for next in graph.get(&current).into_iter().flatten() {
            reachable.insert(next.clone());
            // Any Markdown file can be an endpoint; only configured indexes
            // may supply another discovery hop after an instruction root.
            if depth + 1 < max_depth && is_named(next, indexes) {
                queue.push_back((next.clone(), depth + 1));
            }
        }
    }
    reachable
}

pub(super) fn shortest_depths(
    roots: &BTreeSet<String>,
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
) -> BTreeMap<PathBuf, usize> {
    let mut queue = graph
        .keys()
        .filter(|path| is_named(path, roots))
        .cloned()
        .map(|path| (path, 0usize))
        .collect::<VecDeque<_>>();
    let mut depths = BTreeMap::new();
    while let Some((current, depth)) = queue.pop_front() {
        if depths.contains_key(&current) {
            continue;
        }
        depths.insert(current.clone(), depth);
        queue.extend(
            graph
                .get(&current)
                .into_iter()
                .flatten()
                .cloned()
                .map(|next| (next, depth + 1)),
        );
    }
    depths
}
