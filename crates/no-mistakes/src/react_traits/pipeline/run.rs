use crate::react_traits::analyze::file::analyze_file_from_visible;
use crate::react_traits::pipeline::glob::expand_globs_from_files;
use crate::react_traits::report::types::{AggregatedFacts, ComponentFacts, FileConfig, RootConfig};
use anyhow::Result;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn run_analyze(
    root: &Path,
    config_path: Option<&Path>,
    targets: &[String],
    depth: Option<usize>,
) -> Result<Vec<ComponentFacts>> {
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    let stems = [".no-mistakes"];
    let root_config: RootConfig =
        crate::config::load_config_from_visible(&root, config_path, &stems, &visible_paths)?;
    let file_config = root_config.into_file_config();
    run_analyze_inner_from_visible(&root, &file_config, targets, depth, &visible_paths)
}

/// Discover the candidate React source files for an analysis run.
///
/// Expands `targets` (defaulting to all TS/JS extensions) from `root` first, and
/// only falls back to the configured `frontend_root` when the root yields no
/// matches. Shared by `run_analyze` and `run_usages` so both scan the same
/// universe.
pub(crate) fn discover_react_files_from_visible(
    root: &Path,
    file_config: &FileConfig,
    targets: &[String],
    visible_paths: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let frontend_root = root.join(file_config.frontend_root.as_deref().unwrap_or("app"));
    let default_targets;
    let targets = if targets.is_empty() {
        default_targets = vec![
            "**/*.tsx".to_string(),
            "**/*.ts".to_string(),
            "**/*.jsx".to_string(),
            "**/*.js".to_string(),
        ];
        default_targets.as_slice()
    } else {
        targets
    };
    // Expand globs from root first; only fall back to frontend_root when root yields no matches.
    // Skip the frontend_root.exists() gate entirely when patterns match at root level.
    let from_root = expand_globs_from_files(&root, targets, visible_paths)?;
    if !from_root.is_empty() {
        return Ok(from_root);
    }
    if !frontend_root.exists() {
        anyhow::bail!("frontend root not found: {}", frontend_root.display());
    }
    expand_globs_from_files(&frontend_root, targets, visible_paths)
}

pub(crate) fn run_analyze_inner_from_visible(
    root: &Path,
    file_config: &FileConfig,
    targets: &[String],
    _depth: Option<usize>,
    visible_paths: &[PathBuf],
) -> Result<Vec<ComponentFacts>> {
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let visible_files = visible_paths
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<HashSet<_>>();
    let files = discover_react_files_from_visible(&root, file_config, targets, visible_paths)?;

    let mut file_cache: HashMap<PathBuf, Vec<ComponentFacts>> = HashMap::new();
    let analyses = files
        .par_iter()
        .map(|file| {
            analyze_file_from_visible(file, &root, &visible_files)
                .map(|analysis| (file.clone(), analysis.components))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut results = Vec::new();
    for (file, components) in analyses {
        file_cache.insert(
            crate::codebase::ts_resolver::normalize_path(&file),
            components.as_ref().clone(),
        );
        results.extend(components.iter().cloned());
    }

    let mut all_results = Vec::new();
    for mut facts in results {
        let agg = aggregate_children_from_visible(
            &facts,
            &mut file_cache,
            &root,
            &visible_files,
            &mut HashSet::new(),
        );
        if agg != AggregatedFacts::default() {
            facts.inherited_from_children = Some(agg);
        }
        all_results.push(facts);
    }

    Ok(all_results)
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

fn aggregate_children_from_visible(
    facts: &ComponentFacts,
    file_cache: &mut HashMap<PathBuf, Vec<ComponentFacts>>,
    root: &Path,
    visible_files: &HashSet<PathBuf>,
    visited: &mut HashSet<String>,
) -> AggregatedFacts {
    aggregate_children_inner(facts, file_cache, root, Some(visible_files), visited)
}

fn aggregate_children_inner(
    facts: &ComponentFacts,
    file_cache: &mut HashMap<PathBuf, Vec<ComponentFacts>>,
    root: &Path,
    visible_files: Option<&HashSet<PathBuf>>,
    visited: &mut HashSet<String>,
) -> AggregatedFacts {
    let mut agg = AggregatedFacts::default();
    for child_ref in &facts.children {
        let key = format!("{}#{}", child_ref.file, child_ref.name);
        if visited.contains(&key) {
            continue;
        }
        visited.insert(key.clone());
        let child_path = crate::codebase::ts_resolver::normalize_path(&root.join(&child_ref.file));
        if visible_files.is_some_and(|visible| !visible.contains(&child_path)) {
            continue;
        }
        // Analyze on-demand and cache so repeated child refs avoid redundant parsing (Cgv-B).
        if !file_cache.contains_key(&child_path) {
            let analysis = match visible_files {
                Some(visible) => analyze_file_from_visible(&child_path, root, visible),
                None => crate::react_traits::analyze::file::analyze_file(&child_path, root),
            };
            match analysis {
                Ok(a) => {
                    file_cache.insert(child_path.clone(), a.components.as_ref().clone());
                }
                Err(_) => continue,
            }
        }
        // Clone only the matching component (not the whole Vec) so the borrow of
        // file_cache is dropped before the recursive mutable borrow in aggregate_children.
        let child_facts_opt = file_cache
            .get(&child_path)
            .and_then(|comps| comps.iter().find(|c| c.name == child_ref.name))
            .cloned();
        if let Some(child_facts) = child_facts_opt {
            agg.has_state |= child_facts.has_state;
            agg.has_props |= child_facts.has_props;
            agg.passes_props |= child_facts.passes_props;
            agg.uses_memo |= child_facts.uses_memo;
            agg.uses_context_provider |= child_facts.uses_context_provider;
            agg.uses_suspense |= child_facts.uses_suspense;
            agg.has_fetch |= !child_facts.fetches.is_empty();
            let child_agg =
                aggregate_children_inner(&child_facts, file_cache, root, visible_files, visited);
            agg.has_state |= child_agg.has_state;
            agg.has_fetch |= child_agg.has_fetch;
            agg.uses_suspense |= child_agg.uses_suspense;
            agg.uses_context_provider |= child_agg.uses_context_provider;
            agg.uses_memo |= child_agg.uses_memo;
            agg.has_props |= child_agg.has_props;
            agg.passes_props |= child_agg.passes_props;
        }
    }
    agg
}
