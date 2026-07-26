use super::is_named;
use crate::codebase::md_links;
use pulldown_cmark::{Event, Options as MarkdownOptions, Parser, Tag};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Component, Path, PathBuf};

pub(super) fn link_graph(
    root: &Path,
    markdown: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let known = markdown.iter().cloned().collect::<BTreeSet<_>>();
    markdown
        .iter()
        .map(|path| {
            let links = super::super::read_source(sources, path)
                .map(|source| extract_local_links(root, path, &source, &known))
                .unwrap_or_default();
            (path.clone(), links)
        })
        .collect()
}

fn extract_local_links(
    root: &Path,
    source: &Path,
    content: &str,
    known: &BTreeSet<PathBuf>,
) -> Vec<PathBuf> {
    let mut paths = BTreeSet::new();
    for event in Parser::new_ext(content, MarkdownOptions::all()) {
        let Event::Start(Tag::Link { dest_url, .. }) = event else {
            continue;
        };
        let destination = dest_url
            .as_ref()
            .split(['#', '?'])
            .next()
            .unwrap_or_default();
        if destination.is_empty()
            || md_links::is_external(destination)
            || !destination.ends_with(".md")
        {
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
            if known.contains(&path) {
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

pub(super) fn shortest_depth(
    target: &Path,
    roots: &BTreeSet<String>,
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
) -> Option<usize> {
    let mut queue = graph
        .keys()
        .filter(|path| is_named(path, roots))
        .cloned()
        .map(|path| (path, 0usize))
        .collect::<VecDeque<_>>();
    let mut seen = BTreeSet::new();
    while let Some((current, depth)) = queue.pop_front() {
        if !seen.insert(current.clone()) {
            continue;
        }
        if current == target {
            return Some(depth);
        }
        queue.extend(
            graph
                .get(&current)
                .into_iter()
                .flatten()
                .cloned()
                .map(|next| (next, depth + 1)),
        );
    }
    None
}
