use crate::react_traits::pipeline::glob::expand_globs_from_files;
use crate::react_traits::report::types::{AggregatedFacts, ComponentFacts, FileConfig};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

mod facts_only;
mod prerequisites;

pub(crate) fn run_analyze_with_loaded_config_and_facts(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    targets: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<ComponentFacts>> {
    let file_config = super::check::file_config_from_loaded(config);
    run_analyze_inner_with_facts(root, &file_config, targets, shared)
}

pub(crate) fn run_analyze_inner_with_facts(
    root: &Path,
    file_config: &FileConfig,
    targets: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<ComponentFacts>> {
    facts_only::run(root, file_config, targets, shared)
}

pub(crate) struct PreparedComponentFacts {
    pub(crate) facts: ComponentFacts,
    pub(crate) inherited_fetch_locations: Vec<(String, usize)>,
}

pub(crate) fn run_with_session(
    root: &Path,
    file_config: &FileConfig,
    targets: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
) -> Result<Vec<PreparedComponentFacts>> {
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let files = target_files(&root, file_config, targets, shared.files())?;
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
    let child_path_index = child_path_index(&root, &file_cache);
    let source_index = (!parse_errors.is_empty()).then(|| prerequisites::source_index(shared));
    let mut all_results = Vec::new();
    for file in files {
        if let Some(error) = parse_errors.get(&file) {
            prerequisites::accept(session, &[(file.clone(), error.as_str())])?;
            continue;
        }
        let Some(components) = file_cache.get(&file).cloned() else {
            continue;
        };
        for mut facts in components.iter().cloned() {
            if let Some(source_index) = &source_index {
                if !prerequisites::accept(
                    session,
                    &prerequisites::child_failures(&facts, &root, source_index),
                )? {
                    continue;
                }
            }
            let agg = aggregate_children_cached(
                &facts,
                &file_cache,
                &child_path_index,
                &mut HashSet::new(),
            );
            if agg.facts != AggregatedFacts::default() {
                facts.inherited_from_children = Some(agg.facts);
            }
            all_results.push(PreparedComponentFacts {
                facts,
                inherited_fetch_locations: agg.fetch_locations,
            });
        }
    }
    Ok(all_results)
}

fn target_files(
    root: &Path,
    file_config: &FileConfig,
    targets: &[String],
    visible_paths: &[PathBuf],
) -> Result<Vec<PathBuf>> {
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
    let from_root = expand_globs_from_files(root, targets, visible_paths)?;
    if !from_root.is_empty() {
        return Ok(from_root);
    }
    if !frontend_root.exists() {
        anyhow::bail!("frontend root not found: {}", frontend_root.display());
    }
    expand_globs_from_files(&frontend_root, targets, visible_paths)
}

fn aggregate_children_cached(
    facts: &ComponentFacts,
    file_cache: &HashMap<PathBuf, std::sync::Arc<Vec<ComponentFacts>>>,
    child_path_index: &HashMap<String, PathBuf>,
    visited: &mut HashSet<String>,
) -> AggregateResult {
    let mut agg = AggregateResult::default();
    for child_ref in &facts.children {
        let key = format!("{}#{}", child_ref.file, child_ref.name);
        if !visited.insert(key) {
            continue;
        }
        let normalized_child_file =
            crate::codebase::ts_source::normalize_discovery_path(Path::new(&child_ref.file));
        let child_facts_opt = child_path_index
            .get(&child_ref.file)
            .or(child_path_index.get(normalized_child_file.to_string_lossy().as_ref()))
            .and_then(|path| file_cache.get(path))
            .and_then(|comps| comps.iter().find(|c| c.name == child_ref.name));
        if let Some(child_facts) = child_facts_opt {
            merge_component(&mut agg, child_facts);
            let child_agg =
                aggregate_children_cached(child_facts, file_cache, child_path_index, visited);
            merge_aggregate(&mut agg, &child_agg);
        }
    }
    agg
}

#[derive(Default)]
struct AggregateResult {
    facts: AggregatedFacts,
    fetch_locations: Vec<(String, usize)>,
}

pub(super) fn child_path_index(
    root: &Path,
    file_cache: &HashMap<PathBuf, std::sync::Arc<Vec<ComponentFacts>>>,
) -> HashMap<String, PathBuf> {
    let mut index = HashMap::new();
    for path in file_cache.keys() {
        index.insert(path.to_string_lossy().to_string(), path.clone());
        index.insert(
            crate::codebase::ts_source::relative_slash_path(root, path),
            path.clone(),
        );
    }
    index
}

include!("run_with_facts/merge.rs");

#[cfg(test)]
mod tests;
