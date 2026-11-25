use super::super::{CheckFactPlan, PlaywrightFactPlan};
use crate::codebase::ts_source::FileIdMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

pub(super) fn cached_config_file_facts(
    session: crate::codebase::check_facts::FactRequest<'_>,
    root: &Path,
    file_scope: (&[PathBuf], &[PathBuf]),
    plan: &CheckFactPlan,
    playwright: &PlaywrightFactPlan,
    sources: &crate::codebase::ts_source::SourceStore,
    precollected: &FileIdMap<super::super::CheckFileFacts>,
) -> FileIdMap<super::super::CheckFileFacts> {
    crate::perf_trace::trace("parse", || {
        let universe = file_scope
            .0
            .iter()
            .chain(file_scope.1)
            .collect::<HashSet<_>>();
        let collected = playwright
            .config_files()
            .iter()
            .filter(|path| {
                !plan
                    .integration_runner_configs
                    .as_ref()
                    .is_some_and(|runner| runner.contains(path))
            })
            .filter(|path| !precollected.contains_key(path))
            .filter_map(|path| {
                let demanded = plan.vitest_default_exclude_paths.contains(path)
                    || plan.csp_allowed_origins_paths.contains(path)
                    || plan.test_reliability_paths.contains_key(path);
                if !demanded && !universe.contains(path) && !plan.legacy_symbol_paths.contains(path)
                {
                    return None;
                }
                let source = match sources.read_source_path(path) {
                    Ok(source) => source,
                    Err(error) if demanded => {
                        return Some((
                            path.clone(),
                            super::super::file_parse_error::read_facts(root, path, plan, error),
                        ));
                    }
                    Err(_) => return None,
                };
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
