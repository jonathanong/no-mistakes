use super::*;
#[test]
fn native_json_projects_the_opt_in_assertion_owner_relationship() {
    let options = serde_json::json!({
        "root": fixture(), "files": ["src/producers.mts#crossProcess"],
        "relationships": ["assertion-owner"], "includeSymbols": true, "projection": "graph"
    });
    let output: serde_json::Value =
        serde_json::from_str(&crate::napi_api::dependencies_json_impl(options).unwrap()).unwrap();
    assert_eq!(
        output["roots"],
        serde_json::json!(["src/producers.mts#crossProcess"])
    );
    assert!(
        output["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["file"] == "src/assertions.mts"
                && node["symbol"] == "assertResult"
                && node["depth"] == 1
                && node["via"] == serde_json::json!(["assertion-owner"])),
        "{output}"
    );
}
