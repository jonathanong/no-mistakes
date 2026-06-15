//! Detect which test frameworks a repo uses, for `impacted-checks`.

use crate::config::v2::schema::NoMistakesConfig;
use crate::tests::TestFramework;
use std::path::Path;

/// A framework is "present" when it is explicitly configured or its config file
/// exists at the repo root (matching how `tests plan` discovers frameworks).
pub(super) fn framework_present(
    root: &Path,
    config: &NoMistakesConfig,
    framework: TestFramework,
) -> bool {
    match framework {
        TestFramework::Vitest => {
            let c = &config.tests.vitest;
            c.configs.is_some()
                || !c.projects.is_empty()
                || !config.test_plan.vitest.environments.is_empty()
                || config_file_present(root, &["vitest.config", "vite.config"])
        }
        TestFramework::Playwright => {
            let c = &config.tests.playwright;
            c.configs.is_some()
                || !c.projects.is_empty()
                || !config.test_plan.playwright.environments.is_empty()
                || config_file_present(root, &["playwright.config"])
        }
        TestFramework::Swift => {
            let c = &config.tests.swift;
            !c.packages.is_empty()
                || !c.projects.is_empty()
                || !config.test_plan.swift.environments.is_empty()
        }
    }
}

fn config_file_present(root: &Path, stems: &[&str]) -> bool {
    const EXTENSIONS: &[&str] = &["ts", "mts", "cts", "js", "mjs", "cjs"];
    stems.iter().any(|stem| {
        EXTENSIONS
            .iter()
            .any(|ext| root.join(format!("{stem}.{ext}")).exists())
    })
}
