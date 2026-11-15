use std::path::PathBuf;

#[test]
fn lossy_package_json_still_yields_version() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/forbidden-workspace-closure/invalid-utf8/package.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path.clone()]);
    let version = super::workspace_package_version_with_sources(path.parent().unwrap(), &sources);
    assert_eq!(version.as_deref(), Some("1.2.3"));
}
