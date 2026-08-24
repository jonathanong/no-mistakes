use super::changed_files::{collect_changed_files, existing_changed_files, ChangedFiles};
use super::dotnet_dependency_changes::{
    analyze_dotnet_dependency_changes, DotnetDependencyAnalysis,
};
use super::lockfile_changes::{analyze_lockfile_changes, LockfileAnalysis};
use super::package_manifest_changes::{analyze_package_manifest_changes, PackageManifestAnalysis};
use super::swift_manifest_changes::{analyze_swift_manifest_changes, SwiftManifestAnalysis};
use super::swift_resolved_changes::{analyze_swift_resolved_changes, SwiftResolvedAnalysis};
use super::{PlanArgs, TestFramework};
use anyhow::{Context, Result};
use no_mistakes::codebase::dependencies::graph::{
    DepGraph, GraphBuildPlan, GraphFiles, PreparedGraphBuild, PreparedGraphConfig,
};
use no_mistakes::codebase::test_discovery::{DiscoveredTests, TestRunner};
use no_mistakes::codebase::ts_resolver::TsConfig;
use no_mistakes::codebase::ts_source::{SourceStore, VisiblePathSnapshot};
use no_mistakes::codebase::workspaces::WorkspaceMap;
use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

mod helpers;
use helpers::{framework_graph_plan, lockfile_packages, resolve_args, test_framework, test_runner};
pub(crate) mod revisions;
use revisions::RevisionSources;

/// Immutable, request-scoped inputs shared by direct test plans and the
/// multi-framework impacted-check fanout.
pub(crate) struct PreparedTestPlanInputs {
    args: PlanArgs,
    pub(crate) root: PathBuf,
    pub(crate) visible_paths: Arc<VisiblePathSnapshot>,
    root_visible_paths: Arc<Vec<PathBuf>>,
    pub(crate) config: NoMistakesConfig,
    config_path: Option<PathBuf>,
    pub(crate) collected: ChangedFiles,
}

pub(crate) struct PreparedTestPlanRequest {
    args: PlanArgs,
    pub(crate) root: PathBuf,
    pub(crate) visible_paths: Arc<VisiblePathSnapshot>,
    sources: Arc<SourceStore>,
    root_visible_paths: Arc<Vec<PathBuf>>,
    pub(crate) config: NoMistakesConfig,
    config_path: Option<PathBuf>,
    pub(crate) tsconfig: TsConfig,
    tsconfig_catalog: Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    pub(crate) collected: ChangedFiles,
    pub(crate) changed_files: Vec<PathBuf>,
    pub(crate) package_manifest_analysis: PackageManifestAnalysis,
    pub(crate) lockfile_analysis: LockfileAnalysis,
    pub(crate) swift_resolved_analysis: SwiftResolvedAnalysis,
    pub(crate) swift_manifest_analysis: SwiftManifestAnalysis,
    pub(crate) dotnet_dependency_analysis: DotnetDependencyAnalysis,
    pub(crate) lockfile_changed_packages: Vec<(String, String, Vec<PathBuf>)>,
    pub(crate) workspace_map: WorkspaceMap,
    graph_files: GraphFiles,
    graph_plan: GraphBuildPlan,
    prepared_graph_config: PreparedGraphConfig,
    prepared_test_projects: Arc<no_mistakes::codebase::test_discovery::PreparedTestProjects>,
    test_filter: no_mistakes::codebase::test_filter::TestFileFilter,
    graph: OnceLock<std::result::Result<DepGraph, String>>,
    discovered_tests: Mutex<HashMap<TestFramework, std::result::Result<DiscoveredTests, String>>>,
    config_invalidation: OnceLock<
        std::result::Result<Option<super::config_invalidation::ConfigInvalidation>, String>,
    >,
    graph_builds: AtomicUsize,
    framework_discoveries: AtomicUsize,
}

impl PreparedTestPlanInputs {
    pub(crate) fn prepare(args: &PlanArgs) -> Result<Self> {
        let args = resolve_args(args)?;
        let cwd = std::env::current_dir().context("cwd must be accessible")?;
        let root = no_mistakes::cli::resolve_optional_root(Some(&args.root), &cwd);
        let root = no_mistakes::codebase::ts_resolver::normalize_path(&root);
        let root = root.canonicalize().unwrap_or(root);

        let visible_paths = Arc::new(VisiblePathSnapshot::new(&root));
        let root_visible_paths = visible_paths.paths_for(&root);
        let config_path = no_mistakes::config::v2::effective_v2_config_path_from_visible(
            &root,
            args.config.as_deref(),
            &root_visible_paths,
        )?;
        let config = no_mistakes::config::v2::load_v2_config_from_visible(
            &root,
            args.config.as_deref(),
            &root_visible_paths,
        )?;
        let collected = collect_changed_files(&args, &root)?;

        Ok(Self {
            args,
            root,
            visible_paths,
            root_visible_paths,
            config,
            config_path,
            collected,
        })
    }

