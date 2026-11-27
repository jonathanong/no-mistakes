use super::AnalysisSession;
use crate::codebase::testing_analysis::{self, vitest_policy::Diagnostics};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl AnalysisSession {
    pub fn prepare_vitest_project_policy(
        &self,
        root: &Path,
        config: &NoMistakesConfig,
        files: &[PathBuf],
        sources: &SourceStore,
    ) -> anyhow::Result<()> {
        let hooks = testing_analysis::hooks();
        let rule_id = hooks.vitest_policy_rule_id;
        if self.check_outcomes.vitest_project_policy.get().is_some()
            || !config.rule_configured(rule_id)
        {
            return Ok(());
        }
        let result = crate::perf_trace::rule("analysis.rules.vitest-project-policy", || {
            (hooks.vitest_policy_diagnostics)(root, config, files, sources, true)
        });
        let diagnostics = self
            .capture_check_result(rule_id, result)?
            .unwrap_or_default();
        self.set_prepared_vitest_project_policy(Arc::new(diagnostics));
        Ok(())
    }

    pub fn set_prepared_vitest_project_policy(&self, diagnostics: Arc<Diagnostics>) {
        for failure in &diagnostics.incomplete {
            self.record_check_failure(failure.clone());
        }
        let _ = self.check_outcomes.vitest_project_policy.set(diagnostics);
    }

    pub fn prepared_vitest_project_policy(&self) -> Option<Arc<Diagnostics>> {
        self.check_outcomes.vitest_project_policy.get().cloned()
    }
}
