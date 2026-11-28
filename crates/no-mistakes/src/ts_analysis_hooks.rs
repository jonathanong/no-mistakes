use std::path::{Path, PathBuf};
use std::sync::Arc;

use no_mistakes_ts_analysis::hooks::{AnalysisHooks, MetricsObserver};
use no_mistakes_ts_analysis::workspace_resolution::WorkspaceMap;

use crate::diagnostics::InvocationObserver;

impl MetricsObserver for InvocationObserver {
    fn increment(&self, metric: &'static str, amount: u64) {
        InvocationObserver::increment(self, metric, amount);
    }
}

struct NoMistakesTsAnalysisHooks;

impl AnalysisHooks for NoMistakesTsAnalysisHooks {
    fn current_observer(&self) -> Option<Arc<dyn MetricsObserver>> {
        crate::diagnostics::current().map(|observer| observer as Arc<dyn MetricsObserver>)
    }

    fn discover_visible_paths(&self, root: &Path) -> Vec<PathBuf> {
        crate::codebase::ts_source::discover_visible_paths(root)
    }

    fn load_workspace_from_files(
        &self,
        root: &Path,
        visible_paths: &[PathBuf],
    ) -> Option<WorkspaceMap> {
        crate::codebase::package_analysis::load_from_files(root, visible_paths).ok()
    }
}

static TS_ANALYSIS_HOOKS: NoMistakesTsAnalysisHooks = NoMistakesTsAnalysisHooks;

pub(crate) fn ensure_registered() {
    no_mistakes_ts_analysis::hooks::register_hooks(&TS_ANALYSIS_HOOKS);
}