    pub(crate) fn root_visible_paths(&self) -> &[PathBuf] {
        &self.root_visible_paths
    }

    pub(crate) fn finish(self) -> Result<PreparedTestPlanRequest> {
        let Self {
            args,
            root,
            visible_paths,
            root_visible_paths,
            config,
            config_path,
            collected,
        } = self;
        let tsconfig = no_mistakes::codebase::ts_resolver::resolve_tsconfig_from_visible(
            args.tsconfig.as_deref(),
            &root,
            &root_visible_paths,
        )
        .or_else(|error| {
            if args.tsconfig.is_some() {
                Err(error)
            } else {
                Ok(TsConfig {
                    dir: root.clone(),
                    paths: Vec::new(),
                    paths_dir: root.clone(),
                    base_url: None,
                })
            }
        })?;
        let sources = visible_paths.source_store_for(&root);
        let revisions = RevisionSources::prepare(&root, &args, Arc::clone(&sources));
        let changed_files = existing_changed_files(&collected);
        let workspace_map =
            no_mistakes::codebase::workspaces::load_from_source_store(&root, &sources)
                .unwrap_or_default();
        let package_manifest_analysis = analyze_package_manifest_changes(
            &root,
            &collected.files,
            &revisions,
            &workspace_map,
            &sources,
        );
        let lockfile_analysis = analyze_lockfile_changes(&root, &collected.files, &revisions);
        let swift_resolved_analysis =
            analyze_swift_resolved_changes(&root, &config, &collected.files, &revisions);
        let swift_manifest_analysis =
            analyze_swift_manifest_changes(&root, &config, &collected.files, &revisions);
        let dotnet_dependency_analysis =
            analyze_dotnet_dependency_changes(&root, &collected.files, &revisions);
        let mut lockfile_changed_packages = lockfile_packages(&root, &lockfile_analysis);
        lockfile_changed_packages.extend(package_manifest_analysis.changed_packages.iter().map(
            |(package, changed_file, manifest)| {
                (
                    package.clone(),
                    changed_file.clone(),
                    vec![manifest.clone()],
                )
            },
        ));
        let mut tsconfig_candidate_roots = Vec::with_capacity(workspace_map.packages.len() + 1);
        tsconfig_candidate_roots.push(root.clone());
        tsconfig_candidate_roots.extend(
            workspace_map
                .packages
                .iter()
                .map(|package| package.dir.clone()),
        );
        tsconfig_candidate_roots
            .extend(no_mistakes::integration_tests::configured_runner_config_dirs(&root, &config));
        let preliminary_tsconfig_roots = tsconfig_candidate_roots.clone();
        // Test-runner configs are parsed before their project scopes can
        // contribute more roots. Explicit runner config parents are candidate
        // roots too, so aliases resolve from a configured package even when
        // the repository does not declare it as a workspace.
        let preliminary_tsconfig_catalog = Arc::new(if let Some(path) = args.tsconfig.as_deref() {
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            no_mistakes::codebase::ts_resolver::TsConfigCatalog::forced(
                &root,
                tsconfig.clone(),
                Some(no_mistakes::codebase::ts_resolver::normalize_path(&path)),
            )
        } else {
            no_mistakes::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
                &root,
                &tsconfig_candidate_roots,
                &root_visible_paths,
                &sources,
            )
        });
        let framework_plan = args.framework.map_or_else(
            no_mistakes::codebase::test_discovery::FrameworkPreparationPlan::all,
            |framework| {
                no_mistakes::codebase::test_discovery::FrameworkPreparationPlan::for_runners([
                    test_runner(framework),
                ])
            },
        );
        let excluded_configs =
            framework_plan.excluded_config_paths(&root, &config, &root_visible_paths);
        let graph_all_files = no_mistakes::codebase::ts_source::discover_files_from_visible(
            &root,
            &[],
            &root_visible_paths,
        );
        let discovery_files =
            no_mistakes::codebase::ts_source::filter_discovered_files_by_skip_directories(
                &root,
                &config.filesystem.skip_directories,
                &graph_all_files,
            );
        let mut resource_candidates = visible_paths.tracked_paths_for(&root).as_ref().clone();
        // The post-change snapshot cannot contain deleted tracked resources,
        // but reverse impact still needs their phantom nodes. Reuse the diff
        // inventory collected for this request; do not perform another walk.
        resource_candidates.extend(collected.deleted.iter().cloned());
        let mut graph_files = GraphFiles::from_files_with_resource_candidates_excluding_indexable(
            graph_all_files.clone(),
            // Preserve tracked runtime inputs under source-skipped directories
            // such as `fixtures/`; they are resource targets, not parse roots.
            resource_candidates,
            &excluded_configs,
        );
        for path in &collected.authoritative_files {
            graph_files.add_explicit_root(path);
        }
        // Framework plans historically ignore --symbols. The non-framework
        // planner is the only stable surface that opts into symbol edges.
        let graph_plan = args
            .framework
            .map_or_else(GraphBuildPlan::all, framework_graph_plan)
            .with_symbols(args.framework.is_none() && args.include_symbols);
        let codebase_config = no_mistakes::codebase::config::config_from_loaded_v2(
            &root,
            args.config.as_deref(),
            &config,
        );
        let preliminary_graph_config =
            no_mistakes::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
                &root,
                graph_plan,
                &codebase_config,
                &config,
                &visible_paths,
                no_mistakes::codebase::test_filter::TestFileFilter::fallback_only(),
            )?;
        let (runner_graph_plan, runner_graph_context) =
            no_mistakes::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
                &root,
                graph_plan,
                &preliminary_graph_config,
            );
        let mut prepared_test_projects =
            no_mistakes::codebase::test_discovery::prepare_test_projects_from_visible_with_sources_and_plan(
                &root,
                &config,
                &root_visible_paths,
                Arc::clone(&preliminary_tsconfig_catalog),
                no_mistakes::codebase::test_discovery::PreparedTestProjectRequest {
                    discovery_files: &discovery_files,
                    graph: (graph_files.indexable(), runner_graph_plan, runner_graph_context),
                    sources: Arc::clone(&sources),
                    collect_graph_facts: true,
                    preparation_plan: &framework_plan,
                },
            );
        tsconfig_candidate_roots.extend(prepared_test_projects.tsconfig_candidate_roots(&root));
        let tsconfig_catalog =
            no_mistakes::codebase::test_discovery::reuse_or_rebuild_prepared_catalog(
                preliminary_tsconfig_catalog,
                &preliminary_tsconfig_roots,
                &mut tsconfig_candidate_roots,
                |candidate_roots| {
                    Arc::new(
                        no_mistakes::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
                            &root,
                            candidate_roots,
                            &root_visible_paths,
                            &sources,
                        ),
                    )
                },
            );
        prepared_test_projects.reparse_vitest_with_final_catalog(
            &root,
            &config,
            &root_visible_paths,
            Arc::clone(&tsconfig_catalog),
            Arc::clone(&sources),
        );
        prepared_test_projects.reresolve_vitest_setups(
            &root,
            &tsconfig_catalog,
            &root_visible_paths,
        );
        let prepared_test_projects = Arc::new(prepared_test_projects);
        let test_filter =
            no_mistakes::codebase::test_filter::TestFileFilter::from_prepared_projects(
                &root,
                &config,
                &root_visible_paths,
                prepared_test_projects.project_filters(),
            );
        let prepared_graph_config =
            no_mistakes::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
                &root,
                graph_plan,
                &codebase_config,
                &config,
                &visible_paths,
                test_filter.clone(),
            )?;

        Ok(PreparedTestPlanRequest {
            args,
            root,
            visible_paths,
            sources,
            root_visible_paths,
            config,
            config_path,
            tsconfig,
            tsconfig_catalog,
            collected,
            changed_files,
            package_manifest_analysis,
            lockfile_analysis,
            swift_resolved_analysis,
            swift_manifest_analysis,
            dotnet_dependency_analysis,
            lockfile_changed_packages,
            workspace_map,
            graph_files,
            graph_plan,
            prepared_graph_config,
            prepared_test_projects,
            test_filter,
            graph: OnceLock::new(),
            discovered_tests: Mutex::new(HashMap::new()),
            config_invalidation: OnceLock::new(),
            graph_builds: AtomicUsize::new(0),
            framework_discoveries: AtomicUsize::new(0),
        })
    }
}

