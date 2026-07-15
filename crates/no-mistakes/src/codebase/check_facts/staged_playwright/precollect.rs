use super::super::{CheckFactPlan, PlaywrightFactPlan};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::path::PathBuf;

pub(super) fn cached_config_file_facts(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    files: &[PathBuf],
    graph_files: &[PathBuf],
    plan: &CheckFactPlan,
    playwright: &PlaywrightFactPlan,
    sources: &crate::codebase::ts_source::SourceStore,
) -> HashMap<PathBuf, super::super::CheckFileFacts> {
    if !crate::ast::request_parse_cache_active() {
        return HashMap::new();
    }
    let universe = files.iter().chain(graph_files).collect::<HashSet<_>>();
    playwright
        .config_files()
        .iter()
        .filter(|path| universe.contains(path) || plan.legacy_symbol_paths.contains(*path))
        .filter_map(|path| {
            let source = sources.read_path(path).ok()?;
            let mut facts = session
                .with_program(path, &source, |program, parsed_source| {
                    super::super::collect_file_facts_from_program(
                        root,
                        path,
                        plan,
                        Some(playwright),
                        parsed_source,
                        program,
                    )
                })
                .ok()?;
            if plan.source || plan.raw_source {
                std::sync::Arc::make_mut(&mut facts.ts).source = Some(source.to_string());
                facts.source = Some(source);
            }
            Some((path.clone(), facts))
        })
        .collect()
}
