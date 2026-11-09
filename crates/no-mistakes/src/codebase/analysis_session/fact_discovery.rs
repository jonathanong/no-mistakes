use super::AnalysisSession;
use crate::codebase::check_facts::FamilyPathScope;
use std::path::{Path, PathBuf};

impl AnalysisSession {
    #[doc(hidden)]
    pub fn prepare_integration_fact_discovery(
        &self,
        root: &Path,
        files: &[PathBuf],
        config: &crate::config::v2::NoMistakesConfig,
    ) {
        self.check_outcomes.integration_discovery.get_or_init(|| {
            crate::integration_tests::config::test_discovery_scope(root, files, config)
        });
    }

    pub(crate) fn integration_fact_discovery(&self) -> &FamilyPathScope {
        self.check_outcomes
            .integration_discovery
            .get()
            .unwrap_or(&FamilyPathScope::All)
    }
}
