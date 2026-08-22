//! Detect which test frameworks a repo uses, for `impacted-checks`.

use crate::config::v2::schema::{NoMistakesConfig, TestPlanFrameworkConfig};
use crate::tests::TestFramework;
use std::path::{Path, PathBuf};

/// A framework is "present" when it is explicitly configured or its config file
/// exists at the repo root (matching how `tests plan` discovers frameworks).
pub(super) fn framework_present(
    root: &Path,
    config: &NoMistakesConfig,
    framework: TestFramework,
    visible_paths: &[PathBuf],
) -> bool {
    match framework {
        TestFramework::Dotnet => dotnet_present(config),
        TestFramework::Vitest => vitest_present(root, config, visible_paths),
        TestFramework::Playwright => playwright_present(root, config, visible_paths),
        TestFramework::Swift => swift_present(config),
        TestFramework::Python => {
            nonempty_or_plan(&config.tests.python.packages, &config.test_plan.python)
        }
        TestFramework::Go => nonempty_or_plan(&config.tests.go.modules, &config.test_plan.go),
        TestFramework::Cargo => {
            nonempty_or_plan(&config.tests.rust.packages, &config.test_plan.cargo)
        }
        TestFramework::Rails => nonempty_or_plan(&config.tests.rails.apps, &config.test_plan.rails),
        TestFramework::Php => nonempty_or_plan(&config.tests.php.apps, &config.test_plan.php),
        TestFramework::Java => {
            nonempty_or_plan(&config.tests.java.packages, &config.test_plan.java)
        }
        TestFramework::Kotlin => {
            nonempty_or_plan(&config.tests.kotlin.packages, &config.test_plan.kotlin)
        }
        TestFramework::Jest => jest_present(config),
    }
}

fn nonempty_or_plan<T>(items: &[T], plan: &TestPlanFrameworkConfig) -> bool {
    !items.is_empty() || test_plan_configured(plan)
}

fn dotnet_present(config: &NoMistakesConfig) -> bool {
    let c = &config.tests.dotnet;
    !c.projects.is_empty()
        || !c.solutions.is_empty()
        || test_plan_configured(&config.test_plan.dotnet)
}

fn vitest_present(root: &Path, config: &NoMistakesConfig, visible_paths: &[PathBuf]) -> bool {
    let c = &config.tests.vitest;
    c.configs.is_some()
        || !c.projects.is_empty()
        || test_plan_configured(&config.test_plan.vitest)
        // Only `vitest.config.*` proves Vitest — a bare `vite.config.*`
        // may belong to a Vite app that uses Jest/Mocha.
        || config_file_present(root, &["vitest.config"], visible_paths)
}

fn playwright_present(root: &Path, config: &NoMistakesConfig, visible_paths: &[PathBuf]) -> bool {
    let c = &config.tests.playwright;
    c.configs.is_some()
        || !c.projects.is_empty()
        || test_plan_configured(&config.test_plan.playwright)
        || config_file_present(root, &["playwright.config"], visible_paths)
}

fn swift_present(config: &NoMistakesConfig) -> bool {
    let c = &config.tests.swift;
    !c.packages.is_empty()
        || !c.projects.is_empty()
        || test_plan_configured(&config.test_plan.swift)
}

fn jest_present(config: &NoMistakesConfig) -> bool {
    let c = &config.tests.jest;
    c.configs.is_some() || !c.projects.is_empty() || test_plan_configured(&config.test_plan.jest)
}

/// True when the framework has any `testPlan` configuration — environments or
/// full-suite (dependency) triggers — that `tests plan` would act on.
fn test_plan_configured(plan: &TestPlanFrameworkConfig) -> bool {
    !plan.environments.is_empty()
        || !plan.full_suite_triggers.projects.is_empty()
        || !plan.full_suite_triggers.triggers.is_empty()
        || !plan.full_suite_triggers.ignore_changed_tests.is_empty()
        || plan.deprecated_dependencies_key
}

fn config_file_present(root: &Path, stems: &[&str], visible_paths: &[PathBuf]) -> bool {
    const EXTENSIONS: &[&str] = &["ts", "mts", "cts", "js", "mjs", "cjs"];
    stems.iter().any(|stem| {
        EXTENSIONS.iter().any(|ext| {
            let candidate =
                crate::codebase::ts_resolver::normalize_path(&root.join(format!("{stem}.{ext}")));
            visible_paths
                .iter()
                .any(|path| crate::codebase::ts_resolver::normalize_path(path) == candidate)
        })
    })
}
