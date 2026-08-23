use super::super::{CheckFactPlan, PlaywrightFactPlan};
use crate::codebase::ts_source::FileIdMap;
use std::collections::HashSet;
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
) -> FileIdMap<super::super::CheckFileFacts> {
    if !crate::ast::request_parse_cache_active() {
        return FileIdMap::with_inventory(std::sync::Arc::clone(sources.inventory()));
    }
    let universe = files.iter().chain(graph_files).collect::<HashSet<_>>();
    let collected = playwright
        .config_files()
        .iter()
        .filter(|path| universe.contains(path) || plan.legacy_symbol_paths.contains(*path))
        .filter_map(|path| {
            let source = sources.read_path(path).ok()?;
            let variants = [super::super::file::CheckFactVariant {
                root,
                plan,
                playwright: Some(playwright),
            }];
            let facts = super::super::file::collect_file_fact_variants_from_source_with_session(
                session, path, source, &variants,
            )
            .into_iter()
            .next()
            .flatten()?;
            Some((path.clone(), facts))
        });
    FileIdMap::from_iter_with_inventory(collected, std::sync::Arc::clone(sources.inventory()))
}
