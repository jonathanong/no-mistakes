use super::*;

fn import_usages_fixture_root() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-usages/fixture")
        .display()
        .to_string()
}

fn gitignore_tsconfig_fixture() -> tempfile::TempDir {
    let fixture = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    fixture
}

fn dependency_rows(root: &std::path::Path, tsconfig: Option<&str>) -> Vec<serde_json::Value> {
    let mut options = serde_json::json!({
        "root": root,
        "files": ["entry.ts"],
        "relationships": ["import"]
    });
    if let Some(tsconfig) = tsconfig {
        options["tsconfig"] = serde_json::Value::String(tsconfig.to_string());
    }
    let output = dependencies_json_impl(options.to_string()).unwrap();
    serde_json::from_str::<serde_json::Value>(&output).unwrap()["files"]
        .as_array()
        .unwrap()
        .clone()
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

#[test]
fn dependencies_napi_ignores_automatic_tsconfig_but_honors_explicit_ignored_config() {
    let fixture = gitignore_tsconfig_fixture();

    let automatic = dependency_rows(fixture.path(), None);
    assert!(automatic
        .iter()
        .any(|row| row["module"] == "@lib/forbidden"));
    assert!(!automatic
        .iter()
        .any(|row| row["path"] == "src/forbidden.ts"));

    let explicit = dependency_rows(fixture.path(), Some("tsconfig.json"));
    assert!(explicit.iter().any(|row| row["path"] == "src/forbidden.ts"));
    assert!(!explicit.iter().any(|row| row["module"] == "@lib/forbidden"));
}

#[test]
fn dependencies_napi_honors_explicit_ignored_root_but_not_ignored_transitives() {
    let fixture = gitignore_tsconfig_fixture();
    for relationships in [None, Some(serde_json::json!(["import"]))] {
        let mut options = serde_json::json!({
            "root": fixture.path(),
            "files": ["ignored-explicit/effect-entry.ts"]
        });
        if let Some(relationships) = relationships {
            options["relationships"] = relationships;
        }
        let output = dependencies_json_impl(options.to_string()).unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        let paths = value["files"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| row["path"].as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"src/effect.ts"), "{value:#?}");
        assert!(
            !paths.contains(&"ignored-transitive/effect.ts"),
            "{value:#?}"
        );
    }
}

#[test]
fn queues_napi_ignores_automatic_tsconfig_but_honors_explicit_ignored_config() {
    let fixture = gitignore_tsconfig_fixture();
    let root = fixture.path().display().to_string();

    let automatic =
        crate::napi_api::queues_json_impl(serde_json::json!({ "root": root }).to_string()).unwrap();
    let automatic: serde_json::Value = serde_json::from_str(&automatic).unwrap();
    assert!(automatic["producers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|producer| { producer["file"] == "enqueue.ts" && producer["queueFile"].is_null() }));

    let explicit = crate::napi_api::queues_json_impl(
        serde_json::json!({ "root": root, "tsconfig": "tsconfig.json" }).to_string(),
    )
    .unwrap();
    let explicit: serde_json::Value = serde_json::from_str(&explicit).unwrap();
    assert!(explicit["producers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|producer| {
            producer["file"] == "enqueue.ts" && producer["queueFile"] == "src/queues/emails.ts"
        }));
}
