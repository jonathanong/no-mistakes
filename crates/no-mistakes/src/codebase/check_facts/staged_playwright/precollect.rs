use super::super::{CheckFactPlan, PlaywrightFactPlan};
use crate::codebase::ts_source::FileIdMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

pub(super) fn cached_config_file_facts(
    session: crate::codebase::check_facts::FactRequest<'_>,
    root: &Path,
    files: &[PathBuf],
    graph_files: &[PathBuf],
    plan: &CheckFactPlan,
    playwright: &PlaywrightFactPlan,
    sources: &crate::codebase::ts_source::SourceStore,
) -> FileIdMap<super::super::CheckFileFacts> {
    crate::perf_trace::trace("parse", || {
        let universe = files.iter().chain(graph_files).collect::<HashSet<_>>();
        let collected = playwright
            .config_files()
            .iter()
            .filter(|path| {
                !plan
                    .integration_runner_configs
                    .as_ref()
                    .is_some_and(|runner| runner.contains(path))
            })
            .filter(|path| universe.contains(path) || plan.legacy_symbol_paths.contains(*path))
            .filter_map(|path| {
                let source = sources.read_path(path).ok()?;
                let variants = [super::super::file::CheckFactVariant {
                    collect_tests: session.collect_tests(plan, path),
                    root,
                    plan,
                    playwright: Some(playwright),
                }];
                let facts =
                    super::super::file::collect_file_fact_variants_from_source_with_session(
                        session.session,
                        path,
                        source,
                        &variants,
                    )
                    .into_iter()
                    .next()
                    .flatten()?;
                Some((path.clone(), facts))
            });
        FileIdMap::from_iter_with_inventory(collected, std::sync::Arc::clone(sources.inventory()))
    })
}
