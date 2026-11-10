use super::ConfiguredSetups;
use std::path::Path;

impl ConfiguredSetups {
    pub(in crate::codebase::rules::test_no_live_repository_scan) fn unknown_without_gap(
        root: &Path,
    ) -> Self {
        Self {
            root: root.to_path_buf(),
            projects: Vec::new(),
            known: false,
            configs_known: false,
            gap: None,
        }
    }
}
