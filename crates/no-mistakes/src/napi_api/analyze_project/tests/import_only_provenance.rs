use super::*;
use serde_json::{json, Value};
use std::path::PathBuf;

fn provenance_cache_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/provenance-cache"),
    )
}

fn provenance_config(value: &Value, index: usize) -> String {
    value["reports"][index]["result"]["tsconfig_provenance"][0]["config"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn overlapping_import_only_reports_reuse_cached_tsconfig_and_jsconfig_paths() {
    let root = provenance_cache_root();
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "id": "root",
                    "type": "dependencies",
                    "files": ["src/root.ts"],
                    "relationships": ["import-static"]
                },
                {
                    "id": "root-again",
                    "type": "dependencies",
                    "files": ["src/root.ts"],
                    "relationships": ["import-static"]
                },
                {
                    "id": "leaf",
                    "type": "dependencies",
                    "files": ["pkg/leaf.ts"],
                    "relationships": ["import-static"]
                }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let root_config = provenance_config(&value, 0);
    let root_again = provenance_config(&value, 1);
    let leaf_config = provenance_config(&value, 2);
    assert_eq!(root_config, "tsconfig.json", "{value:#?}");
    assert_eq!(root_again, root_config, "{value:#?}");
    assert_eq!(leaf_config, "tsconfig.json", "{value:#?}");
}
