use super::*;
use std::path::PathBuf;

fn selectors(file: &str) -> Vec<Vec<String>> {
    let result = analyze_typescript_modules(&TypeScriptModulesOptions {
        root: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/typescript-module-facts"),
        ),
        files: vec![PathBuf::from(file)],
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
    proven.into_iter().map(|(_, names)| names).collect()
}

#[test]
fn loader_default_in_jsx_is_proven_and_mixed_use_is_not() {
    assert_eq!(selectors("dynamic-selectors-jsx.tsx"), [["default"]]);
}

#[test]
fn dynamic_import_selectors_record_only_proven_forms() {
    let names = selectors("dynamic-selectors.ts");
    let expected: Vec<Vec<&str>> = vec![
        vec!["A"],
        vec!["A"],
        vec!["B"],
        vec!["C", "D"],
        vec!["default"],
        vec!["default"],
        vec!["default"],
        vec!["default"],
        vec!["default"],
        vec!["default"],
        vec!["default"],
        vec!["E", "F"],
        vec!["G"],
    ];
    assert_eq!(names, expected);
}
