use super::*;
use crate::codebase::ts_resolver::{ImportClassification, ImportResolution};
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

struct MissingWorkspaceSourceResolver {
    missing_path: PathBuf,
    resolved_missing_source: AtomicBool,
}

impl ImportResolution for MissingWorkspaceSourceResolver {
    fn resolve(&self, specifier: &str, _: &Path) -> Option<PathBuf> {
        (specifier == "./missing-projects.cjs").then(|| {
            self.resolved_missing_source.store(true, Ordering::Relaxed);
            self.missing_path.clone()
        })
    }

    fn resolution_candidates(&self, specifier: &str, _: &Path) -> BTreeSet<PathBuf> {
        self.resolve(specifier, Path::new("")).into_iter().collect()
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
        unreachable!("workspace export parsing only resolves direct literal requires")
    }
}

fn saved_fixture(name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config")
        .join(name);
    crate::test_support::materialize_saved_fixture(&source)
}

#[test]
fn vitest_workspace_exports_handle_namespace_reexports_and_safe_empty_forms() {
    for (fixture_name, extension, expected) in [
        (
            "vitest-workspace-namespace",
            "ts",
            Some("namespace-project"),
        ),
        (
            "vitest-workspace-local-and-type",
            "ts",
            Some("local-export-project"),
        ),
        (
            "vitest-workspace-star-fallback",
            "ts",
            Some("star-fallback-project"),
        ),
        ("vitest-workspace-cycle", "ts", None),
        ("vitest-workspace-missing-import", "ts", None),
        ("vitest-workspace-empty-forms", "ts", None),
        ("vitest-workspace-missing-binding", "ts", None),
        ("vitest-workspace-invalid-namespace", "ts", None),
        ("vitest-workspace-import-cycle", "ts", None),
        ("vitest-workspace-default-class", "ts", None),
        ("vitest-workspace-empty-define", "ts", None),
        (
            "vitest-workspace-commonjs-filtering",
            "cjs",
            Some("commonjs-filter-project"),
        ),
        (
            "vitest-workspace-commonjs-require",
            "cjs",
            Some("commonjs-required-workspace-project"),
        ),
        ("vitest-workspace-commonjs-factory", "cjs", None),
        ("vitest-workspace-commonjs-dynamic-require", "cjs", None),
    ] {
        let fixture = saved_fixture(fixture_name);
        let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
        let path = root.join(format!("vitest.workspace.{extension}"));
        let source = std::fs::read_to_string(&path).unwrap();
        let projects = parse_vitest_fixture(&source, &path, &root).unwrap();

        assert_eq!(
            projects
                .first()
                .and_then(|project| project.policy_name.as_deref()),
            expected,
            "{fixture_name}",
        );
        assert_eq!(
            projects.len(),
            usize::from(expected.is_some()),
            "{fixture_name}"
        );
    }
}

#[test]
fn missing_literal_commonjs_workspace_require_is_a_safe_empty_export() {
    let fixture = saved_fixture("vitest-workspace-commonjs-missing-require");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.workspace.cjs");
    let source = std::fs::read_to_string(&path).unwrap();
    let resolver = MissingWorkspaceSourceResolver {
        missing_path: root.join("missing-projects.cjs"),
        resolved_missing_source: AtomicBool::new(false),
    };
    let projects =
        crate::integration_tests::runner_config::with_program(&path, &source, |program, source| {
            crate::integration_tests::test_config::vitest::parse_program_with_resolver(
                program, source, &path, &root, &root, &resolver,
            )
        })
        .unwrap()
        .unwrap();

    assert!(resolver.resolved_missing_source.load(Ordering::Relaxed));
    assert!(projects.is_empty());
}
