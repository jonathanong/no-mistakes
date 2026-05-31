use serde_json::json;

use super::*;

#[test]
fn dependencies_json_preserves_hash_in_structured_entrypoint_file() {
    let options = json!({
        "root": fixture_root("symbol-export"),
        "includeSymbols": true,
        "files": [{ "file": "hash#entry.mts", "symbol": "run" }]
    })
    .to_string();

    let output = dependencies_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file["file"] == "source.mts" && file["symbol"] == "alpha"));
}
