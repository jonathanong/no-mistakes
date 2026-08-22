use super::*;
use crate::codebase::dependencies::graph::{GraphBuildPlan, GraphFiles, PreparedGraphBuild};

/// Run the `rsc-callers` query.
pub fn run(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig: Option<&Path>,
    component: &Path,
    depth: Option<usize>,
) -> Result<RscCallersReport> {
    let root = normalize_path(root);
    let root = root.canonicalize().unwrap_or(root);
    let component_abs = if component.is_absolute() {
        component.to_path_buf()
    } else {
        root.join(component)
    };
    if !component_abs.is_file() {
        anyhow::bail!("component file not found: {}", component_abs.display());
    }
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let dataset = session.dataset(&root);
    let visible_paths = dataset.visible_paths_arc();
    let root_visible_paths = dataset.paths_for(&root);
    let sources = dataset.sources_for(&root);
    let mut graph_files = GraphFiles::from_files_with_resource_candidates(
        crate::codebase::ts_source::discover_files_from_visible(&root, &[], &root_visible_paths),
        visible_paths.tracked_paths_for(&root).as_ref().clone(),
    );
    graph_files.add_explicit_root(&component_abs);
    let explicit_tsconfig = tsconfig;
    let (tsconfig, tsconfig_catalog) = match explicit_tsconfig {
        None => {
            let workspace = dataset.workspace();
            let catalog =
                crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources_with_workspace(
                &root,
                std::slice::from_ref(&root),
                &root_visible_paths,
                &sources,
                &workspace,
            );
            let config = catalog.config_for(&component_abs).clone();
            (config, catalog)
        }
        Some(path) => {
            let config = (*dataset.tsconfig(Some(path))?).clone();
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            let catalog = crate::codebase::ts_resolver::TsConfigCatalog::forced(
                &root,
                config.clone(),
                Some(normalize_path(&path)),
            );
            (config, catalog)
        }
    };
    let allowed = runtime_edges();
    // Build only import-edge producers; rsc-callers traverses runtime imports
    // exclusively, so building route/queue/React/Swift/Terraform edges is waste.
    let plan = GraphBuildPlan::from_allowed(Some(&allowed));
    let mut fact_context = crate::codebase::ts_source::facts::TsFactContext::new(&root);
    fact_context.set_visible_file_set(graph_files.visible_path_set());
    let facts =
        crate::codebase::ts_source::facts::collect_ts_facts_with_context_sources_and_session(
            &session,
            graph_files.indexable(),
            crate::codebase::ts_source::facts::TsFactPlan {
                imports: true,
                function_calls: true,
                rsc_environment: true,
                ..Default::default()
            },
            &fact_context,
            &sources,
        );
    crate::invocation::check_timeout()?;
    let config = dataset.config(config_path)?;
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(&root, config_path, &config);
    let prepared_graph = crate::codebase::dependencies::graph::prepare_graph_config(
        &root,
        plan,
        &codebase_config,
        &config,
        &visible_paths,
    )?;
    let interner = session.interner_arc();
    let graph = DepGraph::build_with_plan_files_prepared_config_facts_resolution_cache_and_session(
        PreparedGraphBuild {
            root: &root,
            tsconfig: &tsconfig,
            tsconfig_catalog: Some(&tsconfig_catalog),
            plan,
            graph_files: &graph_files,
            config_path,
            prepared: &prepared_graph,
            facts: Some(&facts),
            import_resolution_cache: None,
            dotnet_facts: None,
            swift_facts: None,
            visible_paths: Some(&visible_paths),
        },
        session,
    )?;

    run_with_prepared(&root, component, depth, &graph, &facts, &interner)
}
