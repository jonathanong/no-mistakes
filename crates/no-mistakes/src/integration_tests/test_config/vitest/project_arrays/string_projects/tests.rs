use super::{
    folder_configs::visible_folder_config_glob, is_vitest_project_config,
    parse_string_project_with_resolver, slash_path, string_project_paths_with_resolver,
    string_project_roots_with_resolver,
};
use crate::codebase::ts_resolver::{ImportClassification, ImportResolution, VisiblePathLookup};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

struct DirectProjectResolver {
    target: PathBuf,
}

impl ImportResolution for DirectProjectResolver {
    fn resolve(&self, _: &str, _: &Path) -> Option<PathBuf> {
        Some(self.target.clone())
    }

    fn resolution_candidates(&self, _: &str, _: &Path) -> BTreeSet<PathBuf> {
        BTreeSet::from([self.target.clone()])
    }

    fn visible_files(&self) -> Option<&dyn crate::codebase::ts_resolver::VisiblePathLookup> {
        None
    }

    fn classify_import(
        &self,
        _: &str,
        _: &Path,
        _: &crate::codebase::workspaces::IndexedWorkspaceMap,
        _: &dyn crate::codebase::ts_resolver::VisiblePathLookup,
    ) -> ImportClassification {
        unreachable!("direct project resolution does not classify imports")
    }
}

#[test]
fn project_config_suffixes_are_executable_vitest_configs() {
    for path in [
        "vitest.config.unit.ts",
        "vite.config.e2e.js",
        "vitest.unit.config.ts",
        "vite.e2e.config.js",
    ] {
        assert!(is_vitest_project_config(Path::new(path)), "{path}");
    }
    for path in [
        "vitest.config.unit.d.ts",
        "vite.config.d.mts",
        "vitest.workspace.mts",
        "vitest.projects.cjs",
        "vitest.foo.bar.config.ts",
        "vite.foo.bar.config.ts",
        "vitest.foo!.config.ts",
        "vite.foo!.config.ts",
        "vite.workspace.ts",
        "vitest.config.tsx",
        "vitest.unit.config.jsx",
        "vitest.workspace.tsx",
        "vitest.projects.tsx",
    ] {
        assert!(!is_vitest_project_config(Path::new(path)), "{path}");
    }
}

#[test]
fn only_named_json_project_arrays_are_supported() {
    assert!(crate::integration_tests::is_vitest_project_array_path(
        Path::new("vitest.workspace.json")
    ));
    assert!(crate::integration_tests::is_vitest_project_array_path(
        Path::new("vitest.projects.json")
    ));
    assert!(!crate::integration_tests::is_vitest_project_array_path(
        Path::new("custom.json")
    ));
    assert!(!crate::integration_tests::is_vitest_project_array_path(
        Path::new("vitest.workspace")
    ));
}

#[test]
fn active_string_project_cycles_are_skipped() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-config/vitest-workspace-json/string-project"),
    );
    let path = root.join("vitest.config.ts");
    let tsconfig = crate::integration_tests::test_support::tsconfig_without_config(&root);
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    let mut seen = BTreeSet::from([path.clone()]);

    assert!(
        parse_string_project_with_resolver(&path, &resolver, &mut seen)
            .unwrap()
            .is_none()
    );
    seen.clear();
    assert!(
        parse_string_project_with_resolver(&path, &resolver, &mut seen)
            .unwrap()
            .is_some()
    );
}

#[test]
fn absolute_project_globs_use_slashes_for_windows_paths() {
    let pattern = slash_path(Path::new(r"C:\repo\configs\..\packages\*"));
    let candidate = slash_path(Path::new(r"C:\repo\packages\e2e"));
    assert!(!pattern.contains('\\'));
    assert!(!candidate.contains('\\'));

    let normalized_pattern = pattern.replace("configs/../", "");
    assert!(visible_folder_config_glob(&normalized_pattern)
        .unwrap()
        .is_match(candidate));
}

