use super::*;
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-calls/explicit-playwright-project/fixture"),
    )
}

fn fixture_findings(yaml: &str) -> anyhow::Result<Vec<crate::codebase::rules::RuleFinding>> {
    let root = fixture_root();
    let config = tempfile::Builder::new().suffix(".yml").tempfile()?;
    std::fs::write(config.path(), yaml)?;
    run_check(&root, Some(config.path()), None)
}

const EXPLICIT_PROJECTS: &str =
    "tests:\n  playwright:\n    projects:\n      custom:\n        include: [src/custom.spec.ts]\n";

#[test]
fn named_playwright_roots_include_explicit_projects() {
    let findings = fixture_findings(&format!(
        "{EXPLICIT_PROJECTS}rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{{ playwright: [custom] }}]\n      targets: [{{ global: setTimeout }}]\n",
    ))
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/custom.spec.ts");
}

#[test]
fn all_playwright_roots_include_explicit_projects() {
    let findings = fixture_findings(&format!(
        "{EXPLICIT_PROJECTS}rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{{ playwright: true }}]\n      targets: [{{ global: setTimeout }}]\n",
    ))
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/custom.spec.ts");
}

#[test]
fn explicit_playwright_roots_still_reject_unknown_names() {
    let error = fixture_findings(&format!(
        "{EXPLICIT_PROJECTS}rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{{ playwright: [missing] }}]\n      targets: [{{ global: setTimeout }}]\n",
    ))
    .unwrap_err();

    assert!(
        error.to_string().contains("names an unknown project"),
        "{error:#}"
    );
}

#[test]
fn prepared_catalog_merges_explicit_projects_without_runner_configs() {
    let root = fixture_root();
    let visible = crate::codebase::ts_source::discover_files(&root, &[]);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &visible);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let tsconfig_catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig, None);
    let config: NoMistakesConfig = serde_yaml::from_str(
        "tests:\n  playwright:\n    configs: missing.config.ts\n    projects:\n      custom:\n        include: [src/custom.spec.ts]",
    )
    .unwrap();
    let catalog = crate::codebase::rules::prepare_playwright_project_catalog(
        &root,
        &config,
        &snapshot,
        &tsconfig_catalog,
    );

    assert!(catalog.config_projects().unwrap().is_empty());
    let matched = catalog
        .matching_files(&root, &["custom".to_string()], &visible)
        .unwrap();
    assert_eq!(
        matched,
        vec![root.join("src/custom.spec.ts")],
        "{matched:?}"
    );
}

#[test]
fn prepared_catalog_surfaces_missing_runner_config_errors() {
    let root = fixture_root();
    let visible = crate::codebase::ts_source::discover_files(&root, &[]);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &visible);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let tsconfig_catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig, None);
    let config: NoMistakesConfig =
        serde_yaml::from_str("tests:\n  playwright:\n    configs: missing.config.ts").unwrap();
    let catalog = crate::codebase::rules::prepare_playwright_project_catalog(
        &root,
        &config,
        &snapshot,
        &tsconfig_catalog,
    );

    let error = catalog.config_projects().unwrap_err();
    assert!(error.to_string().contains("does not exist"), "{error:#}");
}
