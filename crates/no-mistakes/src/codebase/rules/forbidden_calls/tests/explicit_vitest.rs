use super::*;
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-calls/explicit-vitest-project/fixture"),
    )
}

fn fixture_findings(yaml: &str) -> anyhow::Result<Vec<crate::codebase::rules::RuleFinding>> {
    let root = fixture_root();
    let config = tempfile::Builder::new().suffix(".yml").tempfile()?;
    std::fs::write(config.path(), yaml)?;
    run_check(&root, Some(config.path()), None)
}

const EXPLICIT_PROJECTS: &str =
    "tests:\n  vitest:\n    projects:\n      custom:\n        include: [src/custom.spec.mts]\n";

#[test]
fn named_vitest_roots_include_explicit_projects() {
    let findings = fixture_findings(&format!(
        "{EXPLICIT_PROJECTS}rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{{ vitest: [custom] }}]\n      targets: [{{ global: setTimeout }}]\n",
    ))
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/custom.spec.mts");
}

#[test]
fn all_vitest_roots_include_explicit_projects() {
    let findings = fixture_findings(&format!(
        "{EXPLICIT_PROJECTS}rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{{ vitest: true }}]\n      targets: [{{ global: setTimeout }}]\n",
    ))
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/custom.spec.mts");
}

#[test]
fn explicit_vitest_roots_still_reject_unknown_names() {
    let error = fixture_findings(&format!(
        "{EXPLICIT_PROJECTS}rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{{ vitest: [missing] }}]\n      targets: [{{ global: setTimeout }}]\n",
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
        "tests:\n  vitest:\n    configs: missing.config.ts\n    projects:\n      custom:\n        include: [src/custom.spec.mts]",
    )
    .unwrap();
    let catalog = crate::codebase::rules::prepare_vitest_project_catalog(
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
        vec![root.join("src/custom.spec.mts")],
        "{matched:?}"
    );
}

#[test]
fn vitest_root_errors_when_no_files_match() {
    // Skip runner discovery with a missing config, then use an include that
    // matches nothing so the empty-collection diagnostic stays distinct from I/O.
    let error = fixture_findings(
        "tests:\n  vitest:\n    configs: missing.config.ts\n    projects:\n      empty:\n        include: [does-not-exist/**/*.ts]\nrules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ vitest: true }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap_err();

    assert!(
        error.to_string().contains("Vitest root matched no files"),
        "{error:#}"
    );
}