#[test]
fn direct_project_paths_resolve_without_a_visible_file_catalog() {
    let target = PathBuf::from("/repo/packages/unit/vitest.config.ts");
    let resolver = DirectProjectResolver {
        target: target.clone(),
    };

    assert_eq!(
        string_project_paths_with_resolver(
            "./packages/unit/vitest.config.ts",
            Path::new("/repo/vitest.config.ts"),
            &resolver,
        ),
        [target]
    );
}

struct CountingVisible {
    paths: Vec<PathBuf>,
    cache_key_calls: AtomicUsize,
}

impl VisiblePathLookup for CountingVisible {
    fn contains_visible(&self, path: &Path) -> bool {
        self.paths.iter().any(|candidate| candidate == path)
    }

    fn visible_len(&self) -> usize {
        self.paths.len()
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        self.cache_key_calls.fetch_add(1, Ordering::Relaxed);
        self.paths.clone()
    }
}

struct CountingVisibleResolver {
    visible: CountingVisible,
}

impl ImportResolution for CountingVisibleResolver {
    fn resolve(&self, _: &str, _: &Path) -> Option<PathBuf> {
        None
    }

    fn resolution_candidates(&self, _: &str, _: &Path) -> BTreeSet<PathBuf> {
        BTreeSet::new()
    }

    fn visible_files(&self) -> Option<&dyn VisiblePathLookup> {
        Some(&self.visible)
    }

    fn classify_import(
        &self,
        _: &str,
        _: &Path,
        _: &crate::codebase::workspaces::IndexedWorkspaceMap,
        _: &dyn VisiblePathLookup,
    ) -> ImportClassification {
        unreachable!("visible root matching does not classify imports")
    }
}

#[test]
fn string_project_roots_reuse_one_visible_snapshot_for_config_checks() {
    let resolver = CountingVisibleResolver {
        visible: CountingVisible {
            paths: vec![
                PathBuf::from("/repo/packages/a/src/a.test.ts"),
                PathBuf::from("/repo/packages/b/src/b.test.ts"),
                PathBuf::from("/repo/packages/c/vitest.config.ts"),
                PathBuf::from("/repo/packages/c/src/c.test.ts"),
            ],
            cache_key_calls: AtomicUsize::new(0),
        },
    };

    let roots = string_project_roots_with_resolver(
        "./packages/*",
        Path::new("/repo/vitest.config.ts"),
        &resolver,
    );

    assert_eq!(
        roots,
        [
            PathBuf::from("/repo/packages/a"),
            PathBuf::from("/repo/packages/b"),
        ]
    );
    assert_eq!(resolver.visible.cache_key_calls.load(Ordering::Relaxed), 1);
}

#[test]
fn string_project_paths_reuse_one_visible_snapshot_for_folder_configs() {
    let resolver = CountingVisibleResolver {
        visible: CountingVisible {
            paths: vec![
                PathBuf::from("/repo/packages/a/src/a.test.ts"),
                PathBuf::from("/repo/packages/c/vitest.config.ts"),
            ],
            cache_key_calls: AtomicUsize::new(0),
        },
    };

    let paths = string_project_paths_with_resolver(
        "./packages/*",
        Path::new("/repo/vitest.config.ts"),
        &resolver,
    );

    assert_eq!(paths, [PathBuf::from("/repo/packages/c/vitest.config.ts")]);
    assert_eq!(resolver.visible.cache_key_calls.load(Ordering::Relaxed), 1);
}

#[test]
fn folder_config_paths_prefer_named_vite_config_when_no_vitest_config_exists() {
    let paths = super::folder_configs::folder_config_paths(
        "./packages/*",
        Path::new("/repo/vitest.config.ts"),
        &[
            PathBuf::from("/repo/packages/c/vite.unit.config.ts"),
            PathBuf::from("/repo/packages/c/src/c.test.ts"),
        ],
    );
    assert_eq!(
        paths,
        [PathBuf::from("/repo/packages/c/vite.unit.config.ts")]
    );
}

#[test]
fn folder_config_paths_ignore_invalid_project_globs() {
    let paths = super::folder_configs::folder_config_paths(
        "./packages/**[",
        Path::new("/repo/vitest.config.ts"),
        std::slice::from_ref(&PathBuf::from("/repo/packages/c/vitest.config.ts")),
    );
    assert!(paths.is_empty());
}
