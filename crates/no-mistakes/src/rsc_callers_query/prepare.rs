use super::*;
use crate::codebase::dependencies::graph::{GraphBuildPlan, GraphFiles, PreparedGraphBuild};
use crate::codebase::ts_resolver::{find_tsconfig_from_visible, load_tsconfig, TsConfig};

pub(super) fn resolve_tsconfig_from_visible(
    root: &Path,
    tsconfig: Option<&Path>,
    visible_paths: &[PathBuf],
) -> Result<TsConfig> {
    match tsconfig {
        // Resolve a relative explicit tsconfig against `root`, not the cwd.
        Some(path) if path.is_absolute() => load_tsconfig(path),
        Some(path) => load_tsconfig(&root.join(path)),
        None => match find_tsconfig_from_visible(root, visible_paths) {
            Some(path) => match load_tsconfig(&path) {
                Ok(config) => Ok(config),
                Err(_) => Ok(TsConfig {
                    dir: root.to_path_buf(),
                    paths: vec![],
                    paths_dir: root.to_path_buf(),
                    base_url: None,
                }),
            },
            None => Ok(TsConfig {
                dir: root.to_path_buf(),
                paths: vec![],
                paths_dir: root.to_path_buf(),
                base_url: None,
            }),
        },
    }
}

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
    let visible_paths = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let root_visible_paths = visible_paths.paths_for(&root);
    let mut graph_files = GraphFiles::from_files(
        crate::codebase::ts_source::discover_files_from_visible(&root, &[], &root_visible_paths),
    );
    graph_files.add_explicit_root(&component_abs);
    let explicit_tsconfig = tsconfig;
    let tsconfig = resolve_tsconfig_from_visible(&root, explicit_tsconfig, &root_visible_paths)?;
    let tsconfig_catalog = explicit_tsconfig.map_or_else(
        || {
            crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
                &root,
                std::slice::from_ref(&root),
                &root_visible_paths,
            )
        },
        |path| {
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            crate::codebase::ts_resolver::TsConfigCatalog::forced(
                &root,
                tsconfig.clone(),
                Some(normalize_path(&path)),
            )
        },
    );
    let allowed = runtime_edges();
    // Build only import-edge producers; rsc-callers traverses runtime imports
    // exclusively, so building route/queue/React/Swift/Terraform edges is waste.
    let plan = GraphBuildPlan::from_allowed(Some(&allowed));
    let mut fact_context = crate::codebase::ts_source::facts::TsFactContext::new(&root);
    fact_context.set_visible_files(graph_files.visible().iter().cloned());
    let facts = crate::codebase::ts_source::facts::collect_ts_facts_with_context(
        graph_files.indexable(),
        crate::codebase::ts_source::facts::TsFactPlan {
            imports: true,
            function_calls: true,
            rsc_environment: true,
            ..Default::default()
        },
        &fact_context,
    );
    crate::invocation::check_timeout()?;
    let config =
        crate::config::v2::load_v2_config_from_visible(&root, config_path, &root_visible_paths)?;
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(&root, config_path, &config);
    let prepared_graph = crate::codebase::dependencies::graph::prepare_graph_config(
        &root,
        plan,
        &codebase_config,
        &config,
        &visible_paths,
    )?;
    let graph = DepGraph::build_with_plan_files_prepared_config_facts_and_resolution_cache(
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
    )?;

    run_with_prepared(&root, component, depth, &graph, &facts)
}