impl PreparedTestPlanRequest {
    pub(crate) fn prepare(args: &PlanArgs) -> Result<Self> {
        PreparedTestPlanInputs::prepare(args)?.finish()
    }

    pub(crate) fn args(&self) -> &PlanArgs {
        &self.args
    }

    pub(crate) fn dotnet_facts(&self) -> Option<&no_mistakes::codebase::dotnet::DotnetFactMap> {
        self.prepared_test_projects.dotnet_facts()
    }

    pub(crate) fn root_visible_paths(&self) -> &[PathBuf] {
        &self.root_visible_paths
    }

    pub(crate) fn changed_file_inventory(&self) -> Vec<String> {
        let mut files: Vec<String> = self
            .collected
            .inventory_files
            .iter()
            .map(|path| no_mistakes::codebase::ts_source::relative_slash_path(&self.root, path))
            .collect();
        files.sort();
        files.dedup();
        files
    }

    pub(crate) fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    pub(crate) fn graph(&self) -> Result<&DepGraph> {
        self.graph
            .get_or_init(|| {
            self.graph_builds.fetch_add(1, Ordering::Relaxed);
            let (fact_plan, fact_context) =
                no_mistakes::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
                    &self.root,
                    self.graph_plan,
                    &self.prepared_graph_config,
                );
            let graph_visible_paths = VisiblePathSnapshot::from_paths(
                &self.root,
                self.graph_files.all(),
            );
            let playwright = self
                .prepared_graph_config
                .playwright_fact_plan(&self.root, &self.tsconfig, &graph_visible_paths)
                .map_err(|error| format!("{error:#}"))?;
            let facts: Box<dyn no_mistakes::codebase::dependencies::graph::TsFactLookup> =
                if let Some(playwright) = playwright {
                    Box::new(no_mistakes::codebase::check_facts::collect_check_facts_with_precollected_graph_facts(
                        &self.root,
                        self.graph_files.iter_visible().cloned().collect(),
                        no_mistakes::codebase::check_facts::CheckFactPlan {
                            graph: fact_plan,
                            graph_context: fact_context,
                            ..Default::default()
                        },
                        playwright,
                        self.prepared_test_projects.graph_facts().clone(),
                    ))
                } else {
                    let mut facts = self.prepared_test_projects.graph_facts().clone();
                    let remaining = self
                        .graph_files
                        .indexable()
                        .iter()
                        .filter(|path| !facts.contains_key(path))
                        .cloned()
                        .collect::<Vec<_>>();
                    facts.extend(no_mistakes::codebase::ts_source::facts::collect_ts_facts_with_context_and_sources(
                        &remaining,
                        fact_plan,
                        &fact_context,
                        &self.sources,
                    ));
                    Box::new(facts)
                };
            let graph = DepGraph::build_with_plan_files_prepared_config_and_all_facts(
                PreparedGraphBuild {
                    root: &self.root,
                    tsconfig: &self.tsconfig,
                    tsconfig_catalog: Some(&self.tsconfig_catalog),
                    plan: self.graph_plan,
                    graph_files: &self.graph_files,
                    config_path: self.args.config.as_deref(),
                    prepared: &self.prepared_graph_config,
                    facts: Some(facts.as_ref()),
                    import_resolution_cache: None,
                    dotnet_facts: self.prepared_test_projects.dotnet_facts(),
                    swift_facts: self.prepared_test_projects.swift_facts(),
                    visible_paths: None,
                },
            )
            .map_err(|error| format!("{error:#}"))?;
            let graph = graph.with_vitest_setup_projects(
                self.prepared_test_projects.vitest_setup_projects(),
            );
            Ok(graph)
        })
            .as_ref()
            .map_err(|error| anyhow::Error::msg(error.clone()))
    }

    pub(crate) fn graph_is_initialized(&self) -> bool {
        self.graph.get().is_some()
    }

    pub(crate) fn graph_build_count(&self) -> usize {
        self.graph_builds.load(Ordering::Relaxed)
    }

    /// Parsed Vitest project metadata from the request's single runner-config
    /// pass. Callers use this for conservative setup diagnostics and must not
    /// reparse configuration when Vitest was not requested.
    pub(crate) fn vitest_projects(
        &self,
    ) -> Option<&[no_mistakes::integration_tests::types::ConfigProject]> {
        self.prepared_test_projects
            .prepared_projects(TestRunner::Vitest)
    }
}

pub(crate) fn javascript_dependency_framework(framework: Option<TestFramework>) -> bool {
    matches!(
        framework,
        None | Some(TestFramework::Vitest)
            | Some(TestFramework::Playwright)
            | Some(TestFramework::Jest)
    )
}

#[path = "prepared_plan_discovery.rs"]
mod prepared_plan_discovery;

#[cfg(test)]
#[path = "prepared_plan/tests.rs"]
mod tests;
