// N-API bindings for the `ci` and `impacted-checks` commands. Included by
// napi_api.rs so the `#[napi]` registrations live in the crate-root module.

json_binding!(ci_impact_json, "ciImpactJson", ci_impact_json_impl);
json_binding!(ci_env_json, "ciEnvJson", ci_env_json_impl);
json_binding!(ci_topology_json, "ciTopologyJson", ci_topology_json_impl);
json_binding!(ci_topology_impact_json, "ciTopologyImpactJson", ci_topology_impact_json_impl);
json_binding!(
    impacted_checks_json,
    "impactedChecksJson",
    impacted_checks_json_impl
);
