use super::*;
use crate::playwright::test_support::fixture_path;
use load::helpers::is_playwright_config_name;

#[test]
fn missing_default_config_uses_defaults() {
    let root = fixture_path(&["scan-config", "missing-default"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    assert_eq!(settings.frontend_root, "app");
    assert!(settings.playwright_configs.is_empty());
    assert_eq!(settings.selector_attributes, vec!["data-testid", "data-pw"]);
    assert!(settings.component_selector_attributes.is_empty());
    assert!(!settings.html_ids);
    assert_eq!(settings.selector_roots, vec!["app"]);
}

#[test]
fn explicit_missing_config_errors() {
    let root = fixture_path(&["scan-config", "missing-default"]);
    let err = load_settings(&root, Some(Path::new("missing.yaml")), &[], None)
        .err()
        .expect("expected missing config to fail");
    assert!(err.to_string().contains("config file does not exist"));
}

#[test]
fn explicit_generic_v2_config_uses_playwright_settings() {
    let root = fixture_path(&["scan-config", "explicit-v2-config"]);
    let settings = load_settings(&root, Some(Path::new("configs/ci.yaml")), &[], None).unwrap();
    assert_eq!(settings.frontend_root, "explicit-app");
    assert_eq!(settings.selector_attributes, vec!["data-explicit"]);
    assert_eq!(
        settings.playwright_configs,
        vec![root.join("playwright.explicit.config.ts")]
    );
}

#[test]
fn explicit_v2_config_without_playwright_settings_uses_defaults() {
    let root = fixture_path(&["scan-config", "explicit-v2-config"]);
    let settings = load_settings(
        &root,
        Some(Path::new("configs/no-playwright.yaml")),
        &[PathBuf::from("playwright.cli.config.ts")],
        Some("web".to_string()),
    )
    .unwrap();
    assert_eq!(settings.frontend_root, "app");
    assert_eq!(
        settings.playwright_configs,
        vec![root.join("playwright.cli.config.ts")]
    );
    assert_eq!(settings.project, Some("web".to_string()));
    assert_eq!(settings.selector_attributes, vec!["data-testid", "data-pw"]);
}

#[test]
fn v2_cli_playwright_configs_override_file_settings() {
    let root = fixture_path(&["scan-config", "explicit-v2-config"]);
    let settings = load_settings(
        &root,
        Some(Path::new("configs/ci.yaml")),
        &[PathBuf::from("playwright.cli.config.ts")],
        None,
    )
    .unwrap();
    assert_eq!(
        settings.playwright_configs,
        vec![root.join("playwright.cli.config.ts")]
    );
}

#[test]
fn v2_without_config_paths_finds_default_playwright_config() {
    let root = fixture_path(&["scan-config", "no-mistakes-v2-default-playwright"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    assert_eq!(settings.frontend_root, "v2-default-app");
    assert_eq!(settings.selector_attributes, vec!["data-testid", "data-pw"]);
    assert_eq!(
        settings.playwright_configs,
        vec![root.join("playwright.config.ts")]
    );
}

#[test]
fn cli_playwright_configs_override_file_settings() {
    let root = fixture_path(&["scan-config", "full"]);
    let settings = load_settings(
        &root,
        None,
        &[PathBuf::from("playwright.cli.config.ts")],
        None,
    )
    .unwrap();
    assert_eq!(
        settings.playwright_configs,
        vec![root.join("playwright.cli.config.ts")]
    );
}

#[test]
fn duplicate_no_mistakes_configs_error() {
    let root = fixture_path(&["scan-config", "multiple-no-mistakes"]);
    let err = load_settings(&root, None, &[], None)
        .err()
        .expect("expected duplicate config files to fail");
    assert!(err.to_string().contains("multiple config files found"));
}

#[test]
fn reads_yaml_and_finds_default_playwright_config() {
    let root = fixture_path(&["scan-config", "full"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    assert_eq!(settings.frontend_root, "web/app");
    assert_eq!(settings.test_exclude, vec!["**/skip/**"]);
    assert_eq!(settings.navigation_helpers, vec!["navigateTo"]);
    assert!(settings.html_ids);
    assert_eq!(settings.selector_roots, vec!["web/components"]);
    assert_eq!(settings.selector_include, vec!["web/components/**/*.tsx"]);
    assert_eq!(settings.selector_exclude, vec!["**/*.test.tsx"]);
    assert_eq!(
        settings.playwright_configs,
        vec![root.join("playwright.config.mts")]
    );
}

#[test]
fn no_mistakes_config_has_priority_and_supports_nesting() {
    let root = fixture_path(&["scan-config", "no-mistakes-priority"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    assert_eq!(settings.frontend_root, "no-mistakes-app");

    let root = fixture_path(&["scan-config", "no-mistakes-nested"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    assert_eq!(settings.frontend_root, "nested-app");
}

#[test]
fn no_mistakes_v2_config_loads_playwright_settings() {
    let root = fixture_path(&["scan-config", "no-mistakes-v2-priority"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    assert_eq!(settings.frontend_root, "v2-app");
    assert_eq!(settings.test_include, vec!["tests/**/*.spec.ts"]);
    assert_eq!(settings.test_exclude, vec!["tests/flaky/**"]);
    assert_eq!(settings.ignore_routes, vec!["/ignored"]);
    assert_eq!(settings.navigation_helpers, vec!["navigateTo"]);
    assert_eq!(settings.selector_attributes, vec!["data-v2"]);
    assert_eq!(settings.selector_roots, vec!["v2-components"]);
    assert_eq!(settings.selector_include, vec!["v2-components/**/*.tsx"]);
    assert_eq!(
        settings.playwright_configs,
        vec![root.join("playwright.v2.config.ts")]
    );
}

#[test]
fn v2_playwright_ignore_routes_empty_is_preserved() {
    let root = fixture_path(&["scan-config", "no-mistakes-v2-clear-ignore-routes"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    assert!(
        settings.ignore_routes.is_empty(),
        "explicit empty ignoreRoutes should be preserved"
    );
}

#[test]
fn v2_playwright_frontend_root_and_ignore_routes() {
    let root = fixture_path(&["scan-config", "no-mistakes-v2-playwright-routes"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    assert_eq!(settings.frontend_root, "pw-app");
    assert_eq!(settings.ignore_routes, vec!["/admin/**", "/api/**"]);
    assert_eq!(settings.selector_roots, vec!["pw-app"]);
    assert_eq!(
        settings.playwright_configs,
        vec![root.join("playwright.config.ts")]
    );
}

#[test]
fn test_is_playwright_config_name_edge_cases() {
    assert!(!is_playwright_config_name(Path::new("")));
    assert!(!is_playwright_config_name(Path::new(
        "playwright.config.txt"
    )));
    assert!(!is_playwright_config_name(Path::new(
        "notplaywright.config.ts"
    )));
    assert!(!is_playwright_config_name(Path::new("playwright.config")));
    assert!(!is_playwright_config_name(Path::new("playwrightconfig")));
}

#[test]
fn test_playwright_config_from_file() {
    let root = fixture_path(&["scan-config", "playwright-config-array"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    assert_eq!(settings.playwright_configs.len(), 2);
    assert!(settings.playwright_configs[0].ends_with("playwright.config.ts"));
    assert!(settings.playwright_configs[1].ends_with("playwright.other.config.ts"));

    let root = fixture_path(&["scan-config", "playwright-config-single"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    assert_eq!(settings.playwright_configs.len(), 1);
    assert!(settings.playwright_configs[0].ends_with("playwright.config.ts"));
}

#[test]
fn test_has_configured_html_id_selector_false() {
    let mut settings = load_settings(
        &fixture_path(&["scan-config", "missing-default"]),
        None,
        &[],
        None,
    )
    .unwrap();
    settings.selector_attributes = vec!["data-testid".to_string(), "data-pw".to_string()];
    settings.component_selector_attributes = BTreeMap::new();
    settings
        .component_selector_attributes
        .insert("Button".to_string(), "data-testid".to_string());

    assert!(!has_configured_html_id_selector(&settings));
}

#[test]
fn test_has_configured_html_id_selector_true_attributes() {
    let mut settings = load_settings(
        &fixture_path(&["scan-config", "missing-default"]),
        None,
        &[],
        None,
    )
    .unwrap();
    settings.selector_attributes = vec!["data-testid".to_string(), "id".to_string()];
    settings.component_selector_attributes = BTreeMap::new();

    assert!(has_configured_html_id_selector(&settings));
}

#[test]
fn test_load_discovered_v2_without_playwright_settings() {
    let root = fixture_path(&["scan-config", "no-mistakes-v2-no-playwright"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    assert_eq!(settings.frontend_root, "app");
    assert_eq!(settings.selector_attributes, vec!["data-testid", "data-pw"]);
}

#[test]
fn test_default_frontend_root_with_app_dir() {
    let root = fixture_path(&["scan-config", "no-mistakes-v2-app-exists"]);
    let settings = load_settings(&root, None, &[], None).unwrap();
    // The default value assigned is "app", since the dir exists
    assert_eq!(settings.frontend_root, "app");
}

#[test]
fn test_has_configured_html_id_selector_true_components() {
    let mut settings = load_settings(
        &fixture_path(&["scan-config", "missing-default"]),
        None,
        &[],
        None,
    )
    .unwrap();
    settings.selector_attributes = vec!["data-testid".to_string()];
    settings.component_selector_attributes = BTreeMap::new();
    settings
        .component_selector_attributes
        .insert("Button".to_string(), "id".to_string());

    assert!(has_configured_html_id_selector(&settings));
}
