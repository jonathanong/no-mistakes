use super::Escapes;
use crate::fx::FxHashSet;
use std::path::Path;

impl Escapes {
    pub(in crate::codebase::rules::test_no_live_repository_scan) fn with_local(
        file: &Path,
        name: &str,
    ) -> Self {
        let mut escapes = Self::default();
        let mut names = FxHashSet::default();
        names.insert(name.to_string());
        escapes.local.insert(file.to_path_buf(), names);
        escapes
    }

    pub(in crate::codebase::rules::test_no_live_repository_scan) fn with_reference(
        file: &Path,
        name: &str,
        line: u32,
    ) -> Self {
        let mut escapes = Self::with_local(file, name);
        escapes
            .refs
            .push((file.to_path_buf(), 1, line, name.to_string()));
        escapes
    }
}
