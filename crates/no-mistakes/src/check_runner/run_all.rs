use super::{
    complete_domain_checks, empty_results, fact_collection, finite_set_plan, prepared, results,
    CheckResults,
};
use crate::check_parallel::{run_domain_checks, DomainCheckInputs};
use anyhow::Result;
use std::path::PathBuf;

mod playwright;
mod setup;

pub(crate) fn run_all_with_suppressed(
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
    include_suppressed: bool,
) -> Result<CheckResults> {
    no_mistakes::invocation::set_timeout_phase("check.prepare");
    no_mistakes::invocation::check_timeout()?;
    let root = root.canonicalize().unwrap_or(root);
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::new(
        no_mistakes::diagnostics::current(),
    );
    let prepared = prepared::prepare_with_session(
        &session,
        &root,
        config_path.as_deref(),
        tsconfig_path.as_deref(),
    );
    let prepared = prepared?;
    let config_path = prepared.config_path.clone();
    let config = &prepared.config;
    let setup::CheckPlan {
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
    } = setup::build(&root, &prepared)?;
    let needs_shared_facts = fact_demand.needs_shared_facts();
    if finite_set_plan::no_analysis_requested(
        needs_shared_facts,
        filesystem_rules_enabled,
        no_mistakes::playwright::rules::configured(config),
    ) {
        fact_collection::release_extract_programs(&session);
        let mut results = empty_results([None]);
        results.include_suppressed = include_suppressed;
        return Ok(results);
    }
    no_mistakes::invocation::set_timeout_phase("check.discovery");
    no_mistakes::invocation::check_timeout()?;
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
        dynamic_import_rules,
    );
    let sources = prepared.visible_paths.source_store_for(&root);
    // Parse and analysis are the CPU-bound check phases. The deadline names
    // whichever of them is active; it does not drop files from the result.
    no_mistakes::invocation::set_timeout_phase("check.parse");
    no_mistakes::invocation::check_timeout()?;
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
    no_mistakes::invocation::set_timeout_phase("check.analysis");
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
            playwright_projects: prepared.playwright_projects.as_ref(),
            workflow_documents: prepared.workflow_documents.as_deref(),
            tsconfig_gate_project_inputs: prepared.tsconfig_gate_project_inputs.as_ref(),
            // Ordinary checks preserve each domain's early suppression path;
            // only audit requests need cross-domain suppression accounting.
            defer_suppression: include_suppressed,
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
