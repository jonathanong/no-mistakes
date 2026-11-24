use super::helpers::{lockfile_packages, resolve_args};
use super::revisions::RevisionSources;
use super::*;
use crate::tests::changed_files::{collect_changed_files, existing_changed_files};
use crate::tests::dotnet_dependency_changes::analyze_dotnet_dependency_changes;
use crate::tests::lockfile_changes::analyze_lockfile_changes;
use crate::tests::package_manifest_changes::analyze_package_manifest_changes;
use crate::tests::swift_manifest_changes::analyze_swift_manifest_changes;
use crate::tests::swift_resolved_changes::analyze_swift_resolved_changes;
use anyhow::Context;

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
        let sources = visible_paths.source_store_for(&root);
        let config = no_mistakes::config::v2::load_v2_config_from_source_store(
            &root,
            args.config.as_deref(),
            &root_visible_paths,
            &sources,
        )?;
        let mut collected = collect_changed_files(&args, &root)?;
        let mut changes_input = crate::tests::plan::changes::ChangesInput::load_from_parts(
            &args, &root, &config, &sources,
        );
        if let Some(input) = &mut changes_input {
            if let Ok(report) = &input.report {
                for path in &report.changed_files {
                    if let Err(error) = collected.push_manual_path(Path::new(path), &root) {
                        input.report = Err(format!(
                            "Invalid changedFiles entry `{path}` in changes report: {error}"
                        ));
                        break;
                    }
                }
            }
        }

        Ok(Self {
            args,
            root,
            visible_paths,
            root_visible_paths,
            config,
            config_path,
            collected,
            changes_input,
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
            ..
        } = &self;
        let tsconfig = no_mistakes::codebase::ts_resolver::resolve_tsconfig_from_visible(
            args.tsconfig.as_deref(),
            root,
            root_visible_paths,
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
        let sources = visible_paths.source_store_for(root);
        let revisions = RevisionSources::prepare(root, args, Arc::clone(&sources));
        let changed_files = existing_changed_files(collected);
        let workspace_map =
            no_mistakes::codebase::workspaces::load_from_source_store(root, &sources)
                .unwrap_or_default();
        let package_manifest_analysis = analyze_package_manifest_changes(
            root,
            &collected.files,
            &revisions,
            &workspace_map,
            &sources,
        );
        let lockfile_analysis = analyze_lockfile_changes(root, &collected.files, &revisions);
        let swift_resolved_analysis =
            analyze_swift_resolved_changes(root, config, &collected.files, &revisions);
        let swift_manifest_analysis =
            analyze_swift_manifest_changes(root, config, &collected.files, &revisions);
        let dotnet_dependency_analysis =
            analyze_dotnet_dependency_changes(root, &collected.files, &revisions);
        let mut lockfile_changed_packages = lockfile_packages(root, &lockfile_analysis);
        lockfile_changed_packages.extend(package_manifest_analysis.changed_packages.iter().map(
            |(package, changed_file, manifest)| {
                (
                    package.clone(),
                    changed_file.clone(),
                    vec![manifest.clone()],
                )
            },
        ));
        let GraphInputs {
            tsconfig_catalog,
            graph_files,
            graph_plan,
            prepared_graph_config,
            prepared_test_projects,
            test_filter,
        } = self.prepare_graph(&tsconfig, &sources, &workspace_map)?;
        let Self {
            args,
            root,
            visible_paths,
            root_visible_paths,
            config,
            config_path,
            collected,
            changes_input,
        } = self;

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
            changes_input,
            config_invalidation: OnceLock::new(),
            graph_builds: AtomicUsize::new(0),
            framework_discoveries: AtomicUsize::new(0),
        })
    }
}
