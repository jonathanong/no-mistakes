use super::{
    complete_domain_checks, empty_results, enabled, fact_collection, finite_set_plan, graph_plan,
    prepared, results, CheckResults,
};
use crate::check_parallel::{run_domain_checks, DomainCheckInputs};
use crate::check_tasks;
use anyhow::{Context, Result};
use enabled::{fact_plan, integration_configured};
use std::path::PathBuf;

#[allow(dead_code)]
pub(crate) fn run_all(
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
) -> Result<CheckResults> {
    run_all_with_suppressed(root, config_path, tsconfig_path, false)
}

pub(crate) fn run_all_with_suppressed(
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
    include_suppressed: bool,
) -> Result<CheckResults> {
    let root = root.canonicalize().unwrap_or(root);
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::new(
        no_mistakes::diagnostics::current(),
    );
    let prepared = prepared::prepare_with_session(
        &session,
        &root,
        config_path.as_deref(),
        tsconfig_path.as_deref(),
    )?;
    let config_path = prepared.config_path.clone();
    let config = &prepared.config;
    let queues_enabled = check_tasks::queues_configured(config);
    let unique_exports_enabled = check_tasks::unique_exports_configured(config);
    let enabled = enabled::ConfiguredChecks::from_config(config);
    let filesystem_rules_enabled = check_tasks::filesystem_rules_configured(config);
    let canonical_graph_plan = no_mistakes::codebase::rules::canonical_graph_plan(config);
    let graph_requires_full_file_universe =
        no_mistakes::codebase::rules::canonical_graph_requires_full_file_universe(config);
    let playwright_consumers = canonical_graph_plan
        .map(
            |plan| no_mistakes::playwright::rules::PlaywrightFactConsumers {
                graph_selectors: plan.playwright_selectors,
                graph_routes: plan.playwright_routes,
            },
        )
        .unwrap_or_default();
    let mut playwright_fact_plan = match prepared.playwright.as_ref() {
        Some(prepared) => Some(prepared.fact_plan()),
        None => no_mistakes::playwright::rules::fact_plan_for_consumers(
            &root,
            config_path.as_deref(),
            config,
            playwright_consumers,
        )
        .context("failed to prepare Playwright shared facts")?,
    };
    let integration_enabled = integration_configured(config);
    let react_enabled = prepared.react.enabled();
    let mut plan = fact_plan(enabled::EnabledChecks {
        react: react_enabled,
        queue: queues_enabled,
        queue_factory_names: config.queues.factories.clone(),
        dynamic_import_rules: enabled.dynamic_import_rules,
        boundary_rules: enabled.boundary_rules,
        nextjs_api_routes: enabled.nextjs_api_routes,
        nextjs_caching: enabled.nextjs_caching,
        storybook_stories: enabled.storybook_stories,
        integration: integration_enabled,
        unique_exports: unique_exports_enabled,
    });
    if integration_enabled {
        plan.integration_runner_configs = Some(std::sync::Arc::new(
            no_mistakes::integration_tests::prepare_runner_configs_with_catalog(
                &root,
                config,
                prepared.visible_paths.paths_for(&root).as_ref(),
                std::sync::Arc::clone(&prepared.tsconfig_catalog),
                prepared.visible_paths.source_store_for(&root),
            ),
        ));
    }
    let prepared_graph = graph_plan::prepare(
        &root,
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
    )?;
    let fact_demand = finite_set_plan::prepare(
        &root,
        config,
        &mut plan,
        canonical_graph_plan.is_some(),
        playwright_fact_plan.is_some(),
    );
    let needs_shared_facts = fact_demand.needs_shared_facts();
    if finite_set_plan::no_analysis_requested(
        needs_shared_facts,
        filesystem_rules_enabled,
        no_mistakes::playwright::rules::configured(config),
    ) {
        return Ok(empty_results([None]));
    }
    let (views, discover_duration) = no_mistakes::diagnostics::measure_if_enabled(
        "discovery",
        no_mistakes::diagnostics::TimingKind::Serial,
        || {
            crate::check_discovery::discover_check_file_views_from_snapshot(
                &root,
                config,
                &config.filesystem.skip_directories,
                unique_exports_enabled,
                prepared.visible_paths.as_ref(),
            )
        },
    );
    let (discovered, graph_files) = crate::check_discovery::select_graph_files(
        views,
        needs_shared_facts,
        graph_requires_full_file_universe,
        playwright_fact_plan.is_some(),
        enabled.dynamic_import_rules,
    );
    let sources = prepared.visible_paths.source_store_for(&root);
    let ((fs_files, facts), facts_duration) =
        fact_collection::collect(fact_collection::CollectInput {
            session: &session,
            root: &root,
            discovered,
            graph_files,
            needs_shared_facts,
            filesystem_rules_enabled,
            fact_demand: &fact_demand,
            plan,
            playwright_fact_plan,
            sources: std::sync::Arc::clone(&sources),
        });
    no_mistakes::invocation::check_timeout()?;
    let (react, queues, rules, integration, codebase, filesystem_rules) =
        run_domain_checks(DomainCheckInputs {
            session: session.clone(),
            root: &root,
            config_path: &config_path,
            tsconfig_path: &tsconfig_path,
            react_enabled,
            queues_enabled,
            integration_enabled,
            unique_exports_enabled,
            filesystem_rules_enabled,
            discovered_files: &fs_files,
            facts: &facts,
            prepared_playwright: prepared.playwright.as_ref(),
            prepared_react: &prepared.react,
            prepared_graph: prepared_graph.as_ref(),
            dependency_graph: None,
            prepared_tsconfig: &prepared.tsconfig,
            prepared_tsconfig_catalog: &prepared.tsconfig_catalog,
            visible_paths: prepared.visible_paths.as_ref(),
            sources: std::sync::Arc::clone(&sources),
            inferred_roots: &prepared.inferred_roots,
            config,
            codebase_config: &prepared.codebase_config,
            vitest_projects: prepared.vitest_projects.as_ref(),
            workflow_documents: prepared.workflow_documents.as_deref(),
            tsconfig_gate_project_inputs: prepared.tsconfig_gate_project_inputs.as_ref(),
            defer_suppression: true,
        });
    no_mistakes::invocation::check_timeout()?;
    results::finalize_domain_checks(results::FinalizeInput {
        root: &root,
        config,
        filesystem_files: &fs_files,
        sources: &sources,
        filesystem_rules_enabled,
        react_warning: None,
        discover_duration,
        facts_duration,
        completed: complete_domain_checks((
            react,
            queues,
            rules,
            integration,
            codebase,
            filesystem_rules,
        ))?,
        include_suppressed,
    })
}
