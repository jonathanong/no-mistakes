use crate::codebase::check_facts::CheckFactMap;
use crate::react_traits::report::types::{AggregatedFacts, ComponentFacts, FileConfig};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The normal prepared-facts analysis path intentionally has no suppression
/// locations. Those private sidecars are constructed only for aggregate check
/// accounting in the sibling pipeline.
pub(super) fn run(
    root: &Path,
    file_config: &FileConfig,
    targets: &[String],
    shared: &CheckFactMap,
) -> Result<Vec<ComponentFacts>> {
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let files = super::target_files(&root, file_config, targets, shared.files())?;
    let mut file_cache = HashMap::new();
    let mut parse_errors = HashMap::new();
    for (path, facts) in &shared.ts {
        let path = crate::codebase::ts_source::normalize_discovery_path(path);
        if let Some(analysis) = &facts.react {
            file_cache.insert(path.clone(), analysis.components.clone());
        }
        if let Some(error) = &facts.parse_error {
            parse_errors.insert(path, error);
        }
    }
    let child_path_index = super::child_path_index(&root, &file_cache);
    let mut all_results = Vec::new();
    for file in files {
        if let Some(error) = parse_errors.get(&file) {
            anyhow::bail!("failed to parse {}: {error}", file.display());
        }
        let Some(components) = file_cache.get(&file) else {
            continue;
        };
        for mut facts in components.iter().cloned() {
            let agg =
                aggregate_children(&facts, &file_cache, &child_path_index, &mut HashSet::new());
            if agg != AggregatedFacts::default() {
                facts.inherited_from_children = Some(agg);
            }
            all_results.push(facts);
        }
    }
    Ok(all_results)
}

fn aggregate_children(
    facts: &ComponentFacts,
    file_cache: &HashMap<PathBuf, std::sync::Arc<Vec<ComponentFacts>>>,
    child_path_index: &HashMap<String, PathBuf>,
    visited: &mut HashSet<String>,
) -> AggregatedFacts {
    let mut agg = AggregatedFacts::default();
    for child_ref in &facts.children {
        let key = format!("{}#{}", child_ref.file, child_ref.name);
        if !visited.insert(key) {
            continue;
        }
        let normalized_child_file =
            crate::codebase::ts_source::normalize_discovery_path(Path::new(&child_ref.file));
        let child_facts = child_path_index
            .get(&child_ref.file)
            .or_else(|| child_path_index.get(normalized_child_file.to_string_lossy().as_ref()))
            .and_then(|path| file_cache.get(path))
            .and_then(|components| components.iter().find(|item| item.name == child_ref.name));
        if let Some(child_facts) = child_facts {
            merge_component(&mut agg, child_facts);
            let child_agg = aggregate_children(child_facts, file_cache, child_path_index, visited);
            merge_aggregate(&mut agg, &child_agg);
        }
    }
    agg
}

fn merge_component(agg: &mut AggregatedFacts, facts: &ComponentFacts) {
    agg.has_state |= facts.has_state;
    agg.has_props |= facts.has_props;
    agg.passes_props |= facts.passes_props;
    agg.uses_memo |= facts.uses_memo;
    agg.uses_context_provider |= facts.uses_context_provider;
    agg.uses_suspense |= facts.uses_suspense;
    agg.has_fetch |= !facts.fetches.is_empty();
}

fn merge_aggregate(agg: &mut AggregatedFacts, child: &AggregatedFacts) {
    agg.has_state |= child.has_state;
    agg.has_fetch |= child.has_fetch;
    agg.uses_suspense |= child.uses_suspense;
    agg.uses_context_provider |= child.uses_context_provider;
    agg.uses_memo |= child.uses_memo;
    agg.has_props |= child.has_props;
    agg.passes_props |= child.passes_props;
}
