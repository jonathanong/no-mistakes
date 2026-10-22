use crate::check_runner::enabled::{integration_configured, ConfiguredChecks};
use crate::check_tasks::{queues_configured, unique_exports_configured};
use no_mistakes::codebase::rules;
use no_mistakes::config::v2::schema::ProjectType;
use no_mistakes::config::v2::NoMistakesConfig;
use no_mistakes::playwright;

const CATALOG_RULES: &[&str] = &[
    rules::FINITE_SET_CONSISTENCY,
    rules::FORBIDDEN_CALLS,
    rules::FORBIDDEN_DEPENDENCIES,
    rules::INTEGRATION_TEST_NO_MOCKS,
    rules::NEXTJS_NO_API_ROUTES,
    rules::NEXTJS_NO_CACHING,
    rules::POSTGRES_CONFLICT_ORDERING,
    rules::POSTGRES_EXPLICIT_COLUMNS,
    rules::POSTGRES_LOCK_ORDERING,
    rules::POSTGRES_REQUIRE_QUERY_ANNOTATION,
    rules::POSTGRES_REQUIRED_PREDICATES,
    rules::PRODUCTION_DEPENDENCY_DECLARATIONS,
    rules::QUERY_REACHED_PER_ITEM,
    rules::REQUIRE_STORYBOOK_STORIES,
    rules::REQUIRED_COMPANION_IMPORTS,
    rules::REQUIRED_ENTRYPOINT_REACHABILITY,
    rules::SERVER_ROUTE_CLIENT_BOUNDARY,
    rules::TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
    rules::TSCONFIG_ALIAS_FOLDER_MAPPING,
    rules::TSCONFIG_FILE_COVERAGE,
    rules::TSCONFIG_GATE_COVERAGE,
    rules::UNCONSTRUCTED_ERROR_CLASS,
    rules::VITEST_CI_PATH_COVERAGE,
    rules::VITEST_PROJECT_MAPPING,
    rules::VITEST_TEST_CORRESPONDENCE,
];

pub(crate) fn needs_tsconfig_catalog(config: &NoMistakesConfig, deadlines: bool) -> bool {
    playwright::rules::configured(config)
        || unique_exports_configured(config)
        || queues_configured(config)
        || integration_configured(config)
        || react_enabled(config)
        || catalog_rule_configured(config)
        || (deadlines && declared_runner_configs(config))
        || !matches!(
            no_mistakes::codebase::rules::try_canonical_graph_plan(config),
            Ok(None)
        )
}

pub(crate) fn needs_workspace(
    config: &NoMistakesConfig,
    tsconfig_path: Option<&std::path::Path>,
    deadlines: bool,
) -> bool {
    config.rule_configured(rules::TSCONFIG_GATE_COVERAGE)
        || playwright::rules::configured(config)
        || (tsconfig_path.is_none() && needs_tsconfig_catalog(config, deadlines))
}

pub(crate) fn needs_inferred_roots(config: &NoMistakesConfig, deadlines: bool) -> bool {
    needs_tsconfig_catalog(config, deadlines) || config.projects.values().any(infers_framework_root)
}

fn infers_framework_root(project: &no_mistakes::config::v2::schema::Project) -> bool {
    matches!(
        project.type_,
        Some(ProjectType::Nextjs | ProjectType::Remix | ProjectType::Vitejs)
    )
}

fn catalog_rule_configured(config: &NoMistakesConfig) -> bool {
    let enabled = ConfiguredChecks::from_config(config);
    enabled.dynamic_import_rules
        || enabled.boundary_rules
        || enabled.nextjs_api_routes
        || enabled.nextjs_caching
        || enabled.storybook_stories
        || prepared_embedded_sql_configured(config)
        || CATALOG_RULES
            .iter()
            .any(|rule| config.rule_configured(rule))
}

fn prepared_embedded_sql_configured(config: &NoMistakesConfig) -> bool {
    no_mistakes::codebase::postgres::PREPARED_EMBEDDED_SQL_RULE_IDS
        .iter()
        .any(|rule| config.rule_configured(rule))
}

fn declared_runner_configs(config: &NoMistakesConfig) -> bool {
    config.tests.vitest.configs.is_some() || config.tests.playwright.configs.is_some()
}

fn react_enabled(config: &NoMistakesConfig) -> bool {
    no_mistakes::react_traits::prepare_check_from_loaded_config(config, false).enabled()
}
