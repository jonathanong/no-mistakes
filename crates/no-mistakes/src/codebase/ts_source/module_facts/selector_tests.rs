use super::*;
use std::path::PathBuf;

fn facts(file: &str) -> TypeScriptModuleFacts {
    let result = analyze_typescript_modules(&TypeScriptModulesOptions {
        root: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/typescript-module-facts"),
        ),
        files: vec![PathBuf::from(file)],
    })
    .unwrap();
    let facts = result.modules.into_iter().next().unwrap().facts;
    assert!(
        facts.diagnostics.is_empty(),
        "fixture must parse cleanly: {:?}",
        facts.diagnostics
    );
    facts
}

fn selectors(file: &str) -> Vec<Vec<String>> {
    let mut proven = facts(file).dynamic_import_selectors;
    proven.sort_by_key(|(start, _, _)| *start);
    proven.into_iter().map(|(_, names, _)| names).collect()
}

#[test]
fn loader_default_in_jsx_is_proven_and_mixed_use_is_not() {
    // Jsx, the attribute and child loaders, `.then` (Name), then the bare loader.
    assert_eq!(
        selectors("dynamic-selectors-jsx.tsx"),
        [["default"], ["default"], ["default"], ["Name"], ["default"]]
    );
}

#[test]
fn returned_import_selectors_follow_one_local_return() {
    assert_eq!(
        selectors("dynamic-selectors-returned.ts"),
        [vec!["default"], vec!["A", "B"], vec!["C", "D"]]
    );
}

#[test]
fn recovered_static_return_does_not_prove_a_selector() {
    let result = analyze_typescript_modules(&TypeScriptModulesOptions {
        root: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/typescript-module-facts"),
        ),
        files: vec![PathBuf::from("dynamic-selectors-returned-static.ts")],
    })
    .unwrap();
    let facts = &result.modules[0].facts;
    assert!(
        !facts.diagnostics.is_empty(),
        "a return in a class static block is a diagnosed recovery"
    );
    assert!(facts.dynamic_import_selectors.is_empty());
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
        vec!["default"],
        vec!["E", "F"],
        vec!["G"],
        vec!["A"],
        vec!["a"],
        vec!["A"],
    ];
    assert_eq!(names, expected);
}

#[test]
fn discarded_import_results_are_isolated_from_export_policy_selectors() {
    let facts = facts("dynamic-selectors-unused.mts");
    let discarded: Vec<_> = facts
        .loads
        .iter()
        .filter(|load| facts.discarded_dynamic_imports.contains(&load.span.start))
        .map(|load| load.specifier.as_str())
        .collect();
    assert_eq!(
        discarded,
        [
            "./bare",
            "./awaited",
            "./void",
            "./unused",
            "./typed",
            "./promise"
        ]
    );
    assert!(facts.dynamic_import_selectors.is_empty());
    let public = serde_json::to_value(&facts).unwrap();
    let mut without_discarded = facts.clone();
    without_discarded.discarded_dynamic_imports.clear();
    assert_eq!(public, serde_json::to_value(without_discarded).unwrap());
}
