use super::*;
use crate::codebase::test_discovery::TestRunner;
use crate::config::v2::schema::NoMistakesConfig;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/python-test-plan/fixture"),
    )
}

#[test]
fn empty_language_lists_discover_no_projects() {
    let root = fixture_root();
    let config = NoMistakesConfig::default();
    for runner in [
        TestRunner::Python,
        TestRunner::Go,
        TestRunner::Cargo,
        TestRunner::Rails,
        TestRunner::Php,
        TestRunner::Java,
        TestRunner::Kotlin,
        TestRunner::Elixir,
    ] {
        assert!(language_projects(&root, &config, runner).is_empty());
    }
}

#[test]
fn python_projects_scope_configured_package_globs() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.tests.python.packages = vec!["app".to_string()];
    let projects = language_projects(&root, &config, TestRunner::Python);
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].config.as_deref(), Some("app"));
    assert!(projects[0]
        .include
        .iter()
        .any(|glob| glob.contains("test_*.py")));
}

#[test]
fn cargo_projects_read_package_name_from_manifest() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/cargo-test-plan/fixture"),
    );
    let mut config = NoMistakesConfig::default();
    config.tests.rust.packages = vec![
        "app".to_string(),
        "missing".to_string(),
        "unnamed".to_string(),
    ];
    let projects = language_projects(&root, &config, TestRunner::Cargo);
    assert_eq!(projects[0].runner_project_arg.as_deref(), Some("app"));
    assert_eq!(projects[1].runner_project_arg.as_deref(), Some("missing"));
    assert_eq!(projects[2].runner_project_arg.as_deref(), Some("unnamed"));
}

#[test]
fn non_language_runner_returns_no_projects() {
    let root = fixture_root();
    let config = NoMistakesConfig::default();
    assert!(language_projects(&root, &config, TestRunner::Vitest).is_empty());
}

#[test]
fn php_projects_preserve_laravel_framework() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/php-test-plan/fixture"),
    );
    let mut config = NoMistakesConfig::default();
    config.tests.php.apps = vec![".".to_string()];
    config.tests.php.framework = Some("laravel".to_string());
    let projects = language_projects(&root, &config, TestRunner::Php);
    assert_eq!(projects[0].runner_project_arg.as_deref(), Some("laravel"));
}

#[test]
fn java_projects_scope_configured_package_globs() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/lang-frontends/java-spring"),
    );
    let mut config = NoMistakesConfig::default();
    config.tests.java.packages = vec![".".to_string()];
    let projects = language_projects(&root, &config, TestRunner::Java);
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].config.as_deref(), Some("."));
    assert!(projects[0]
        .include
        .iter()
        .any(|glob| glob.contains("Test.java")));
}

#[test]
fn kotlin_projects_scope_configured_package_globs() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/lang-frontends/kotlin-spring"),
    );
    let mut config = NoMistakesConfig::default();
    config.tests.kotlin.packages = vec![".".to_string()];
    let projects = language_projects(&root, &config, TestRunner::Kotlin);
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].config.as_deref(), Some("."));
    assert!(projects[0]
        .include
        .iter()
        .any(|glob| glob.contains("Test.kt")));
}

#[test]
fn elixir_projects_scope_configured_app_globs() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/lang-frontends/phoenix-routes"),
    );
    let mut config = NoMistakesConfig::default();
    config.tests.elixir.apps = vec![".".to_string()];
    let projects = language_projects(&root, &config, TestRunner::Elixir);
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].config.as_deref(), Some("."));
    assert!(projects[0]
        .include
        .iter()
        .any(|glob| glob.contains("_test.exs")));
}
