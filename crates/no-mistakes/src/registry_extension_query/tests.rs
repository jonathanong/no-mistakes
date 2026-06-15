use super::*;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/registry-extension/fixture")
}

fn report(file: &str) -> RegistryExtensionReport {
    run(&fixture(), Path::new(file)).unwrap()
}

#[test]
fn detects_register_call_pattern() {
    let report = report("register-call.ts");
    assert_eq!(report.pattern_kind, "register-call");
    assert_eq!(report.registrant.as_deref(), Some("registry.register"));
    assert_eq!(report.confidence, "high");
    assert_eq!(report.entries.len(), 2);

    let first = &report.entries[0];
    let import = first.entry_import.as_ref().unwrap();
    assert_eq!(import.specifier, "./plugins/foo");
    assert_eq!(import.symbol.as_deref(), Some("FooPlugin"));
    assert!(first
        .call_shape
        .starts_with("registry.register(new FooPlugin"));
    assert_eq!(
        report.template.as_deref(),
        Some("registry.register(new <Entry>({ id: \"foo\" }))")
    );
}

#[test]
fn detects_container_array() {
    let report = report("container-array.ts");
    assert_eq!(report.pattern_kind, "container-array");
    assert!(report.registrant.is_none());
    assert_eq!(report.entries.len(), 2);
    assert_eq!(
        report.entries[0].entry_import.as_ref().unwrap().specifier,
        "./alpha"
    );
}

#[test]
fn detects_container_object() {
    let report = report("container-object.ts");
    assert_eq!(report.pattern_kind, "container-object");
    assert_eq!(report.entries.len(), 2);
    // Bare identifier values resolve to their imports.
    assert_eq!(
        report.entries[0]
            .entry_import
            .as_ref()
            .unwrap()
            .symbol
            .as_deref(),
        Some("Alpha")
    );
}

#[test]
fn detects_dynamic_import_registrants() {
    let report = report("dynamic-import.ts");
    assert_eq!(report.pattern_kind, "register-call");
    assert_eq!(report.entries.len(), 2);
    let import = report.entries[0].entry_import.as_ref().unwrap();
    assert_eq!(import.kind, "dynamic");
    assert_eq!(import.specifier, "./plugins/lazy-a");
    assert!(import.symbol.is_none());
}

#[test]
fn mixed_file_reports_dominant_and_notes_side_effect() {
    let report = report("mixed.ts");
    assert_eq!(report.pattern_kind, "register-call");
    assert!(report
        .notes
        .iter()
        .any(|note| note.contains("side-effect import")));
}

#[test]
fn no_pattern_reports_none() {
    let report = report("none.ts");
    assert_eq!(report.pattern_kind, "none");
    assert_eq!(report.confidence, "low");
    assert!(report.entries.is_empty());
    assert!(report.template.is_none());
    assert!(report.notes.iter().any(|note| note.contains("no repeated")));
}

#[test]
fn missing_file_errors() {
    let err = run(&fixture(), Path::new("does-not-exist.ts")).unwrap_err();
    assert!(err.to_string().contains("cannot read"));
}

#[test]
fn both_register_dominant_notes_container() {
    let report = report("both-register-dominant.ts");
    assert_eq!(report.pattern_kind, "register-call");
    assert_eq!(report.entries.len(), 3);
    assert!(report
        .notes
        .iter()
        .any(|note| note.contains("container literal with 2 entries")));
}

#[test]
fn both_container_dominant_notes_register() {
    let report = report("both-container-dominant.ts");
    assert_eq!(report.pattern_kind, "container-array");
    assert_eq!(report.entries.len(), 3);
    assert!(report
        .notes
        .iter()
        .any(|note| note.contains("register-call shape with 2 entries")));
}

#[test]
fn collects_default_and_namespace_imports() {
    let report = report("import-kinds.ts");
    assert_eq!(report.pattern_kind, "register-call");
    let import = report.entries[0].entry_import.as_ref().unwrap();
    assert_eq!(import.kind, "default");
    assert_eq!(import.symbol.as_deref(), Some("default"));
}

#[test]
fn container_object_skips_spread() {
    let report = report("container-spread.ts");
    assert_eq!(report.pattern_kind, "container-object");
    // Only the two named properties are entries; the spread is skipped.
    assert_eq!(report.entries.len(), 2);
}

#[test]
fn function_expression_dynamic_import() {
    let report = report("fn-expr-dynamic.ts");
    assert_eq!(report.pattern_kind, "register-call");
    let import = report.entries[0].entry_import.as_ref().unwrap();
    assert_eq!(import.kind, "dynamic");
    assert_eq!(import.specifier, "./plugins/fn-a");
}

#[test]
fn absolute_registry_path_is_accepted() {
    let abs = fixture().join("register-call.ts");
    let report = run(&fixture(), &abs).unwrap();
    assert_eq!(report.pattern_kind, "register-call");
}
