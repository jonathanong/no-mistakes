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
}
