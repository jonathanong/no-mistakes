use crate::codebase::rules::{
    coverage_producer_assertion_owner, test_no_live_repository_scan,
    test_no_unmocked_dynamic_imports::config as dynamic_config, test_reliability,
    vitest_default_excludes, vitest_project_policy,
};
use crate::codebase::test_discovery::traversal;
use crate::codebase::testing_analysis::{self, Hooks};

static TESTING_HOOKS: Hooks = Hooks {
    prepare_framework_catalog: traversal::prepare_tsconfig_catalog_with_framework_projects,
    excluded_config_paths: traversal::excluded_config_paths,
    test_filters_from_prepared: traversal::test_filters_from_prepared,
    new_test_filter: traversal::new_test_filter,
    test_filter_from_visible: traversal::test_filter_from_visible,
    legacy_filter_from_visible: dynamic_config::test_filter_from_visible,
    needs_playwright_configs: dynamic_config::needs_playwright_configs,
    needs_vitest_configs: dynamic_config::needs_vitest_configs,
    vitest_roots_configured: test_no_live_repository_scan::vitest_roots_configured,
    collect_reliability: test_reliability::ast::collect,
    collect_fixture_initializers: test_reliability::ast::fixture_initializers,
    has_vitest_candidate_exports: vitest_default_excludes::has_candidate_exports,
    collect_vitest_excludes: vitest_default_excludes::collect_prepared,
    vitest_policy_diagnostics: vitest_project_policy::diagnostics_with_files_and_sources,
    assertion_ownership_edges: coverage_producer_assertion_owner::ownership_edges,
};

ctor::declarative::ctor! {
    // SAFETY: The constructor only installs an immutable table of function pointers.
    #[ctor(unsafe)]
    fn register_testing_analysis() {
        testing_analysis::register(&TESTING_HOOKS);
    }
}
