use super::Framework;
use std::path::{Path, PathBuf};

const PLAYWRIGHT_CONFIGS: &[&str] = &[
    "playwright.config.ts",
    "playwright.config.mts",
    "playwright.config.cts",
    "playwright.config.js",
    "playwright.config.mjs",
    "playwright.config.cjs",
];
const VITEST_CONFIGS: &[&str] = &[
    "vitest.config.ts",
    "vitest.config.mts",
    "vitest.config.cts",
    "vitest.config.js",
    "vitest.config.mjs",
    "vitest.config.cjs",
];

pub(crate) fn discovered_config_paths(
    root: &Path,
    framework: Framework,
    visible_paths: &[PathBuf],
) -> Vec<String> {
    let names = match framework {
        Framework::Dotnet | Framework::Swift => &[],
        Framework::Playwright => PLAYWRIGHT_CONFIGS,
        Framework::Vitest => VITEST_CONFIGS,
    };
    names
        .iter()
        .filter(|name| {
            let candidate = crate::codebase::ts_resolver::normalize_path(&root.join(name));
            visible_paths
                .iter()
                .any(|path| crate::codebase::ts_resolver::normalize_path(path) == candidate)
        })
        .map(|name| (*name).to_string())
        .collect()
}
