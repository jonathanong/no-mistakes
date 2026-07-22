use super::{is_vitest_project_config, slash_path, visible_config_glob};
use std::path::Path;

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
fn absolute_project_globs_use_slashes_for_windows_paths() {
    let pattern = slash_path(Path::new(r"C:\repo\configs\..\packages\*\vitest.config.ts"));
    let candidate = slash_path(Path::new(r"C:\repo\packages\e2e\vitest.config.ts"));
    assert!(!pattern.contains('\\'));
    assert!(!candidate.contains('\\'));

    let normalized_pattern = pattern.replace("configs/../", "");
    assert!(visible_config_glob(&normalized_pattern)
        .unwrap()
        .is_match(candidate));
}
