use super::{
    is_vitest_project_config, parse_string_project_with_resolver, slash_path,
    visible_folder_config_glob,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[test]
fn project_config_suffixes_are_executable_vitest_configs() {
    for path in [
        "vitest.config.unit.ts",
        "vite.config.e2e.js",
        "vitest.unit.config.ts",
        "vite.e2e.config.js",
        "vitest.workspace.mts",
        "vitest.projects.cjs",
    ] {
        assert!(is_vitest_project_config(Path::new(path)), "{path}");
    }
    for path in [
        "vitest.config.unit.d.ts",
        "vite.config.d.mts",
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
    let candidate = slash_path(Path::new(r"C:\repo\packages\e2e\vitest.config.ts"));
    assert!(!pattern.contains('\\'));
    assert!(!candidate.contains('\\'));

    let normalized_pattern = pattern.replace("configs/../", "");
    assert!(visible_folder_config_glob(&normalized_pattern)
        .unwrap()
        .is_match(candidate));
}
