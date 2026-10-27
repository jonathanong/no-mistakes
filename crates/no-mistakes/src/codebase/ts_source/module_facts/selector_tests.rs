use super::*;
use std::path::PathBuf;

#[test]
fn dynamic_import_selectors_record_only_proven_forms() {
    let result = analyze_typescript_modules(&TypeScriptModulesOptions {
        root: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/typescript-module-facts"),
        ),
        files: vec![PathBuf::from("dynamic-selectors.ts")],
    })
    .unwrap();
    let facts = &result.modules[0].facts;
    assert!(
        facts.diagnostics.is_empty(),
        "fixture must parse cleanly: {:?}",
        facts.diagnostics
    );
    let mut proven = facts.dynamic_import_selectors.clone();
    proven.sort_by_key(|(start, _)| *start);
    let names: Vec<_> = proven.into_iter().map(|(_, names)| names).collect();
    let expected: Vec<Vec<&str>> = vec![
        vec!["A"],
        vec!["A"],
        vec!["B"],
        vec!["C", "D"],
        vec!["default"],
        vec!["E", "F"],
        vec!["G"],
    ];
    assert_eq!(names, expected);
    // Every `import("./a")` is still a recorded load; unproven ones stay unknown.
    assert!(facts.loads.len() > expected.len() * 2);
}
