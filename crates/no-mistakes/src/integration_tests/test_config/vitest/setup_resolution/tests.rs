use super::*;
use crate::codebase::ts_resolver::ImportClassification;
use std::collections::HashSet;

struct MissingSourceResolver;

impl ImportResolution for MissingSourceResolver {
    fn resolve(&self, _: &str, _: &Path) -> Option<PathBuf> {
        unreachable!("a missing setup source has no imports to resolve")
    }

    fn resolution_candidates(&self, _: &str, _: &Path) -> BTreeSet<PathBuf> {
        unreachable!("a missing setup source has no imports to resolve")
    }

    fn visible_files(&self) -> Option<&HashSet<PathBuf>> {
        None
    }

    fn classify_import(
        &self,
        _: &str,
        _: &Path,
        _: &crate::codebase::workspaces::IndexedWorkspaceMap,
        _: &HashSet<PathBuf>,
    ) -> ImportClassification {
        unreachable!("a missing setup source has no imports to classify")
    }
}

#[test]
fn deleted_runtime_setup_source_skips_transitive_import_walk() {
    // This fixture deliberately leaves only runtime.d.ts after runtime.ts is
    // deleted. The missing source must not abort runner-config parsing.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/vitest-declaration-runtime-deleted");
    let mut candidates = BTreeSet::new();

    runtime_setup_candidates(
        &root.join("setup/runtime.ts"),
        &root,
        &MissingSourceResolver,
        &mut candidates,
    );

    assert!(candidates.is_empty());
}
