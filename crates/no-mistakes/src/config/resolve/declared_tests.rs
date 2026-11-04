use serde_json::json;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

fn resolve(options: serde_json::Value) -> serde_json::Value {
    let output = crate::napi_api::resolve_config_json_impl(options).unwrap();
    serde_json::from_str(&output).unwrap()
}

#[test]
fn declared_native_view_matches_the_library_without_materialized_defaults() {
    let root = fixture("config/declared-view");
    let value = resolve(json!({"root": root, "view": "declared"}));
    assert_eq!(
        value,
        crate::config::v2::declared_config(&root, None).unwrap()
    );
    assert_eq!(value.as_object().unwrap().len(), 1);
    let rules = value["rules"].as_array().unwrap();
    assert_eq!(
        rules
            .iter()
            .map(|rule| rule["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["root-policy", "first-policy", "last-policy"]
    );
    assert!(rules.iter().all(|rule| rule.get("enabled").is_none()));
}

#[test]
fn declared_explicit_config_selects_one_file_and_missing_config_is_empty() {
    let root = fixture("config/declared-view");
    let value = resolve(json!({"root": root, "config": ".no-mistakes.yml", "view": "declared"}));
    assert_eq!(value["rules"].as_array().unwrap().len(), 1);
    assert_eq!(
        resolve(json!({"root": fixture("config/declared-empty"), "view": "declared"})),
        json!({})
    );
}

#[test]
fn default_native_view_keeps_the_resolved_contract_and_rejects_unknown_views() {
    let root = fixture("config/declared-view");
    assert_eq!(
        resolve(json!({"root": root})),
        resolve(json!({"root": root, "view": "resolved"}))
    );
    assert!(
        crate::napi_api::resolve_config_json_impl(json!({"root": root, "view": "unknown"}))
            .is_err()
    );
    assert!(crate::napi_api::resolve_config_json_impl(
        json!({"root": root, "view": "declared", "unknown": true})
    )
    .is_err());
}
