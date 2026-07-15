use super::*;
use serde_json::{json, Value};
use std::path::PathBuf;

fn fixture(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for part in parts {
        path.push(part);
    }
    crate::codebase::ts_resolver::normalize_path(&path)
}

fn parse_json(value: String) -> Value {
    serde_json::from_str(&value).unwrap()
}

#[test]
fn prepared_check_matches_standalone_tracked_only_banned_paths() {
    let source = fixture(&["fixtures", "gitignore", "banned-paths-tracked-only"]);
    let directory = crate::test_support::materialize_saved_fixture(&source);
    let root = directory.path();
    crate::test_support::git_init(root);
    crate::test_support::git_add_force(root, &[".no-mistakes.yml", "tracked.patch"]);
    std::fs::rename(
        root.join("gitignore-after.fixture"),
        root.join(".gitignore"),
    )
    .unwrap();
    crate::test_support::git_add_force(root, &[".gitignore"]);

    let standalone =
        parse_json(crate::napi_api::check_json_impl(json!({ "root": root }).to_string()).unwrap());
    let aggregate = parse_json(
        analyze_project_json_impl(
            json!({
                "root": root,
                "reports": [{ "type": "check" }]
            })
            .to_string(),
        )
        .unwrap(),
    );

    assert_eq!(aggregate["reports"][0]["result"], standalone);
    let files = standalone["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "banned-paths")
        .map(|finding| finding["file"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(files, ["tracked.patch"]);
}
