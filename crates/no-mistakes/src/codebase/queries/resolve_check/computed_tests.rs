use super::*;
use std::path::PathBuf;

fn kinds_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries-kinds/fixture"),
    )
}

fn computed_report() -> ResolveCheckReport {
    compute(&ResolveCheckArgs {
        files: vec![PathBuf::from("computed.ts")],
        root: Some(kinds_root()),
        tsconfig: None,
        format: None,
        json: false,
    })
    .unwrap()
}

#[test]
fn computed_specifiers_are_unresolved_and_literal_dynamic_stays_resolved() {
    let report = computed_report();
    assert!(!report.all_resolve);

    let computed: Vec<_> = report.imports.iter().filter(|row| row.computed).collect();
    assert!(computed
        .iter()
        .all(|row| row.status == Status::Unresolved && row.resolved.is_none()));
    assert!(computed
        .iter()
        .any(|row| row.specifier == "./${}" && row.kind == "dynamic"));
    assert!(computed
        .iter()
        .any(|row| row.specifier == "moduleName" && row.kind == "dynamic"));
    assert!(computed
        .iter()
        .any(|row| row.specifier == "moduleName" && row.kind == "require"));
    assert!(computed
        .iter()
        .any(|row| row.specifier == "moduleName" && row.kind == "require-resolve"));
    assert!(computed
        .iter()
        .any(|row| row.specifier == "<computed>" && row.kind == "dynamic"));

    let literal = report
        .imports
        .iter()
        .find(|row| row.specifier == "./dep" && row.kind == "dynamic")
        .expect("literal next/dynamic import('./dep')");
    assert!(matches!(literal.status, Status::Resolved));
    assert!(!literal.computed);
    assert_eq!(literal.resolved.as_deref(), Some("dep.ts"));

    let static_require = report
        .imports
        .iter()
        .find(|row| row.specifier == "./dep" && row.kind == "require")
        .expect("expression-free require(`./dep`)");
    assert!(matches!(static_require.status, Status::Resolved));
    assert!(!static_require.computed);
    assert_eq!(static_require.resolved.as_deref(), Some("dep.ts"));
}

#[test]
fn identifier_computed_import_is_unresolved_not_external() {
    let ident = computed_report()
        .imports
        .into_iter()
        .find(|row| row.specifier == "moduleName" && row.kind == "dynamic")
        .expect("import(moduleName)");
    assert!(matches!(ident.status, Status::Unresolved));
}
