use super::{
    candidate_index::RuleCandidateIndex, preserved, rule_enabled, MARKDOWN_REACHABILITY,
    MARKDOWN_STRUCTURE_BUDGET,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Collect tracked files once for the request root and the explicitly configured
/// Markdown project roots. The dispatcher needs the complete inventory for
/// tracked-baseline validation, not merely the source-discovery work list.
pub(super) fn tracked_inventory_with_markdown_project_roots(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
) -> Arc<Vec<PathBuf>> {
    let mut inventory = snapshot.tracked_paths_for(root).as_ref().clone();
    for rule_id in [MARKDOWN_REACHABILITY, MARKDOWN_STRUCTURE_BUDGET] {
        if !rule_enabled(config, rule_id) {
            continue;
        }
        for project_root in preserved::filesystem_rule_preserved_roots(root, config, rule_id) {
            inventory.extend(snapshot.tracked_paths_for(&project_root).iter().cloned());
        }
    }
    inventory.sort();
    inventory.dedup();
    Arc::new(inventory)
}

pub(super) fn register_trusted_external_candidates(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    candidates: &RuleCandidateIndex,
    sources: &crate::codebase::ts_source::SourceStore,
) {
    let trusted_roots = preserved::filesystem_rule_target_roots(
        root,
        config,
        &[MARKDOWN_REACHABILITY, MARKDOWN_STRUCTURE_BUDGET],
    )
    .into_iter()
    .filter(|path| !path.starts_with(root))
    .collect::<Vec<_>>();
    let external = candidates
        .all_candidates()
        .filter(|path| !path.starts_with(root))
        .cloned()
        .collect::<Vec<_>>();
    sources.register_trusted_regular_paths(&external, &trusted_roots);
}
