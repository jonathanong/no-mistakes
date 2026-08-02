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

pub(super) fn direct_or_readme_hop(
    target: &Path,
    roots: &BTreeSet<String>,
    indexes: &BTreeSet<String>,
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
    max_depth: usize,
) -> bool {
    graph
        .keys()
        .filter(|path| is_named(path, roots))
        .any(|root| {
            graph
                .get(root)
                .is_some_and(|links| links.contains(&target.to_path_buf()))
                || (max_depth >= 2
                    && graph
                        .get(root)
                        .into_iter()
                        .flatten()
                        .filter(|path| is_named(path, indexes))
                        .any(|index| {
                            graph
                                .get(index)
                                .is_some_and(|links| links.contains(&target.to_path_buf()))
                        }))
        })
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
