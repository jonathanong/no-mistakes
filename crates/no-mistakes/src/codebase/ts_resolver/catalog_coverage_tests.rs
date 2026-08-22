use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn fixture(name: &str) -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig")
            .join(name),
    )
}

fn catalog_for(root: &Path) -> TsConfigCatalog {
    let files = crate::codebase::ts_source::discover_visible_paths(root);
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
    TsConfigCatalog::from_visible(root, &[root.to_path_buf()], &visible)
}

#[test]
fn unresolved_package_extends_records_an_invalid_extends_diagnostic() {
    let root = fixture("unresolved-package-extends");
    let catalog = catalog_for(&root);
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind == TsConfigDiagnosticKind::InvalidExtends
            && diagnostic.detail.contains("@missing/tsconfig")
    }));
}

#[test]
fn relative_missing_extends_is_an_invalid_config() {
    let root = fixture("workspace-resolution/invalid-extends");
    let catalog = catalog_for(&root);
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            TsConfigDiagnosticKind::InvalidExtends | TsConfigDiagnosticKind::InvalidConfig
        )
    }));
}

#[test]
fn extends_cycle_is_reported_without_hanging() {
    let root = fixture("extends-cycle");
    let catalog = catalog_for(&root);
    assert!(catalog
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.detail.contains("cycle")));
}

#[test]
fn package_extends_reads_package_json_tsconfig_field_and_directory_fallback() {
    let field = fixture("package-extends-field");
    let field_catalog = catalog_for(&field);
    assert!(
        field_catalog.diagnostics().is_empty(),
        "{:#?}",
        field_catalog.diagnostics()
    );
    assert_eq!(
        field_catalog
            .provenance_for(&field.join("src/entry.ts"))
            .config,
        Some(field.join("tsconfig.json"))
    );

    let dir = fixture("package-extends-dir");
    let dir_catalog = catalog_for(&dir);
    assert!(
        dir_catalog.diagnostics().is_empty(),
        "{:#?}",
        dir_catalog.diagnostics()
    );

    let file = fixture("package-extends-file");
    let file_catalog = catalog_for(&file);
    assert!(
        file_catalog.diagnostics().is_empty(),
        "{:#?}",
        file_catalog.diagnostics()
    );
}

#[test]
fn directory_and_json_suffix_extends_resolve() {
    let directory = fixture("directory-extends");
    assert!(catalog_for(&directory).diagnostics().is_empty());
    let suffix = fixture("json-suffix-extends");
    assert!(catalog_for(&suffix).diagnostics().is_empty());
}

#[test]
fn source_store_read_failures_surface_as_invalid_config() {
    let root = fixture("unreadable-tsconfig");
    let tsconfig = root.join("tsconfig.json");
    let inventory = FileInventory::from_paths(&[tsconfig.clone()]);
    let sources = SourceStore::new(Arc::new(inventory));
    let catalog = TsConfigCatalog::from_visible_and_sources(
        &root,
        &[root.clone()],
        &[tsconfig.clone(), root.join("src/entry.ts")],
        &sources,
    );
    assert!(
        catalog.diagnostics().iter().any(|diagnostic| {
            diagnostic.detail.contains("reading")
                || diagnostic.kind == TsConfigDiagnosticKind::InvalidConfig
        }),
        "{:#?}",
        catalog.diagnostics()
    );
}

#[test]
fn invalid_package_tsconfig_records_an_extends_diagnostic_and_continues() {
    let root = fixture("package-extends-invalid");
    let catalog = catalog_for(&root);
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind == TsConfigDiagnosticKind::InvalidExtends
            || diagnostic.detail.contains("parsing")
            || diagnostic.detail.contains("loading extended")
    }));
}
