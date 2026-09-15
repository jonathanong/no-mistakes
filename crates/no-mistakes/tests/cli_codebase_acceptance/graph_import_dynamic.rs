use super::common::{file_paths, fixture, run_json};

#[test]
fn import_dynamic_follows_if_switch_nested_function_and_jsx_handler() {
    let root = fixture("import-forms");
    let value = run_json(
        &root,
        &[
            "dependencies",
            "--relationship",
            "import-dynamic",
            "dynamic-all-shapes.tsx",
        ],
    );
    let mut paths = file_paths(&value);
    paths.sort();
    assert_eq!(
        paths,
        vec![
            "dynamic-click-target.mts",
            "dynamic-if-target.mts",
            "dynamic-nested-target.mts",
            "dynamic-switch-target.mts",
        ],
        "{value:#?}"
    );
}
