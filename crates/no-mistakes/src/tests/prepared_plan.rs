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
    pub(crate) collected: ChangedFiles,
}

pub(crate) struct PreparedTestPlanRequest {
    args: PlanArgs,
    pub(crate) root: PathBuf,
    pub(crate) visible_paths: Arc<VisiblePathSnapshot>,
    root_visible_paths: Arc<Vec<PathBuf>>,
    pub(crate) config: NoMistakesConfig,
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
        let mut graph_files = GraphFiles::from_files(
            no_mistakes::codebase::ts_source::discover_files_from_visible(
                &root,
                &[],
                &root_visible_paths,
            ),
        );
        for path in &collected.authoritative_files {
            graph_files.add_explicit_root(path);
        }
        // Framework plans historically ignore --symbols. The non-framework
        // planner is the only stable surface that opts into symbol edges.
        let graph_plan =
            GraphBuildPlan::all().with_symbols(args.framework.is_none() && args.include_symbols);
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
            no_mistakes::codebase::test_discovery::prepare_test_projects_from_visible(
                &root,
                &config,
                &root_visible_paths,
                &tsconfig,
                graph_files.indexable(),
                runner_graph_plan,
                runner_graph_context,
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
                .map_err(|error| format!("{error:#}"))?
                .ok_or_else(|| {
                    "prepared all-edge graph is missing its Playwright fact plan".to_string()
                })?;
            let facts = no_mistakes::codebase::check_facts::collect_check_facts_with_precollected_graph_facts(
                &self.root,
                self.graph_files.visible().iter().cloned().collect(),
                no_mistakes::codebase::check_facts::CheckFactPlan {
                    graph: fact_plan,
                    graph_context: fact_context,
                    ..Default::default()
                },
                playwright,
                self.prepared_test_projects.graph_facts().clone(),
            );
            DepGraph::build_with_plan_files_prepared_config_and_facts(
                &self.root,
                &self.tsconfig,
                self.graph_plan,
                &self.graph_files,
                self.args.config.as_deref(),
                &self.prepared_graph_config,
                Some(&facts),
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
                .map_err(|error| format!("{error:#}"))
            })
            .clone()
            .map_err(anyhow::Error::msg)
    }

    pub(crate) fn discover_runner_tests(&self, runner: TestRunner) -> Result<DiscoveredTests> {
        self.discover_tests(test_framework(runner))
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

fn test_framework(runner: TestRunner) -> TestFramework {
    match runner {
        TestRunner::Dotnet => TestFramework::Dotnet,
        TestRunner::Playwright => TestFramework::Playwright,
        TestRunner::Vitest => TestFramework::Vitest,
        TestRunner::Swift => TestFramework::Swift,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use no_mistakes::codebase::dependencies::graph::NodeId;

    #[test]
    fn non_framework_plan_reuses_prepared_runner_config_facts_and_filter() {
        let source = no_mistakes::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/parser-count/non-framework-prepared-plan"),
        );
        let fixture = crate::test_support::materialize_saved_fixture(&source);
        let root = fixture.path().canonicalize().unwrap();
        let args = PlanArgs {
            framework: None,
            root: root.clone(),
            config: None,
            tsconfig: None,
            base: None,
            head: None,
            from_git_diff: None,
            changed_file: vec![root.join("src/unit.ts")],
            changed_files: None,
            diff: None,
            diff_stdin: false,
            diff_command: None,
            entrypoints: Vec::new(),
            entrypoint_symbols: Vec::new(),
            include_symbols: false,
            diff_content: None,
            environment: "pre-push".to_string(),
            limit_percent: None,
            limit_files: None,
            global_config_fallback: None,
            format: None,
            json: false,
        };

        crate::ast::begin_parse_count(&root);
        let plan = crate::tests::plan::generate_plan(&args).unwrap();
        let counts = crate::ast::finish_parse_count(&root);

        assert!(plan
            .selected_tests
            .iter()
            .any(|test| test.test_file == "src/unit.test.ts"));
        assert_eq!(counts.len(), 6, "{counts:#?}");
        assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
    }

    #[test]
    fn complete_prepared_graph_keeps_standard_skipped_playwright_sources_outside_its_universe() {
        let source = no_mistakes::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/check-discovery/project-pattern-reopen/fixture"),
        );
        let fixture = crate::test_support::materialize_saved_fixture(&source);
        let root = fixture.path().canonicalize().unwrap();
        let changed = root.join("web/next.config.ts");
        let args = PlanArgs {
            framework: Some(TestFramework::Vitest),
            root: root.clone(),
            config: None,
            tsconfig: None,
            base: None,
            head: None,
            from_git_diff: None,
            changed_file: vec![changed.clone()],
            changed_files: None,
            diff: None,
            diff_stdin: false,
            diff_command: None,
            entrypoints: Vec::new(),
            entrypoint_symbols: Vec::new(),
            include_symbols: false,
            diff_content: None,
            environment: "pre-push".to_string(),
            limit_percent: None,
            limit_files: None,
            global_config_fallback: None,
            format: None,
            json: false,
        };

        let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();
        crate::ast::begin_parse_count(&root);
        let graph = prepared.graph().unwrap();
        let counts = crate::ast::finish_parse_count(&root);

        assert!(graph.dependencies_of_node(&NodeId::File(changed)).is_some());
        assert!(graph
            .dependencies_of_node(&NodeId::File(root.join("web/fixtures/included.ts")))
            .is_none());
        assert!(!counts.contains_key(&root.join("web/fixtures/included.ts")));
    }
}
