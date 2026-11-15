use std::path::PathBuf;

#[test]
fn lossy_package_json_still_finds_workspaces_line() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/package-json-nested-workspace-coverage/invalid-utf8/package.json",
    );
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    assert_eq!(super::line(&path, &sources), 4);
}
