use super::*;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig")
            .join(name),
    )
}

fn catalog_for(root: &Path) -> TsConfigCatalog {
    let root = root.to_path_buf();
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let visible: Vec<PathBuf> = files
        .into_iter()
        .map(|path| {
            let absolute = if path.is_absolute() {
                path
            } else {
                root.join(path)
            };
            normalize_path(&absolute)
        })
        .collect();
    TsConfigCatalog::from_visible(&root, std::slice::from_ref(&root), &visible)
}

#[test]
fn invalid_json_tsconfig_is_an_invalid_config() {
    let catalog = catalog_for(&fixture("invalid-json"));
    assert!(
        catalog
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.detail.contains("parsing")),
        "{:#?}",
        catalog.diagnostics()
    );
}

#[test]
fn empty_json_tsconfig_loads_as_null_object() {
    let root = fixture("empty-json");
    let catalog = catalog_for(&root);
    assert_eq!(
        catalog.provenance_for(&root.join("src/entry.ts")).config,
        Some(root.join("tsconfig.json"))
    );
}

#[test]
fn missing_project_reference_is_diagnosed() {
    let catalog = catalog_for(&fixture("missing-reference"));
    assert!(
        catalog.diagnostics().iter().any(|diagnostic| {
            diagnostic.kind == TsConfigDiagnosticKind::InvalidReference
                && diagnostic.detail.contains("does not exist")
        }),
        "{:#?}",
        catalog.diagnostics()
    );
}

#[test]
fn reference_outside_the_analysis_root_is_diagnosed() {
    let catalog = catalog_for(&fixture("outside-reference"));
    assert!(
        catalog.diagnostics().iter().any(|diagnostic| {
            diagnostic.kind == TsConfigDiagnosticKind::InvalidReference
                && diagnostic
                    .detail
                    .contains("outside configured analysis roots")
        }),
        "{:#?}",
        catalog.diagnostics()
    );
}

#[test]
fn catalog_builder_accepts_json_candidate_roots_and_missing_files() {
    let root = fixture("directory-extends");
    let config = root.join("tsconfig.json");
    let builder = CatalogBuilder::new(
        &root,
        std::slice::from_ref(&config),
        std::slice::from_ref(&config),
        None,
        None,
    );
    assert_eq!(builder.candidates(), vec![normalize_path(&config)]);

    let missing = root.join("missing-tsconfig.json");
    let catalog = CatalogBuilder::new(
        &root,
        std::slice::from_ref(&missing),
        std::slice::from_ref(&missing),
        None,
        None,
    )
    .build();
    assert!(
        catalog.diagnostics().iter().any(|diagnostic| {
            diagnostic.detail.contains("does not exist")
                || diagnostic.kind == TsConfigDiagnosticKind::InvalidConfig
        }),
        "{:#?}",
        catalog.diagnostics()
    );
}

#[test]
fn apply_own_expands_config_dir_files_and_empty_lists() {
    let root = fixture("directory-extends");
    let path = root.join("tsconfig.json");
    let mut config = EffectiveConfig::new(path.clone(), root.clone());
    config
        .apply_own(
            &serde_json::json!({
                "compilerOptions": {
                    "paths": { "@lib/*": ["${configDir}/src/*"] },
                    "baseUrl": "${configDir}"
                },
                "files": ["${configDir}/src/entry.ts"],
                "include": [],
                "exclude": [],
                "references": []
            }),
            &path,
            &root,
            |value| Ok(root.join(value)),
        )
        .expect("configDir files and empty lists should apply");
    assert!(config.files.is_some());
    assert_eq!(config.includes.as_ref().map(Vec::len), Some(0));
    assert_eq!(config.excludes.as_ref().map(Vec::len), Some(0));

    for (value, needle) in [
        (
            serde_json::json!({ "compilerOptions": { "paths": [] } }),
            "paths",
        ),
        (
            serde_json::json!({ "compilerOptions": { "baseUrl": false } }),
            "baseUrl",
        ),
        (
            serde_json::json!({ "compilerOptions": { "outDir": false } }),
            "outDir",
        ),
        (
            serde_json::json!({ "compilerOptions": { "moduleResolution": false } }),
            "moduleResolution",
        ),
        (serde_json::json!({ "files": "src/entry.ts" }), "files"),
        (serde_json::json!({ "include": "src" }), "include"),
        (serde_json::json!({ "exclude": "src" }), "exclude"),
        (serde_json::json!({ "references": "lib" }), "references"),
    ] {
        let err = config
            .apply_own(&value, &path, &root, |value| Ok(root.join(value)))
            .expect_err(needle);
        assert!(err.contains(needle), "{err}");
    }
    let err = config
        .apply_own(
            &serde_json::json!({ "references": [{ "path": "./lib" }] }),
            &path,
            &root,
            |_| Err("missing reference".to_string()),
        )
        .expect_err("resolve_reference failure");
    assert!(err.contains("missing reference"));
}
