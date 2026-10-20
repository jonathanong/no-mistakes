use super::*;

#[test]
fn a_nested_directory_named_like_a_project_root_keeps_its_own_scope() {
    let root = fixture("wait-scopes-nested-root");
    let report = analyze_project(&root, None, &[]).unwrap();
    let waits = report
        .check
        .iter()
        .filter(|finding| finding.kind == "enqueue-and-wait")
        .map(|finding| finding.file.clone())
        .collect::<Vec<_>>();
    assert!(
        waits.iter().any(|file| file.ends_with("api/routes/r.mts")),
        "the api routes file stays in scope, waits: {waits:?}"
    );
    assert!(
        waits.iter().any(|file| file.ends_with("lib/api/util.mts")),
        "lib/api belongs to the root project, which has no waitScopes, waits: {waits:?}"
    );
    assert!(
        waits.iter().all(|file| !file.ends_with("api/other/o.mts")),
        "api/other is outside the api project's routes scope, waits: {waits:?}"
    );
}
