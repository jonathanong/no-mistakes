use no_mistakes::codebase::check_facts::CheckFactPlan;

#[derive(Default)]
pub(crate) struct ConfiguredChecks {
    pub(crate) dynamic_import_rules: bool,
    pub(crate) boundary_rules: bool,
    pub(crate) nextjs_api_routes: bool,
    pub(crate) nextjs_caching: bool,
    pub(crate) storybook_stories: bool,
}

impl ConfiguredChecks {
    pub(crate) fn from_config(config: &no_mistakes::config::v2::NoMistakesConfig) -> Self {
        Self {
            dynamic_import_rules: rule_configured(
                config,
                no_mistakes::codebase::rules::TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
            ),
            boundary_rules: rule_configured(
                config,
                no_mistakes::codebase::rules::SERVER_ROUTE_CLIENT_BOUNDARY,
            ),
            nextjs_api_routes: rule_configured(
                config,
                no_mistakes::codebase::rules::NEXTJS_NO_API_ROUTES,
            ),
            nextjs_caching: rule_configured(
                config,
                no_mistakes::codebase::rules::NEXTJS_NO_CACHING,
            ),
            storybook_stories: rule_configured(
                config,
                no_mistakes::codebase::rules::REQUIRE_STORYBOOK_STORIES,
            ),
        }
    }
}

#[derive(Default)]
pub(crate) struct EnabledChecks {
    pub(crate) react: bool,
    pub(crate) queue: bool,
    pub(crate) queue_factory_names: Vec<String>,
    pub(crate) dynamic_import_rules: bool,
    pub(crate) boundary_rules: bool,
    pub(crate) nextjs_api_routes: bool,
    pub(crate) nextjs_caching: bool,
    pub(crate) storybook_stories: bool,
    pub(crate) integration: bool,
    pub(crate) unique_exports: bool,
}

pub(crate) fn fact_plan(enabled: EnabledChecks) -> CheckFactPlan {
    CheckFactPlan {
        imports: enabled.dynamic_import_rules,
        symbols: enabled.unique_exports || enabled.storybook_stories,
        react: enabled.react || enabled.storybook_stories,
        queue: enabled.queue,
        queue_factory_names: enabled.queue_factory_names,
        integration: enabled.integration,
        dynamic_imports: enabled.dynamic_import_rules || enabled.storybook_stories,
        nextjs_caching: enabled.nextjs_caching,
        storybook: enabled.storybook_stories,
        raw_source: enabled.nextjs_api_routes,
        source: enabled.dynamic_import_rules
            || enabled.boundary_rules
            || enabled.nextjs_caching
            || enabled.unique_exports
            || enabled.storybook_stories,
        graph: Default::default(),
        graph_context: Default::default(),
    }
}

pub(crate) fn plan_requests_facts(plan: &CheckFactPlan) -> bool {
    plan.imports
        || plan.symbols
        || plan.react
        || plan.queue
        || plan.integration
        || plan.dynamic_imports
        || plan.nextjs_caching
        || plan.storybook
        || plan.raw_source
        || plan.source
        || !plan.graph.is_empty()
}

pub(crate) fn integration_configured(config: &no_mistakes::config::v2::NoMistakesConfig) -> bool {
    let vitest_configured = config
        .tests
        .vitest
        .projects
        .values()
        .any(|project| !project.integration_suites.is_empty());
    let playwright_configured = config
        .tests
        .playwright
        .projects
        .values()
        .any(|project| !project.integration_suites.is_empty());
    vitest_configured || playwright_configured
}

fn rule_configured(config: &no_mistakes::config::v2::NoMistakesConfig, rule_id: &str) -> bool {
    crate::check_tasks::rule_configured(config, rule_id)
}
