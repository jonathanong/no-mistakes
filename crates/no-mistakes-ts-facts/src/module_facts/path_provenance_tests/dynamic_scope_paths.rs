use super::*;

#[test]
fn dynamic_scope_forgets_an_imported_call_path() {
    let (dynamic, path) = imported_call_under_dynamic_scope(
        "dynamic-scope-import.mjs",
        "readFileSync(\"reports/a.txt\")",
    );
    assert!(dynamic);
    assert_eq!(path, Some(PathOrigin::unknown()));
}
