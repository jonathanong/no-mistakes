use super::super::path_filter::GlobMatcher;
use super::{is_named, RuleState, RuleStates};
use crate::codebase::rules::markdown_reachability::graph::{
    index_reachable, link_graph, shortest_depths,
};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) struct ScopeOptions<'a> {
    pub(super) roots: &'a BTreeSet<String>,
    pub(super) indexes: &'a BTreeSet<String>,
    pub(super) exclude_roots: &'a GlobMatcher,
    pub(super) max_depth: usize,
    pub(super) facts: &'a super::super::markdown_facts::MarkdownFactMap,
}

pub(super) fn scoped_states(
    root: &Path,
    scope_roots: &[PathBuf],
    dispatcher_markdown: &[PathBuf],
    target_paths: &[PathBuf],
    options: ScopeOptions<'_>,
) -> Result<(RuleStates, BTreeSet<String>)> {
    let markdown_by_scope =
        super::super::markdown_scope::partition_markdown_by_scope(scope_roots, dispatcher_markdown);
    let remapper = crate::codebase::ts_source::FrozenPathRemapper::from_paths(
        markdown_by_scope.values().flatten().cloned(),
    );
    let mut targets_by_scope = BTreeMap::<PathBuf, Vec<&PathBuf>>::new();
    for target in target_paths {
        let Some(scope_root) =
            super::super::markdown_scope::scope_root_for_path(scope_roots, target)
        else {
            continue;
        };
        targets_by_scope
            .entry(scope_root.clone())
            .or_default()
            .push(target);
    }
    let repo_root = crate::codebase::ts_resolver::normalize_path(root);
    let mut states = RuleStates::new();
    let mut target_names = BTreeSet::new();
    let mut named_roots = 0usize;
    let mut kept_roots = 0usize;
    for (scope_root, scoped_targets) in targets_by_scope {
        let scoped_markdown = markdown_by_scope
            .get(&scope_root)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let graph = link_graph(&scope_root, scoped_markdown, options.facts, &remapper)?;
        let (seeds, named, kept) =
            traversal_seeds(&repo_root, &graph, options.roots, options.exclude_roots);
        named_roots += named;
        kept_roots += kept;
        let depths = shortest_depths(&seeds, &graph);
        let reachable = index_reachable(&seeds, options.indexes, &graph, options.max_depth);
        for target in scoped_targets.into_iter().filter(|path| {
            !is_named(path, options.roots) || excludes_root(&repo_root, path, options.exclude_roots)
        }) {
            let baseline_key =
                super::super::markdown_scope::baseline_key(root, &scope_root, target);
            if !target_names.insert(baseline_key.clone()) {
                anyhow::bail!(
                    "{} has ambiguous baseline key `{baseline_key}` across configured project roots; configure separate rule applications",
                    super::RULE_ID
                );
            }
            let depth = depths.get(target).copied();
            let allowed = reachable.contains(target);
            states.insert(
                baseline_key,
                RuleState {
                    finding_file: super::super::markdown_scope::finding_key(root, target),
                    depth,
                    allowed,
                    invalid_intermediary: !allowed
                        && depth.is_some_and(|depth| depth <= options.max_depth),
                },
            );
        }
    }
    if named_roots > 0 && kept_roots == 0 {
        anyhow::bail!(
            "{}: excludeRoots excluded all configured roots",
            super::RULE_ID
        );
    }
    Ok((states, target_names))
}

fn excludes_root(root: &Path, path: &Path, exclude_roots: &GlobMatcher) -> bool {
    exclude_roots.is_match(&relative_slash_path(root, path))
}

fn traversal_seeds(
    root: &Path,
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
    names: &BTreeSet<String>,
    exclude_roots: &GlobMatcher,
) -> (Vec<PathBuf>, usize, usize) {
    let mut seeds = Vec::new();
    let mut named = 0usize;
    for path in graph.keys() {
        if !is_named(path, names) {
            continue;
        }
        named += 1;
        if excludes_root(root, path, exclude_roots) {
            continue;
        }
        seeds.push(path.clone());
    }
    let kept = seeds.len();
    (seeds, named, kept)
}
