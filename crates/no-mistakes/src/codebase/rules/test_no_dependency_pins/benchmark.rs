use super::*;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactMap, CheckFactPlan,
};
use crate::codebase::ts_source::SourceStore;
use std::sync::Arc;

pub struct PreparedDependencyPins {
    root: PathBuf,
    files: Vec<PathBuf>,
    sources: Arc<SourceStore>,
    options: CompiledOptions,
    facts: CheckFactMap,
}

/// Compile the default patterns and extract saved JSX sources in a fresh invocation.
pub fn prepare_dependency_pins(root: &Path) -> Result<PreparedDependencyPins> {
    let files = discover_files(root, &[]);
    let options = compile_options(&Options::default())?;
    let selected: Vec<_> = files
        .into_iter()
        .filter(|path| is_jsx_file(path) && options.includes(&relative_slash_path(root, path)))
        .collect();
    let sources = super::super::source_store_for_files(&selected);
    let facts = collect_check_facts_with_graph_files_playwright_and_sources(
        root,
        selected.clone(),
        vec![],
        CheckFactPlan {
            jsx_text_paths: Arc::new(selected.iter().cloned().collect()),
            ..Default::default()
        },
        None,
        sources.clone(),
    );
    Ok(PreparedDependencyPins {
        root: root.to_path_buf(),
        files: selected,
        sources,
        options,
        facts,
    })
}

pub fn project_dependency_pins(prepared: &PreparedDependencyPins) -> usize {
    prepared
        .files
        .par_iter()
        .map(|path| {
            projection::check_file_with_sources(
                &prepared.root,
                path,
                &prepared.options,
                &prepared.sources,
                Some(&prepared.facts),
                None,
            )
            .len()
        })
        .sum()
}
