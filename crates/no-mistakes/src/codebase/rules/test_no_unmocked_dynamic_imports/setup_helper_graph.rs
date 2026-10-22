use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use anyhow::Result;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

/// Alternating lower/upper execution bounds discard cuts from inactive helpers.
/// Ambiguous mock cycles contribute no registrations until execution is proven.
///
/// `on_active` sees only helpers that survive the mock cut. Earlier bounds still
/// include inactive helpers, so shape diagnostics must use this callback.
pub(super) fn collect(
    graph: &DepGraph,
    setup: &Path,
    visible_files: &crate::fx::PathSet,
    initial_mocks: &HashSet<PathBuf>,
    mut registrations: impl FnMut(&Path) -> Result<HashSet<PathBuf>>,
    mut eager_requires: impl FnMut(&Path) -> HashSet<PathBuf>,
    mut on_active: impl FnMut(&Path),
) -> Result<HashSet<PathBuf>> {
    let mut requires = HashMap::new();
    let mut helper_files = |excluded: &HashSet<PathBuf>| {
        let mut seen = HashSet::from([setup.to_path_buf()]);
        let mut queue = VecDeque::from([setup.to_path_buf()]);
        while let Some(file) = queue.pop_front() {
            let eager = requires
                .entry(file.clone())
                .or_insert_with(|| eager_requires(&file));
            for (node, kind) in graph
                .dependencies_of_node(&NodeId::file(file))
                .into_iter()
                .flatten()
            {
                let Some(path) = node.as_file() else {
                    continue;
                };
                if !(matches!(kind, EdgeKind::Import | EdgeKind::WorkspaceImport)
                    || *kind == EdgeKind::Require && eager.contains(path))
                {
                    continue;
                }
                if visible_files.contains(path)
                    && !excluded.contains(path)
                    && crate::codebase::dependencies::extract::is_indexable(path)
                    && seen.insert(path.to_path_buf())
                {
                    queue.push_back(path.to_path_buf());
                }
            }
        }
        seen.remove(setup);
        seen
    };
    let mut upper = helper_files(initial_mocks);
    let mut by_helper = HashMap::new();
    for file in &upper {
        by_helper.insert(file.clone(), registrations(file));
    }
    let cuts = |active: &HashSet<PathBuf>| {
        let mut cuts = initial_mocks.clone();
        for file in active {
            if let Some(Ok(mocks)) = by_helper.get(file) {
                cuts.extend(mocks.iter().cloned());
            }
        }
        cuts
    };
    let mut lower = HashSet::new();
    loop {
        let next_lower = helper_files(&cuts(&upper));
        let next_upper = helper_files(&cuts(&next_lower));
        if next_lower == lower && next_upper == upper {
            break;
        }
        lower = next_lower;
        upper = next_upper;
    }
    let mut active_mocks = initial_mocks.clone();
    let mut active = lower.into_iter().collect::<Vec<_>>();
    active.sort();
    for file in active {
        if let Some(outcome) = by_helper.remove(&file) {
            active_mocks.extend(outcome?);
            on_active(&file);
        }
    }
    Ok(active_mocks)
}
