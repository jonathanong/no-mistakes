use super::super::*;
use serde_json::json;

#[test]
fn project_json_helpers_report_invalid_options_and_optional_file_filters() {
    let cases = [
        queues_json_impl(json!({"unknownField": true})),
        queue_edges_json_impl(json!({"unknownField": true})),
        queue_check_json_impl(json!({"unknownField": true})),
        server_routes_json_impl(json!({"unknownField": true})),
        server_route_edges_json_impl(json!({"unknownField": true})),
        react_analyze_json_impl(json!({"unknownField": true})),
        react_check_json_impl(json!({"unknownField": true})),
        react_usages_json_impl(json!({"root": "/no-mistakes-missing-coverage-root"})),
        react_usages_json_impl(json!({
            "root": "/no-mistakes-missing-coverage-root",
            "target": "Button",
            "include": "bogus"
        })),
        queue_related_json_impl(json!({
            "root": "/no-mistakes-missing-coverage-root",
            "files": ["src/a.ts"],
            "direction": "bogus"
        })),
        server_route_related_json_impl(json!({
            "root": "/no-mistakes-missing-coverage-root",
            "files": ["src/a.ts"],
            "direction": "bogus"
        })),
    ];
    for (index, result) in cases.into_iter().enumerate() {
        assert!(result.is_err(), "{index}: {result:?}");
    }

    assert!(server_route_list_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "files": []
    }))
    .is_ok());
    assert!(server_route_list_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "files": ["src/missing.ts"]
    }))
    .is_ok());
    assert!(queues_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "tsconfig": "tsconfig.json"
    }))
    .is_err());
    assert!(queue_related_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "files": []
    }))
    .is_err());
    assert!(server_route_related_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "files": []
    }))
    .is_err());
    assert!(effects_json_impl(json!({ "root": "/no-mistakes-missing-coverage-root" })).is_err());
    assert!(effects_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "kind": "fetch"
    }))
    .is_err());
    assert!(
        rsc_callers_json_impl(json!({ "root": "/no-mistakes-missing-coverage-root" })).is_err()
    );
    assert!(data_pw_json_impl(json!({ "root": "/no-mistakes-missing-coverage-root" })).is_err());
    assert!(registry_extension_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root"
    }))
    .is_err());
    assert!(tests_targets_json_impl(json!({
        "framework": "vitest",
        "files": []
    }))
    .is_err());
    assert!(tests_targets_json_impl(json!({ "unknownField": true })).is_err());
    assert!(fetches_json_impl(json!({ "unknownField": true })).is_err());
    assert!(check_json_impl(json!({ "unknownField": true })).is_err());
    assert!(ci_env_json_impl(json!({ "root": "." })).is_err());
    assert!(ci_topology_impact_json_impl(json!({ "root": "." })).is_err());
    assert!(ci_topology_impact_json_impl(json!({
        "root": ".",
        "base": "main"
    }))
    .is_err());
    assert!(ci_topology_impact_json_impl(json!({
        "root": ".",
        "base": "main",
        "head": "HEAD"
    }))
    .is_err());
    assert!(queue_edges_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "tsconfig": "tsconfig.json"
    }))
    .is_err());
    let _ = server_route_related_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "files": ["src/a.ts"],
        "direction": "deps"
    }));
    assert!(react_analyze_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "config": "no-mistakes.json"
    }))
    .is_err());
    assert!(flow_json_impl(json!({ "root": "/no-mistakes-missing-coverage-root" })).is_err());
    assert!(flow_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "target": "a.ts",
        "direction": "sideways"
    }))
    .is_err());
    assert!(server_contracts_json_impl(json!({ "unknownField": true })).is_err());
    assert!(tests_plan_json_impl(json!({ "unknownField": true })).is_err());
    assert!(tests_impact_json_impl(json!({ "unknownField": true })).is_err());
    assert!(ci_impact_json_impl(json!({ "unknownField": true })).is_err());
    assert!(ci_topology_json_impl(json!({ "unknownField": true })).is_err());
    assert!(impacted_checks_json_impl(json!({ "unknownField": true })).is_err());
    assert!(resolve_config_json_impl(json!({ "unknownField": true })).is_err());
    assert!(data_pw_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "value": "submit",
        "include": "bogus"
    }))
    .is_err());
    assert!(effects_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "kind": "fetch",
        "entry": "src/a.ts"
    }))
    .is_err());
    assert!(rsc_callers_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "component": "src/Button.tsx"
    }))
    .is_err());
    let _ = impacted_checks_json_impl(json!({
        "root": "/no-mistakes-missing-coverage-root",
        "timings": true
    }));
}
