use super::*;
use crate::codebase::ts_source::VisiblePathSnapshot;

pub(crate) fn test_filters_from_prepared(
    root: &Path,
    framework: &str,
    config: &NoMistakesConfig,
    tsconfig: &TsConfig,
    visible_paths: &VisiblePathSnapshot,
    prepared_test_projects: Option<&dyn PreparedProjects>,
) -> Vec<String> {
    let runner = TestRunner::from_name(framework);
    if let Some(runner) = runner {
        let root_visible_paths = visible_paths.paths_for(root);
        let discovered = match prepared_test_projects {
            Some(prepared) => {
                prepared.discover_tests(root, config, runner, &root_visible_paths, tsconfig)
            }
            None if runner == TestRunner::Cargo => super::super::discover_cargo_tests_with_sources(
                root,
                config,
                &root_visible_paths,
                &visible_paths.source_store_for(root),
            ),
            None => super::super::discover_tests_from_visible(
                root,
                config,
                runner,
                &root_visible_paths,
                tsconfig,
            ),
        };
        if let Ok(discovered) = discovered {
            let filters = discovered
                .tests
                .iter()
                .map(|path| {
                    super::super::literal_path_glob(
                        &crate::codebase::ts_source::relative_slash_path(root, path),
                    )
                })
                .collect::<Vec<_>>();
            if filters.is_empty() {
                return test_globs(framework);
            }
            return filters;
        }
    }
    test_globs(framework)
}
