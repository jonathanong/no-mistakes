use super::changed_files::{collect_changed_files, existing_changed_files, ChangedFiles};
use super::lockfile_changes::{analyze_lockfile_changes, LockfileAnalysis};
use super::{PlanArgs, TestFramework};
use anyhow::{Context, Result};
use no_mistakes::codebase::dependencies::graph::{
    DepGraph, GraphBuildPlan, GraphFiles, PreparedGraphConfig,
};
use no_mistakes::codebase::test_discovery::{DiscoveredTests, TestRunner};
use no_mistakes::codebase::ts_resolver::TsConfig;
use no_mistakes::codebase::ts_source::VisiblePathSnapshot;
use no_mistakes::codebase::workspaces::WorkspaceMap;
use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

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
    root_visible_paths: Arc<Vec<PathBuf>>,
    pub(crate) config: NoMistakesConfig,
    config_path: Option<PathBuf>,
    pub(crate) tsconfig: TsConfig,
    pub(crate) collected: ChangedFiles,
    pub(crate) changed_files: Vec<PathBuf>,
    pub(crate) lockfile_analysis: LockfileAnalysis,
    pub(crate) lockfile_changed_packages: Vec<(String, String)>,
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
        )?;
        let changed_files = existing_changed_files(&collected);
        let lockfile_analysis = analyze_lockfile_changes(&args, &root, &collected.files);
        let lockfile_changed_packages = lockfile_packages(&root, &lockfile_analysis);
        let workspace_map =
            no_mistakes::codebase::workspaces::load_from_files(&root, &root_visible_paths)
                .unwrap_or_default();
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
        let mut graph_files = GraphFiles::from_files_excluding_indexable(
            no_mistakes::codebase::ts_source::discover_files_from_visible(
                &root,
                &[],
                &root_visible_paths,
            ),
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
        let prepared_test_projects = Arc::new(
            no_mistakes::codebase::test_discovery::prepare_test_projects_from_visible_with_sources_and_plan(
                &root,
                &config,
                &root_visible_paths,
                &tsconfig,
                no_mistakes::codebase::test_discovery::PreparedTestProjectRequest {
                    graph: (graph_files.indexable(), runner_graph_plan, runner_graph_context),
                    sources: visible_paths.source_store_for(&root),
                    collect_graph_facts: true,
                    preparation_plan: &framework_plan,
                },
            ),
        );
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
            root_visible_paths,
            config,
            config_path,
            tsconfig,
            collected,
            changed_files,
            lockfile_analysis,
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

    pub(crate) fn root_visible_paths(&self) -> &[PathBuf] {
        &self.root_visible_paths
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
                        self.graph_files.visible().iter().cloned().collect(),
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
                        .filter(|path| !facts.contains_key(*path))
                        .cloned()
                        .collect::<Vec<_>>();
                    facts.extend(no_mistakes::codebase::ts_source::facts::collect_ts_facts_with_context_and_sources(
                        &remaining,
                        fact_plan,
                        &fact_context,
                        &self.visible_paths.source_store_for(&self.root),
                    ));
                    Box::new(facts)
                };
            DepGraph::build_with_plan_files_prepared_config_and_all_facts(
                &self.root,
                &self.tsconfig,
                self.graph_plan,
                &self.graph_files,
                self.args.config.as_deref(),
                &self.prepared_graph_config,
                no_mistakes::codebase::dependencies::graph::PreparedGraphFacts {
                    ts: Some(facts.as_ref()),
                    dotnet: self.prepared_test_projects.dotnet_facts(),
                    swift: self.prepared_test_projects.swift_facts(),
                },
            )
            .map_err(|error| format!("{error:#}"))
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

    pub(crate) fn framework_discovery_count(&self) -> usize {
        self.framework_discoveries.load(Ordering::Relaxed)
    }

    pub(crate) fn test_filter(&self) -> &no_mistakes::codebase::test_filter::TestFileFilter {
        &self.test_filter
    }

    pub(crate) fn discover_tests(&self, framework: TestFramework) -> Result<DiscoveredTests> {
        let mut cache = self
            .discovered_tests
            .lock()
            .expect("prepared test-discovery cache mutex poisoned");
        cache
            .entry(framework)
            .or_insert_with(|| {
                self.framework_discoveries.fetch_add(1, Ordering::Relaxed);
                no_mistakes::codebase::test_discovery::discover_tests_from_prepared_projects(
                    &self.root,
                    &self.config,
                    test_runner(framework),
                    &self.prepared_test_projects,
                    &self.root_visible_paths,
                    &self.tsconfig,
                )
                .map(|mut discovered| {
                    // Automatic test discovery follows regular files, matching the pre-snapshot
                    // behavior without a live `is_file` pass. Explicit changed paths remain
                    // authoritative and are handled separately by the planner.
                    discovered.tests.retain(|path| {
                        self.visible_paths
                            .classification_for(&self.root, path)
                            .is_some_and(|classification| classification.target_is_file())
                    });
                    discovered
                        .targets_by_path
                        .retain(|path, _| discovered.tests.binary_search(path).is_ok());
                    discovered
                })
                .map_err(|error| format!("{error:#}"))
            })
            .clone()
            .map_err(anyhow::Error::msg)
    }

    pub(crate) fn discover_runner_tests(&self, runner: TestRunner) -> Result<DiscoveredTests> {
        self.discover_tests(test_framework(runner))
    }

    /// Returns a config fallback only when the changed effective config
    /// changes this framework. An unreadable historical endpoint deliberately
    /// remains conservative and falls back for the changed config.
    pub(crate) fn framework_config_trigger(
        &self,
        framework: TestFramework,
    ) -> Option<(String, PathBuf)> {
        let trigger_file = super::config_invalidation::changed_config_path(
            &self.args,
            &self.root,
            &self.collected,
        )?;
        let comparison = self.config_invalidation.get_or_init(|| {
            super::config_invalidation::compare_changed_config(
                &self.args,
                &self.root,
                &self.collected,
            )
            .map_err(|error| format!("{error:#}"))
        });
        match comparison {
            Ok(Some(invalidation)) if invalidation.framework_changed(framework) => {
                Some(invalidation.trigger())
            }
            Ok(Some(_)) | Ok(None) => None,
            // The caller explicitly opted into global configuration fallback;
            // without two complete valid endpoints we cannot safely suppress it.
            Err(_) => Some((
                format!(
                    "Global configuration file changed: {}",
                    super::plan::relative_path(&self.root, &trigger_file)
                ),
                trigger_file,
            )),
        }
    }

    pub(crate) fn requested_runner_projects(
        &self,
        runner: TestRunner,
    ) -> Result<Vec<no_mistakes::codebase::test_discovery::PreparedRunnerProject>> {
        self.prepared_test_projects
            .requested_runner_projects(runner)
    }
}

