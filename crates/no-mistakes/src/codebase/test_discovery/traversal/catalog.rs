use super::*;
use crate::codebase::dependencies::graph;

pub(crate) fn prepare_tsconfig_catalog_with_framework_projects(
    input: FrameworkCatalogPreparation<'_>,
) -> Result<FrameworkCatalog> {
    crate::ast::with_request_parse_cache(|| {
        prepare_tsconfig_catalog_with_framework_projects_and_cache(input)
    })
}

fn prepare_tsconfig_catalog_with_framework_projects_and_cache(
    input: FrameworkCatalogPreparation<'_>,
) -> Result<FrameworkCatalog> {
    let FrameworkCatalogPreparation {
        root,
        tsconfig_path,
        tsconfig,
        config,
        codebase_config,
        workspace,
        root_visible_paths,
        visible_paths,
        sources,
        build_plan,
        graph_files,
        collect_graph_facts,
        framework_plan,
    } = input;
    let mut candidate_roots = Vec::with_capacity(workspace.packages.len() + 1);
    candidate_roots.push(root.to_path_buf());
    candidate_roots.extend(workspace.packages.iter().map(|package| package.dir.clone()));
    candidate_roots.extend(
        crate::integration_tests::configured_runner_config_dirs_from_visible(
            root,
            config,
            root_visible_paths,
        ),
    );
    let preliminary_roots = candidate_roots.clone();
    let preliminary_catalog = std::sync::Arc::new(catalog_for_request(
        root,
        tsconfig_path,
        tsconfig,
        &candidate_roots,
        root_visible_paths,
        &sources,
    ));
    let preliminary_graph = graph::prepare_graph_config_with_test_filter_and_workspace(
        root,
        build_plan,
        codebase_config,
        config,
        visible_paths,
        TestFileFilter::fallback_only(),
        std::sync::Arc::clone(workspace),
    );
    let preliminary_graph = preliminary_graph?;
    let (fact_plan, fact_context) = graph::ts_fact_plan_and_context_for_plan_with_prepared(
        root,
        build_plan,
        &preliminary_graph,
    );
    let graph_fact_files = if collect_graph_facts {
        graph_files.indexable()
    } else {
        &[]
    };
    let discovery_files = crate::codebase::ts_source::filter_discovered_files_by_skip_directories(
        root,
        &config.filesystem.skip_directories,
        graph_files.all(),
    );
    let mut projects = super::super::prepare_test_projects_from_visible_with_sources_and_plan(
        root,
        config,
        root_visible_paths,
        std::sync::Arc::clone(&preliminary_catalog),
        super::super::PreparedTestProjectRequest {
            discovery_files: &discovery_files,
            graph: (graph_fact_files, fact_plan, fact_context),
            sources: std::sync::Arc::clone(&sources),
            collect_graph_facts,
            playwright: if crate::codebase::playwright_facts::integration::configured(config) {
                preliminary_graph.playwright_fact_plan(root, tsconfig, visible_paths)?
            } else {
                None
            },
            preparation_plan: framework_plan,
        },
    );
    candidate_roots.extend(projects.tsconfig_candidate_roots(root));
    let catalog = super::super::reuse_or_rebuild_prepared_catalog(
        preliminary_catalog,
        &preliminary_roots,
        &mut candidate_roots,
        |candidate_roots| {
            std::sync::Arc::new(catalog_for_request(
                root,
                tsconfig_path,
                tsconfig,
                candidate_roots,
                root_visible_paths,
                &sources,
            ))
        },
    );
    projects.reparse_vitest_with_final_catalog(
        root,
        config,
        root_visible_paths,
        std::sync::Arc::clone(&catalog),
        std::sync::Arc::clone(&sources),
    );
    projects.reresolve_vitest_setups(root, &catalog, root_visible_paths);
    Ok((catalog, Box::new(projects)))
}

fn catalog_for_request(
    root: &Path,
    tsconfig_path: Option<&Path>,
    tsconfig: &TsConfig,
    candidate_roots: &[PathBuf],
    visible_paths: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> crate::codebase::ts_resolver::TsConfigCatalog {
    if let Some(path) = tsconfig_path {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        crate::codebase::ts_resolver::TsConfigCatalog::forced(
            root,
            tsconfig.clone(),
            Some(crate::codebase::ts_resolver::normalize_path(&path)),
        )
    } else {
        crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
            root,
            candidate_roots,
            visible_paths,
            sources,
        )
    }
}
