//! Detect which test frameworks a repo uses, for `impacted-checks`.

use crate::config::v2::schema::{NoMistakesConfig, TestPlanFrameworkConfig};
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
        TestFramework::Dotnet => {
            let c = &config.tests.dotnet;
            !c.projects.is_empty()
                || !c.solutions.is_empty()
                || test_plan_configured(&config.test_plan.dotnet)
        }
        TestFramework::Vitest => {
            let c = &config.tests.vitest;
            c.configs.is_some()
                || !c.projects.is_empty()
                || test_plan_configured(&config.test_plan.vitest)
                // Only `vitest.config.*` proves Vitest — a bare `vite.config.*`
                // may belong to a Vite app that uses Jest/Mocha.
                || config_file_present(root, &["vitest.config"])
        }
        TestFramework::Playwright => {
            let c = &config.tests.playwright;
            c.configs.is_some()
                || !c.projects.is_empty()
                || test_plan_configured(&config.test_plan.playwright)
                || config_file_present(root, &["playwright.config"])
        }
        TestFramework::Swift => {
            let c = &config.tests.swift;
            !c.packages.is_empty()
                || !c.projects.is_empty()
                || test_plan_configured(&config.test_plan.swift)
        }
    }
}

/// True when the framework has any `testPlan` configuration — environments or
/// full-suite (dependency) triggers — that `tests plan` would act on.
fn test_plan_configured(plan: &TestPlanFrameworkConfig) -> bool {
    !plan.environments.is_empty()
        || !plan.full_suite_triggers.projects.is_empty()
        || !plan.full_suite_triggers.ignore_changed_tests.is_empty()
        || plan.deprecated_dependencies_key
}

fn config_file_present(root: &Path, stems: &[&str]) -> bool {
    const EXTENSIONS: &[&str] = &["ts", "mts", "cts", "js", "mjs", "cjs"];
    stems.iter().any(|stem| {
        EXTENSIONS
            .iter()
            .any(|ext| root.join(format!("{stem}.{ext}")).exists())
    })
}
