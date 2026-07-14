use crate::check_parallel::{run_domain_checks, DomainCheckInputs};
use crate::check_tasks::{
    filesystem_rules_configured, forbidden_dependencies_configured, queues_configured,
    unique_exports_configured,
};
use anyhow::{Context, Result};
use enabled::{fact_plan, integration_configured, plan_requests_facts};
use no_mistakes::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources;
use std::path::PathBuf;
use std::time::Instant;

pub(crate) mod enabled;
pub(crate) mod prepared;
mod results;

pub(crate) use results::{complete_domain_checks, empty_results, json_value, CheckResults};

pub(crate) fn run_all(
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
) -> Result<CheckResults> {
    let root = root.canonicalize().unwrap_or(root);
    let prepared = prepared::prepare(&root, config_path.as_deref(), tsconfig_path.as_deref())?;
    let config = &prepared.config;
    let queues_enabled = queues_configured(config);
    let unique_exports_enabled = unique_exports_configured(config);
    let enabled = enabled::ConfiguredChecks::from_config(config);
    let filesystem_rules_enabled = filesystem_rules_configured(config);
    let forbidden_deps_enabled = forbidden_dependencies_configured(config);
    let forbidden_graph_plan = if forbidden_deps_enabled {
        no_mistakes::codebase::rules::forbidden_dependencies::graph_plan(config)
    } else {
        None
    };
    let playwright_consumers = forbidden_graph_plan
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
    let playwright_facts_enabled = playwright_fact_plan.is_some();
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
            no_mistakes::integration_tests::prepare_runner_configs(
                &root,
                config,
                prepared.visible_paths.paths_for(&root).as_ref(),
                &prepared.tsconfig,
            ),
        ));
    }
    let prepared_graph = forbidden_graph_plan
        .map(|graph_plan| {
            no_mistakes::codebase::dependencies::graph::prepare_graph_config(
                &root,
                graph_plan,
                &prepared.codebase_config,
                config,
                prepared.visible_paths.as_ref(),
            )
        })
        .transpose()?;
    if let Some(graph_playwright) = prepared_graph
        .as_ref()
        .map(|graph| graph.playwright_fact_plan(&root, prepared.visible_paths.as_ref()))
        .transpose()?
        .flatten()
    {
        match playwright_fact_plan.as_mut() {
            Some(plan) => plan.include(graph_playwright),
            None => playwright_fact_plan = Some(graph_playwright),
        }
    }
    if let (Some(graph_plan), Some(prepared)) = (forbidden_graph_plan, prepared_graph.as_ref()) {
        let (fact_plan, fact_context) =
            no_mistakes::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
                &root,
                graph_plan,
                prepared,
            );
        plan.graph.include(fact_plan);
        plan.graph_context = fact_context;
    }
    let needs_shared_facts =
        plan_requests_facts(&plan) || playwright_fact_plan.is_some() || forbidden_deps_enabled;
    if !needs_shared_facts
        && !filesystem_rules_enabled
        && !no_mistakes::playwright::rules::configured(config)
        && !forbidden_deps_enabled
    {
        return Ok(empty_results([None]));
    }
    let discover_start = Instant::now();
    let skip_directories = config.filesystem.skip_directories.clone();
    let needs_full_graph_files = forbidden_graph_plan.is_some() || playwright_facts_enabled;
    let needs_graph_files =
        needs_shared_facts && (needs_full_graph_files || enabled.dynamic_import_rules);
    let views = crate::check_discovery::discover_check_file_views_from_snapshot(
        &root,
        config,
        &skip_directories,
        unique_exports_enabled,
        prepared.visible_paths.as_ref(),
    );
    let (discovered, graph_files) = if needs_full_graph_files {
        (views.filesystem, views.graph)
    } else if needs_graph_files {
        // The dynamic-import rule traverses the same filesystem-scoped
        // visible universe it analyzes. Supplying that universe explicitly
        // keeps prepared graph construction strict without a fallback parse.
        let graph_files = views.filesystem.clone();
        (views.filesystem, graph_files)
    } else {
        (views.filesystem, Vec::new())
    };
    let discover_duration = discover_start.elapsed();
    let facts_start = Instant::now();
    // When only filesystem rules are enabled, no TS/JS parsing is needed.
    let (fs_files, facts) = if needs_shared_facts {
        let fs = if filesystem_rules_enabled {
            discovered.clone()
        } else {
            Vec::new()
        };
        let f = collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            discovered,
            graph_files,
            plan,
            playwright_fact_plan,
            prepared.visible_paths.source_store_for(&root),
        );
        (fs, f)
    } else {
        (discovered, Default::default())
    };
    let facts_duration = facts_start.elapsed();

    let (react, queues, rules, integration, codebase, filesystem_rules) =
        run_domain_checks(DomainCheckInputs {
            root: &root,
            config_path: &config_path,
            tsconfig_path: &tsconfig_path,
            react_enabled,
            queues_enabled,
            unique_exports_enabled,
            filesystem_rules_enabled,
            discovered_files: fs_files.clone(),
            facts: &facts,
            prepared_playwright: prepared.playwright.as_ref(),
            prepared_react: &prepared.react,
            prepared_graph: prepared_graph.as_ref(),
            dependency_graph: None,
            prepared_tsconfig: &prepared.tsconfig,
            visible_paths: prepared.visible_paths.as_ref(),
            sources: prepared.visible_paths.source_store_for(&root),
            inferred_roots: &prepared.inferred_roots,
            config,
            codebase_config: &prepared.codebase_config,
            vitest_projects: prepared.vitest_projects.as_ref(),
        });

    results::finalize_domain_checks(results::FinalizeInput {
        root: &root,
        config,
        filesystem_files: &fs_files,
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
    })
}

#[cfg(test)]
mod tests;
