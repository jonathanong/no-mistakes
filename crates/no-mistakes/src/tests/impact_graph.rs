use anyhow::Result;
use no_mistakes::codebase::dependencies::graph::{
    DepGraph, GraphBuildPlan, GraphFiles, PreparedGraphBuild,
};
use no_mistakes::codebase::test_discovery::{
    DiscoveredTests, FrameworkPreparationPlan, PreparedTestProjectRequest, TestRunner,
};
use no_mistakes::codebase::test_filter::TestFileFilter;
use no_mistakes::codebase::ts_resolver::{TsConfig, TsConfigCatalog};
use no_mistakes::codebase::ts_source::{SourceStore, VisiblePathSnapshot};
use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) struct ImpactGraph {
    pub(crate) graph: DepGraph,
    pub(crate) test_filter: TestFileFilter,
    pub(crate) vitest_projects: Vec<no_mistakes::integration_tests::types::ConfigProject>,
    pub(crate) vitest_discovered: DiscoveredTests,
    pub(crate) visible_files: HashSet<PathBuf>,
}

/// Build the canonical graph shared by the CLI and N-API impact entrypoints.
/// Discovery, source loading, runner parsing, fact collection, and graph
/// construction all consume this request's single visible snapshot.
pub(crate) fn build_test_impact_graph(
    root: &Path,
    tsconfig_path: Option<&Path>,
    config: &NoMistakesConfig,
    config_path: Option<&Path>,
    include_symbols: bool,
) -> Result<ImpactGraph> {
    let visible = VisiblePathSnapshot::new(root);
    let visible_paths = visible.paths_for(root);
    let sources = visible.source_store_for(root);
    let tsconfig = no_mistakes::codebase::ts_resolver::resolve_tsconfig_from_visible(
        tsconfig_path,
        root,
        &visible_paths,
    )
    .or_else(|error| {
        if tsconfig_path.is_some() {
            Err(error)
        } else {
            Ok(TsConfig {
                dir: root.to_path_buf(),
                paths: Vec::new(),
                paths_dir: root.to_path_buf(),
                base_url: None,
            })
        }
    })?;
    let mut catalog_roots = vec![root.to_path_buf()];
    catalog_roots
        .extend(no_mistakes::integration_tests::configured_runner_config_dirs(root, config));
    let preliminary_catalog = catalog(
        root,
        &tsconfig,
        tsconfig_path,
        &catalog_roots,
        &visible_paths,
        &sources,
    );
    // TestOf classification is the union of every configured suite. Prepare
    // all runners in this same pass so native project filters remain
    // authoritative while Vitest setup dependencies are attached below.
    let framework_plan = FrameworkPreparationPlan::all();
    let excluded_configs = framework_plan.excluded_config_paths(root, config, &visible_paths);
    let graph_files = GraphFiles::from_files_with_resource_candidates_excluding_indexable(
        no_mistakes::codebase::ts_source::discover_files_from_visible(root, &[], &visible_paths),
        visible.tracked_paths_for(root).as_ref().clone(),
        &excluded_configs,
    );
    let graph_plan = GraphBuildPlan::test_impact().with_symbols(include_symbols);
    let codebase_config =
        no_mistakes::codebase::config::config_from_loaded_v2(root, config_path, config);
    let preliminary_graph_config =
        no_mistakes::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
            root,
            graph_plan,
            &codebase_config,
            config,
            &visible,
            no_mistakes::codebase::test_filter::TestFileFilter::fallback_only(),
        )?;
    let (runner_fact_plan, runner_fact_context) =
        no_mistakes::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
            root,
            graph_plan,
            &preliminary_graph_config,
        );
    let mut projects =
        no_mistakes::codebase::test_discovery::prepare_test_projects_from_visible_with_sources_and_plan(
            root,
            config,
            &visible_paths,
            preliminary_catalog,
            PreparedTestProjectRequest {
                graph: (graph_files.indexable(), runner_fact_plan, runner_fact_context),
                sources: Arc::clone(&sources),
                collect_graph_facts: true,
                preparation_plan: &framework_plan,
            },
        );
    catalog_roots.extend(projects.tsconfig_candidate_roots(root));
    catalog_roots.sort();
    catalog_roots.dedup();
    let catalog = catalog(
        root,
        &tsconfig,
        tsconfig_path,
        &catalog_roots,
        &visible_paths,
        &sources,
    );
    projects.reresolve_vitest_setups(root, &catalog, &visible_paths);
    let test_filter = TestFileFilter::for_impact_from_prepared_projects(
        root,
        config,
        &visible_paths,
        projects.project_filters(),
    );
    let vitest_projects = projects
        .prepared_projects(TestRunner::Vitest)
        .unwrap_or_default()
        .to_vec();
    let vitest_discovered =
        no_mistakes::codebase::test_discovery::discover_tests_from_prepared_projects(
            root,
            config,
            TestRunner::Vitest,
            &projects,
            &visible_paths,
            &tsconfig,
        )?;
    let prepared_graph_config =
        no_mistakes::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
            root,
            graph_plan,
            &codebase_config,
            config,
            &visible,
            test_filter.clone(),
        )?;
    let (fact_plan, fact_context) =
        no_mistakes::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
            root,
            graph_plan,
            &prepared_graph_config,
        );
    let mut facts = projects.graph_facts().clone();
    let remaining = graph_files
        .indexable()
        .iter()
        .filter(|path| !facts.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    facts.extend(
        no_mistakes::codebase::ts_source::facts::collect_ts_facts_with_context_and_sources(
            &remaining,
            fact_plan,
            &fact_context,
            &sources,
        ),
    );
    let graph =
        DepGraph::build_with_plan_files_prepared_config_and_all_facts(PreparedGraphBuild {
            root,
            tsconfig: &tsconfig,
            tsconfig_catalog: Some(&catalog),
            plan: graph_plan,
            graph_files: &graph_files,
            config_path,
            prepared: &prepared_graph_config,
            facts: Some(&facts),
            import_resolution_cache: None,
            dotnet_facts: projects.dotnet_facts(),
            swift_facts: projects.swift_facts(),
            visible_paths: Some(&visible),
        })?;
    Ok(ImpactGraph {
        graph: graph.with_vitest_setup_projects(projects.vitest_setup_projects()),
        test_filter,
        vitest_projects,
        vitest_discovered,
        visible_files: visible_paths.iter().cloned().collect(),
    })
}

