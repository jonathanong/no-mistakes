use crate::napi_api::*;
use serde_json::{json, Value};
use std::path::PathBuf;

type Api = fn(Value) -> napi::Result<String>;

#[test]
fn graph_api_entrypoints_preserve_explicit_tsconfig_parse_failures() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/invalid-json");
    let apis: &[(&str, Api)] = &[
        ("queues", queues_json_impl),
        ("queueEdges", queue_edges_json_impl),
        ("queueRelated", queue_related_json_impl),
        ("queueCheck", queue_check_json_impl),
        ("serverRoutes", server_routes_json_impl),
        ("serverRouteList", server_route_list_json_impl),
        ("serverRouteEdges", server_route_edges_json_impl),
        ("serverRouteRelated", server_route_related_json_impl),
    ];
    for &(name, api) in apis {
        let error = api(json!({
            "root": root,
            "tsconfig": "tsconfig.json",
            "files": ["src/entry.ts"],
            "direction": "deps"
        }))
        .expect_err(name);
        assert!(error.reason.contains("tsconfig.json"), "{name}: {error}");
        assert!(
            error.reason.to_lowercase().contains("pars"),
            "{name}: {error}"
        );
    }
}

#[test]
fn project_api_entrypoints_preserve_config_parse_diagnostics() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/napi-api/invalid-project-config");
    let apis: &[(&str, Api)] = &[
        ("reactAnalyze", react_analyze_json_impl),
        ("reactCheck", react_check_json_impl),
        ("reactUsages", react_usages_json_impl),
        ("fetches", fetches_json_impl),
        ("ciImpact", ci_impact_json_impl),
        ("ciEnv", ci_env_json_impl),
        ("ciTopology", ci_topology_json_impl),
    ];
    for &(name, api) in apis {
        let mut options = json!({
            "root": root,
            "config": ".no-mistakes.yml"
        });
        match name {
            "reactUsages" => options["target"] = json!("Button"),
            "ciImpact" => options["files"] = json!(["entry.mts"]),
            "ciEnv" => options["var"] = json!("CI"),
            _ => {}
        }
        let error = api(options).expect_err(name);
        assert!(error.reason.contains(".no-mistakes.yml"), "{name}: {error}");
        assert!(
            error.reason.to_lowercase().contains("pars"),
            "{name}: {error}"
        );
    }
}
