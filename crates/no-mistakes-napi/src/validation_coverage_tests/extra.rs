use super::missing_config;
use serde_json::json;

#[test]
fn codebase_project_and_query_json_impls_reject_invalid_options() {
    let bad = json!({ "unknownField": true });
    let missing = missing_config();

    assert!(crate::symbols_json_impl(bad.clone()).is_err());
    assert!(crate::symbols_json_impl(json!({})).is_err());
    assert!(crate::symbols_json_impl(json!({ "files": ["a.ts"], "mode": "bad" })).is_err());
    assert!(crate::symbols_json_impl(json!({ "files": ["a.ts"], "kinds": ["bad"] })).is_err());
    assert!(crate::symbols_json_impl(json!({ "files": ["a.ts"], "include": "bad" })).is_err());

    assert!(crate::import_usages_json_impl(bad.clone()).is_err());
    assert!(crate::dependencies_json_impl(bad.clone()).is_err());
    assert!(crate::dependencies_json_impl(json!({})).is_err());
    assert!(crate::dependencies_json_impl(json!({
        "files": ["a.ts"],
        "relationships": ["bad-rel"]
    }))
    .is_err());
    assert!(crate::dependencies_json_impl(json!({
        "files": ["a.ts"],
        "projection": "invalid"
    }))
    .is_err());

    assert!(crate::data_pw_json_impl(bad.clone()).is_err());
    assert!(crate::data_pw_json_impl(json!({})).is_err());
    assert!(crate::data_pw_json_impl(json!({ "value": "btn", "include": "bad" })).is_err());
    assert!(crate::data_pw_json_impl(json!({ "value": "btn", "config": missing })).is_err());

    assert!(crate::effects_json_impl(bad.clone()).is_err());
    assert!(crate::effects_json_impl(json!({})).is_err());
    assert!(crate::effects_json_impl(json!({ "kind": "db" })).is_err());
    assert!(crate::effects_json_impl(json!({
        "kind": "db",
        "entry": "src/a.ts",
        "config": missing
    }))
    .is_err());

    assert!(crate::rsc_callers_json_impl(bad.clone()).is_err());
    assert!(crate::rsc_callers_json_impl(json!({})).is_err());
    assert!(crate::rsc_callers_json_impl(json!({
        "component": "src/a.tsx",
        "config": missing
    }))
    .is_err());

    assert!(crate::registry_extension_json_impl(bad.clone()).is_err());
    assert!(crate::registry_extension_json_impl(json!({})).is_err());
    assert!(crate::registry_extension_json_impl(json!({ "registryFile": missing })).is_err());

    assert!(crate::flow_json_impl(bad.clone()).is_err());
    assert!(crate::flow_json_impl(json!({})).is_err());
    assert!(crate::flow_json_impl(json!({
        "target": "src/a.ts",
        "relationships": ["bad-rel"]
    }))
    .is_err());
    assert!(crate::flow_json_impl(json!({
        "target": "src/a.ts",
        "direction": "sideways"
    }))
    .is_err());
    assert!(crate::flow_json_impl(json!({
        "target": "src/a.ts",
        "config": missing
    }))
    .is_err());

    assert!(crate::server_contracts_json_impl(bad.clone()).is_err());
    assert!(crate::react_analyze_json_impl(bad.clone()).is_err());
    assert!(crate::react_check_json_impl(bad.clone()).is_err());
    assert!(crate::react_usages_json_impl(bad.clone()).is_err());

    assert!(crate::queries::importers_json_impl(bad.clone()).is_err());
    assert!(crate::queries::importers_json_impl(json!({ "file": "   " })).is_err());
    assert!(crate::queries::exports_of_json_impl(bad.clone()).is_err());
    assert!(crate::queries::exports_of_json_impl(json!({ "file": "" })).is_err());
    assert!(crate::queries::dead_exports_json_impl(bad.clone()).is_err());
    assert!(crate::queries::dead_exports_json_impl(json!({ "file": "" })).is_err());
    assert!(crate::queries::call_sites_json_impl(bad.clone()).is_err());
    assert!(crate::queries::call_sites_json_impl(json!({ "file": "" })).is_err());
    assert!(crate::queries::call_sites_json_impl(json!({
        "file": "a.ts",
        "exportName": " "
    }))
    .is_err());
    assert!(crate::queries::resolve_check_json_impl(bad.clone()).is_err());
    assert!(crate::queries::resolve_check_json_impl(json!({})).is_err());
    assert!(crate::queries::resolve_check_json_impl(json!({ "file": "" })).is_err());
    assert!(crate::queries::resolve_check_json_impl(json!({ "files": [] })).is_err());
    assert!(crate::queries::resolve_check_json_impl(json!({ "files": [" "] })).is_err());
    assert!(crate::queries::resolve_check_json_impl(json!({
        "file": "a.ts",
        "files": ["b.ts"]
    }))
    .is_err());

    assert!(crate::postgres_lock_inventory_json_impl(bad.clone()).is_err());
    assert!(crate::postgres_lock_inventory_json_impl(json!({ "config": missing })).is_err());
    assert!(crate::generate_postgres_catalog_json_impl(bad.clone()).is_err());
    assert!(crate::generate_postgres_catalog_json_impl(json!({ "output": 123 })).is_err());
    assert!(crate::lockfile_diff_json_impl(bad).is_err());
    assert!(crate::lockfile_diff_json_impl(json!({})).is_err());
}
