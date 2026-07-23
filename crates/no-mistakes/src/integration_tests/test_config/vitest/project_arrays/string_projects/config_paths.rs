use std::path::Path;

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn is_vitest_project_config(
    path: &Path,
) -> bool {
    // Default folder discovery only considers conventional executable config
    // filenames. Explicit project-file entries are handled separately.
    if !is_runtime_project_source(path) {
        return false;
    }
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy())
        .unwrap_or_default();
    stem == "vitest.workspace"
        || stem == "vitest.projects"
        || stem == "vitest.config"
        || stem.starts_with("vitest.config.")
        || stem == "vite.config"
        || stem.starts_with("vite.config.")
        || named_config_stem(stem.as_ref(), "vitest")
        || named_config_stem(stem.as_ref(), "vite")
}

pub(super) fn is_runtime_project_source(path: &Path) -> bool {
    const EXTENSIONS: &[&str] = &["mts", "ts", "mjs", "js", "cjs", "cts"];
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
    {
        return false;
    }
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| EXTENSIONS.contains(&extension))
}

fn named_config_stem(stem: &str, runner: &str) -> bool {
    let Some(name) = stem
        .strip_prefix(runner)
        .and_then(|stem| stem.strip_prefix('.'))
        .and_then(|stem| stem.strip_suffix(".config"))
    else {
        return false;
    };
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}
