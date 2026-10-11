use super::super::{enabled, finite_set_plan, graph_plan, prepared::PreparedCheckInputs};
use crate::check_tasks;
use anyhow::Result;
use enabled::{fact_plan, integration_configured};
use no_mistakes::codebase::check_facts::{CheckFactPlan, PlaywrightFactPlan};
use no_mistakes::codebase::dependencies::graph::PreparedGraphConfig;
use std::path::Path;

pub(super) struct CheckPlan {
    pub(super) queues_enabled: bool,
    pub(super) unique_exports_enabled: bool,
    pub(super) filesystem_rules_enabled: bool,
    pub(super) dynamic_import_rules: bool,
    pub(super) graph_requires_full_file_universe: bool,
    pub(super) playwright_fact_plan: Option<PlaywrightFactPlan>,
    pub(super) integration_enabled: bool,
    pub(super) react_enabled: bool,
    pub(super) plan: CheckFactPlan,
    pub(super) prepared_graph: Option<PreparedGraphConfig>,
    pub(super) fact_demand: finite_set_plan::PreparedFactDemand,
}

pub(super) fn build(
    root: &Path,
    prepared: &PreparedCheckInputs,
    deadlines: bool,
) -> Result<CheckPlan> {
    let config = &prepared.config;
    let queues_enabled = check_tasks::queues_configured(config);
    let unique_exports_enabled = check_tasks::unique_exports_configured(config);
    let enabled = enabled::ConfiguredChecks::from_config(config);
    let filesystem_rules_enabled = check_tasks::filesystem_rules_configured(config);
    let canonical_graph_plan = no_mistakes::codebase::rules::try_canonical_graph_plan(config)?;
    let graph_requires_full_file_universe =
        no_mistakes::codebase::rules::canonical_graph_requires_full_file_universe(config);
    let playwright_built = super::playwright::fact_plan(
        root,
        prepared.config_path.as_deref(),
        config,
        canonical_graph_plan,
        prepared.playwright.as_ref(),
    );
    let mut playwright_fact_plan = playwright_built?;
    let integration_enabled = integration_configured(config);
    let react_enabled = prepared.react.enabled();
    let dynamic_import_rules = enabled.dynamic_import_rules;
    let mut plan = fact_plan(enabled::EnabledChecks {
        react: react_enabled,
        queue: queues_enabled,
        queue_factory_names: config.queues.factories.clone(),
        dynamic_import_rules,
        boundary_rules: enabled.boundary_rules,
        nextjs_api_routes: enabled.nextjs_api_routes,
        nextjs_caching: enabled.nextjs_caching,
        storybook_stories: enabled.storybook_stories,
        integration: integration_enabled,
        unique_exports: unique_exports_enabled,
        embedded_sql: enabled.embedded_sql,
    });
    plan.query_annotation_catalog = Some(std::sync::Arc::clone(&prepared.tsconfig_catalog));
    no_mistakes::codebase::postgres::configure_prepared_postgres_plan(config, &mut plan)?;
    plan.embedded_sql_options =
        no_mistakes::codebase::postgres::configured_embedded_sql_options_for_checks(config)?;
    let schema_catalog_paths = no_mistakes::codebase::postgres::configured_schema_catalog_paths(
        config,
        no_mistakes::codebase::postgres::SCHEMA_CATALOG_RULE_IDS,
    );
    plan.postgres_schema_catalog_paths = schema_catalog_paths?;
    if integration_enabled || deadlines {
        plan.integration_runner_configs = Some(std::sync::Arc::new((if deadlines {
            no_mistakes::integration_tests::prepare_runner_configs_with_deadline_evidence
        } else {
            no_mistakes::integration_tests::prepare_runner_configs_with_catalog
        })(
            root,
            config,
            prepared.visible_paths.paths_for(root).as_ref(),
            std::sync::Arc::clone(&prepared.tsconfig_catalog),
            prepared.visible_paths.source_store_for(root),
        )));
    }
    let prepared_graph_result = graph_plan::prepare(
        root,
        config,
        graph_plan::PreparedInputs {
            codebase_config: &prepared.codebase_config,
            tsconfig: &prepared.tsconfig,
            visible_paths: prepared.visible_paths.as_ref(),
            workflow_documents: prepared.workflow_documents.as_ref(),
        },
        canonical_graph_plan,
        &mut playwright_fact_plan,
        &mut plan,
    );
    let prepared_graph = prepared_graph_result?;
    let fact_demand_result = finite_set_plan::prepare(
        root,
        config,
        &mut plan,
        canonical_graph_plan.is_some(),
        playwright_fact_plan.is_some(),
    );
    let fact_demand = fact_demand_result?;
    Ok(CheckPlan {
        queues_enabled,
        unique_exports_enabled,
        filesystem_rules_enabled,
        dynamic_import_rules,
        graph_requires_full_file_universe,
        playwright_fact_plan,
        integration_enabled,
        react_enabled,
        plan,
        prepared_graph,
        fact_demand,
    })
}