fn catalog(
    root: &Path,
    tsconfig: &TsConfig,
    tsconfig_path: Option<&Path>,
    roots: &[PathBuf],
    visible: &[PathBuf],
    sources: &Arc<SourceStore>,
) -> Arc<TsConfigCatalog> {
    Arc::new(if let Some(path) = tsconfig_path {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        TsConfigCatalog::forced(
            root,
            tsconfig.clone(),
            Some(no_mistakes::codebase::ts_resolver::normalize_path(&path)),
        )
    } else {
        TsConfigCatalog::from_visible_and_sources(root, roots, visible, sources)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        no_mistakes::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test-cases/codebase-analysis")
                .join(name)
                .join("fixture"),
        )
    }

    fn filter(name: &str) -> TestFileFilter {
        let root = fixture(name);
        let visible = VisiblePathSnapshot::new(&root);
        let config = no_mistakes::config::v2::load_v2_config_from_visible(
            &root,
            None,
            &visible.paths_for(&root),
        )
        .unwrap();
        build_test_impact_graph(&root, None, &config, None, false)
            .unwrap()
            .test_filter
    }

    #[test]
    fn impact_filter_preserves_configured_dotnet_test_projects() {
        let root = fixture("dotnet-test-plan");
        let filter = filter("dotnet-test-plan");
        assert!(filter.is_match(
            &root,
            &root.join("dotnet-clients/tests/App.Tests/FeedServiceTests.cs")
        ));
        assert!(!filter.is_match(&root, &root.join("dotnet-clients/src/App/FeedService.cs")));
    }

    #[test]
    fn impact_filter_preserves_configured_swift_test_projects() {
        let root = fixture("swift-test-plan");
        let filter = filter("swift-test-plan");
        assert!(filter.is_match(
            &root,
            &root.join("swift-clients/core/Tests/VouchaCoreTests/APIClientTests.swift")
        ));
        assert!(!filter.is_match(
            &root,
            &root.join("swift-clients/core/Sources/VouchaCore/APIClient.swift")
        ));
    }
}
