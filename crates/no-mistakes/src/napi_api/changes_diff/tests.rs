use super::*;
use serde_json::json;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/changes-diff")
}

#[test]
fn changes_diff_binding_shares_library_reports_and_requires_roots() {
    let root = root();
    let options = json!({"baseRoot": root.join("base"), "headRoot": root.join("head")});
    let result = changes_diff_json_impl(options).unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["complete"], true);
    assert_eq!(
        result["fixtures"]["changed"],
        json!(["profile.avatar", "profile.description"])
    );
    let result = changes_diff_json_impl(
        json!({"baseRoot": root.join("missing"), "headRoot": root.join("head")}),
    )
    .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&result).unwrap()["complete"],
        false
    );
    for (options, message) in [
        (
            json!({"headRoot": root.join("head")}),
            "baseRoot is required",
        ),
        (
            json!({"baseRoot": root.join("base")}),
            "headRoot is required",
        ),
        (json!({"unknown": true}), "unknown field"),
        (
            json!({"baseRoot": root.join("base"), "headRoot": root.join("head"), "config": "../reserved.yml"}),
            "invalid changes category",
        ),
    ] {
        let error = changes_diff_json_impl(options).unwrap_err();
        assert!(error.reason.contains(message), "{}", error.reason);
    }
}
