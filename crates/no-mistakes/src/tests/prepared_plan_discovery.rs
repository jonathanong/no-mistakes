use super::{test_framework, test_runner, PreparedTestPlanRequest, TestFramework};
use crate::tests::{config_invalidation, plan, Warning};
use anyhow::Result;
use no_mistakes::codebase::test_discovery::{DiscoveredTests, TestRunner};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

impl PreparedTestPlanRequest {
    pub(crate) fn tsconfig_warnings(&self) -> Vec<Warning> {
        self.tsconfig_catalog
            .diagnostics()
            .into_iter()
            .map(|diagnostic| Warning {
                r#type: match diagnostic.kind {
                    no_mistakes::codebase::ts_resolver::TsConfigDiagnosticKind::AmbiguousOwnership => {
                        "tsconfig-ambiguous-ownership"
                    }
                    no_mistakes::codebase::ts_resolver::TsConfigDiagnosticKind::InvalidConfig => {
                        "tsconfig-invalid-config"
                    }
                    no_mistakes::codebase::ts_resolver::TsConfigDiagnosticKind::InvalidExtends => {
                        "tsconfig-invalid-extends"
                    }
                    no_mistakes::codebase::ts_resolver::TsConfigDiagnosticKind::InvalidReference => {
                        "tsconfig-invalid-reference"
                    }
                }
                .to_string(),
                message: diagnostic.detail,
                file: diagnostic
                    .file
                    .or(diagnostic.config)
                    .and_then(|path| path.strip_prefix(&self.root).ok().map(Path::to_path_buf))
                    .unwrap_or_default()
                    .to_string_lossy()
                    .replace('\\', "/"),
                line: None,
            })
            .collect()
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
        let trigger_file =
            config_invalidation::changed_config_path(&self.args, &self.root, &self.collected)?;
        let comparison = self.config_invalidation.get_or_init(|| {
            config_invalidation::compare_changed_config(&self.args, &self.root, &self.collected)
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
                    plan::relative_path(&self.root, &trigger_file)
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
