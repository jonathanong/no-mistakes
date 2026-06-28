use super::*;

fn import_usages_fixture_root() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-usages/fixture")
        .display()
        .to_string()
}

#[test]
fn import_usages_json_impl_reports_direct_imports() {
    let options = serde_json::json!({
        "root": import_usages_fixture_root(),
        "files": ["src/main.mts"]
    });

    let json = import_usages_json_impl(options.to_string()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["files"][0]["path"], "src/main.mts");
    assert!(value["files"][0]["imports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|import| import["specifier"] == "react"));
}
