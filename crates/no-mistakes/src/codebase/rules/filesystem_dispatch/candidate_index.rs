use super::{preserved, FILESYSTEM_RULE_IDS};
use crate::codebase::rules::{
    rule_enabled, BANNED_PATHS, FORBIDDEN_WORKSPACE_CLOSURE, MARKDOWN_MERMAID_VALIDATION,
    MARKDOWN_REACHABILITY, MARKDOWN_STRUCTURE_BUDGET, PRODUCTION_DEPENDENCY_DECLARATIONS,
    RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_ALLOWS, RUST_NO_INLINE_TESTS,
};
use crate::config::v2::NoMistakesConfig;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::candidate_helpers::{
    is_rust_path, markdown_inventory_path_allowed, normalized_paths, rule_can_consume_rust_source,
};

/// Immutable, request-scoped candidates for every enabled filesystem rule.
///
/// The discovered universe is normalized once, then classified against the
/// prepared per-rule preserved roots in one pass. Rule implementations retain
/// application-specific filtering because some need out-of-scope context files.
pub(super) struct RuleCandidateIndex {
    by_rule: BTreeMap<&'static str, Arc<Vec<PathBuf>>>,
    rust: Arc<Vec<PathBuf>>,
    exclusive_rust: Arc<Vec<PathBuf>>,
}

impl RuleCandidateIndex {
    pub(super) fn prepare_with_inventory(
        root: &Path,
        config: &NoMistakesConfig,
        files: &[PathBuf],
        tracked_files: &[PathBuf],
        metadata_files: &[PathBuf],
        inventory_paths: Option<Arc<Vec<PathBuf>>>,
    ) -> Self {
        let root = crate::codebase::ts_resolver::normalize_path(root);
        let mut plans =
            BTreeMap::<(Vec<PathBuf>, bool, bool, bool, bool), Vec<&'static str>>::new();
        for rule_id in FILESYSTEM_RULE_IDS
            .iter()
            .copied()
            .filter(|rule_id| rule_enabled(config, rule_id))
        {
            plans
                .entry((
                    preserved::filesystem_rule_preserved_roots(&root, config, rule_id)
                        .into_iter()
                        .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
                        .collect(),
                    rule_id == FORBIDDEN_WORKSPACE_CLOSURE
                        || rule_id == PRODUCTION_DEPENDENCY_DECLARATIONS,
                    rule_id == BANNED_PATHS,
                    (rule_id == BANNED_PATHS
                        && config
                            .rule_applications(rule_id)
                            .iter()
                            .any(|rule| rule.applies_to_repository()))
                        || matches!(
                            rule_id,
                            MARKDOWN_MERMAID_VALIDATION
                                | MARKDOWN_REACHABILITY
                                | MARKDOWN_STRUCTURE_BUDGET
                        ),
                    matches!(
                        rule_id,
                        MARKDOWN_MERMAID_VALIDATION
                            | MARKDOWN_REACHABILITY
                            | MARKDOWN_STRUCTURE_BUDGET
                    ),
                ))
                .or_default()
                .push(rule_id);
        }
        let files = normalized_paths(files);
        let tracked_files = normalized_paths(tracked_files);
        let metadata_files = normalized_paths(metadata_files);
        let skip = super::super::skip_dir_set(config);
        let mut by_rule = BTreeMap::new();

        // Rules with identical effective scopes share one candidate vector.
        for (
            (
                preserved_roots,
                includes_metadata,
                tracked_only,
                includes_repository_inventory,
                inventory_only,
            ),
            rule_ids,
        ) in plans
        {
            let universe = if includes_metadata {
                metadata_files.as_ref()
            } else if tracked_only {
                tracked_files.as_ref()
            } else {
                files.as_ref()
            };
            let allowed = |path: &PathBuf| {
                super::super::file_allowed_by_roots_and_skip(&root, &skip, path, &preserved_roots)
            };
            let shared = if inventory_only {
                Arc::new(
                    inventory_paths
                        .as_ref()
                        .map(|paths| paths.as_slice())
                        .unwrap_or_default()
                        .iter()
                        // Markdown's tracked repository inventory must honor
                        // both its configured project roots and every source
                        // skip directory. Unlike source discovery, tracked
                        // docs are never retained merely because a project
                        // root happens to sit below a skipped directory.
                        .filter(|path| {
                            markdown_inventory_path_allowed(&root, path, &preserved_roots, &skip)
                        })
                        .cloned()
                        .collect(),
                )
            } else if includes_repository_inventory {
                let mut candidates = inventory_paths
                    .as_ref()
                    .map(|paths| paths.as_slice())
                    .unwrap_or_default()
                    .iter()
                    .filter(|path| path.starts_with(&root))
                    .cloned()
                    .collect::<Vec<_>>();
                candidates.extend(universe.iter().filter(|path| allowed(path)).cloned());
                candidates.sort();
                candidates.dedup();
                Arc::new(candidates)
            } else {
                inventory_paths
                    .as_ref()
                    .filter(|inventory| {
                        !includes_metadata
                            && inventory.as_slice() == universe
                            && universe.iter().all(&allowed)
                    })
                    .map(Arc::clone)
                    .unwrap_or_else(|| {
                        Arc::new(
                            universe
                                .iter()
                                .filter(|path| allowed(path))
                                .cloned()
                                .collect(),
                        )
                    })
            };
            for rule_id in rule_ids {
                by_rule.insert(rule_id, Arc::clone(&shared));
            }
        }
        let mut rust = [
            RUST_MAX_LINES_PER_FILE,
            RUST_NO_INLINE_TESTS,
            RUST_NO_INLINE_ALLOWS,
        ]
        .into_iter()
        .filter_map(|rule_id| by_rule.get(rule_id))
        .flat_map(|paths| paths.iter().filter(|path| is_rust_path(path)).cloned())
        .collect::<Vec<_>>();
        rust.sort();
        rust.dedup();
        let rust_rule_ids = [
            RUST_MAX_LINES_PER_FILE,
            RUST_NO_INLINE_TESTS,
            RUST_NO_INLINE_ALLOWS,
        ];
        let non_rust = by_rule
            .iter()
            .filter(|(rule_id, _)| {
                !rust_rule_ids.contains(rule_id) && rule_can_consume_rust_source(rule_id)
            })
            .flat_map(|(_, paths)| paths.iter().filter(|path| is_rust_path(path)).cloned())
            .collect::<HashSet<_>>();
        let exclusive_rust = rust
            .iter()
            .filter(|path| !non_rust.contains(*path))
            .cloned()
            .collect();

        Self {
            by_rule,
            rust: Arc::new(rust),
            exclusive_rust: Arc::new(exclusive_rust),
        }
    }

    pub(super) fn candidates(&self, rule_id: &str) -> &[PathBuf] {
        self.by_rule
            .get(rule_id)
            .map(|paths| paths.as_slice())
            .unwrap_or_default()
    }

    pub(super) fn rust_candidates(&self) -> &[PathBuf] {
        &self.rust
    }

    pub(super) fn exclusive_rust_candidates(&self) -> &[PathBuf] {
        &self.exclusive_rust
    }

    pub(super) fn all_candidates(&self) -> impl Iterator<Item = &PathBuf> {
        self.by_rule.values().flat_map(|paths| paths.iter())
    }
}

#[cfg(test)]
mod tests;
