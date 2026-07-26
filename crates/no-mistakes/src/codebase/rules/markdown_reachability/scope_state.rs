use super::{is_named, RuleState, RuleStates};
use crate::codebase::rules::markdown_reachability::graph::{
    direct_or_readme_hop, link_graph, shortest_depths,
};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) struct ScopeOptions<'a> {
    pub(super) roots: &'a BTreeSet<String>,
    pub(super) indexes: &'a BTreeSet<String>,
    pub(super) max_depth: usize,
    pub(super) sources: &'a crate::codebase::ts_source::SourceStore,
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
    let mut states = RuleStates::new();
    let mut target_names = BTreeSet::new();
    for (scope_root, scoped_targets) in targets_by_scope {
        let scoped_markdown = markdown_by_scope
            .get(&scope_root)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let graph = link_graph(&scope_root, scoped_markdown, options.sources, &remapper);
        let depths = shortest_depths(options.roots, &graph);
        for target in scoped_targets
            .into_iter()
            .filter(|path| !is_named(path, options.roots))
        {
            let baseline_key =
                super::super::markdown_scope::baseline_key(root, &scope_root, target);
            if !target_names.insert(baseline_key.clone()) {
                anyhow::bail!(
                    "{} has ambiguous baseline key `{baseline_key}` across configured project roots; configure separate rule applications",
                    super::RULE_ID
                );
            }
            let depth = depths.get(target).copied();
            let allowed = direct_or_readme_hop(
                target,
                options.roots,
                options.indexes,
                &graph,
                options.max_depth,
            );
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
    Ok((states, target_names))
}
