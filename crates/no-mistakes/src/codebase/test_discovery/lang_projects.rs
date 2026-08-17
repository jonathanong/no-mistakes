use crate::config::v2::schema::NoMistakesConfig;
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::ConfigProject;
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

use super::types::TestRunner;

pub(super) fn language_projects(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Vec<ConfigProject> {
    let (roots, includes, project_arg) = match runner {
        TestRunner::Python => (
            config.tests.python.packages.as_slice(),
            python_includes(),
            None,
        ),
        TestRunner::Go => (config.tests.go.modules.as_slice(), go_includes(), None),
        TestRunner::Cargo => (
            config.tests.rust.packages.as_slice(),
            cargo_includes(),
            None,
        ),
        TestRunner::Rails => (config.tests.rails.apps.as_slice(), rails_includes(), None),
        TestRunner::Php => (
            config.tests.php.apps.as_slice(),
            php_includes(),
            config.tests.php.framework.as_deref(),
        ),
        _ => return Vec::new(),
    };
    if roots.is_empty() {
        return Vec::new();
    }
    roots
        .iter()
        .map(|entry| project_for(root, runner, entry, includes, project_arg))
        .collect()
}

fn project_for(
    root: &Path,
    runner: TestRunner,
    entry: &str,
    includes: &[String],
    project_arg: Option<&str>,
) -> ConfigProject {
    let slash = entry.trim_end_matches('/');
    let package_root = crate::codebase::ts_resolver::normalize_path(&root.join(slash));
    let runner_project_arg = match runner {
        TestRunner::Cargo => Some(cargo_package_name(&package_root, slash)),
        TestRunner::Php => project_arg.map(str::to_string),
        _ => None,
    };
    ConfigProject {
        config: Some(slash.to_string()),
        workspace: false,
        policy_name: Some(slash.to_string()),
        runner_project_arg,
        scope: Some(slash.to_string()),
        include: prefix_globs(root, &package_root, includes),
        exclude: Vec::new(),
        vitest_setup: Vec::new(),
    }
}

fn cargo_package_name(package_root: &Path, fallback: &str) -> String {
    let manifest = package_root.join("Cargo.toml");
    let Ok(source) = std::fs::read_to_string(&manifest) else {
        return fallback.to_string();
    };
    cargo_name_re()
        .captures(&source)
        .and_then(|cap| cap.get(1).map(|name| name.as_str().to_string()))
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn cargo_name_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*name\s*=\s*"([^"]+)""#).expect("cargo name"))
}

fn python_includes() -> &'static [String] {
    static GLOBS: OnceLock<Vec<String>> = OnceLock::new();
    GLOBS.get_or_init(|| crate::codebase::dependencies::test_globs("python"))
}

fn go_includes() -> &'static [String] {
    static GLOBS: OnceLock<Vec<String>> = OnceLock::new();
    GLOBS.get_or_init(|| crate::codebase::dependencies::test_globs("go"))
}

fn cargo_includes() -> &'static [String] {
    static GLOBS: OnceLock<Vec<String>> = OnceLock::new();
    GLOBS.get_or_init(|| crate::codebase::dependencies::test_globs("cargo"))
}

fn rails_includes() -> &'static [String] {
    static GLOBS: OnceLock<Vec<String>> = OnceLock::new();
    GLOBS.get_or_init(|| crate::codebase::dependencies::test_globs("rails"))
}

fn php_includes() -> &'static [String] {
    static GLOBS: OnceLock<Vec<String>> = OnceLock::new();
    GLOBS.get_or_init(|| crate::codebase::dependencies::test_globs("php"))
}

#[cfg(test)]
#[path = "lang_projects/tests.rs"]
mod tests;
