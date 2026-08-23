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