fn resolve_args(args: &PlanArgs) -> Result<PlanArgs> {
    if args.from_git_diff.is_some() && (args.base.is_some() || args.head.is_some()) {
        anyhow::bail!("--from-git-diff conflicts with --base/--head; provide only one");
    }
    let mut args = args.clone();
    if let Some(spec) = args.from_git_diff.take() {
        let (base, head) = super::changed_files::parse_git_diff_refspec(&spec)?;
        args.base = Some(base);
        args.head = Some(head.unwrap_or_else(|| "HEAD".to_string()));
    }
    Ok(args)
}

fn lockfile_packages(root: &Path, analysis: &LockfileAnalysis) -> Vec<(String, String)> {
    analysis
        .diff_by_lockfile
        .iter()
        .flat_map(|(lockfile_path, diff)| {
            let relative = super::plan::relative_path(root, lockfile_path);
            diff.all_changed_names()
                .map(|name| (name.to_string(), relative.clone()))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn test_runner(framework: TestFramework) -> TestRunner {
    match framework {
        TestFramework::Dotnet => TestRunner::Dotnet,
        TestFramework::Playwright => TestRunner::Playwright,
        TestFramework::Vitest => TestRunner::Vitest,
        TestFramework::Swift => TestRunner::Swift,
    }
}

fn framework_graph_plan(framework: TestFramework) -> GraphBuildPlan {
    let mut plan = GraphBuildPlan::test_impact();
    plan.dotnet = framework == TestFramework::Dotnet;
    plan.swift = framework == TestFramework::Swift;
    let playwright = framework == TestFramework::Playwright;
    plan.playwright_routes = playwright;
    plan.playwright_selectors = playwright;
    plan
}

fn test_framework(runner: TestRunner) -> TestFramework {
    match runner {
        TestRunner::Dotnet => TestFramework::Dotnet,
        TestRunner::Playwright => TestFramework::Playwright,
        TestRunner::Vitest => TestFramework::Vitest,
        TestRunner::Swift => TestFramework::Swift,
    }
}

#[cfg(test)]
#[path = "prepared_plan/tests.rs"]
mod tests;
