use serde_json::json;

mod extra;
mod rlib;

pub(super) fn missing_config() -> &'static str {
    "/nonexistent-no-mistakes-validation-root/missing.yml"
}

#[test]
fn infra_and_swift_json_impls_reject_invalid_and_missing_options() {
    let bad = json!({ "unknownField": true });
    assert!(crate::infra_swift::infra_resource_refs_json_impl(bad.clone()).is_err());
    assert!(crate::infra_swift::infra_outputs_json_impl(bad.clone()).is_err());
    assert!(crate::infra_swift::infra_test_for_json_impl(bad.clone()).is_err());
    assert!(crate::infra_swift::swift_importers_json_impl(bad.clone()).is_err());
    assert!(crate::infra_swift::swift_test_targets_json_impl(bad).is_err());

    assert!(crate::infra_swift::infra_resource_refs_json_impl(json!({})).is_err());
    assert!(crate::infra_swift::infra_outputs_json_impl(json!({})).is_err());
    assert!(crate::infra_swift::infra_test_for_json_impl(json!({})).is_err());
    assert!(crate::infra_swift::swift_importers_json_impl(json!({})).is_err());
    assert!(crate::infra_swift::swift_test_targets_json_impl(json!({})).is_err());

    let missing = missing_config();
    assert!(crate::infra_swift::infra_resource_refs_json_impl(json!({
        "config": missing,
        "address": "aws_s3_bucket.logs"
    }))
    .is_err());
    assert!(crate::infra_swift::infra_outputs_json_impl(json!({
        "config": missing,
        "moduleDir": "modules/app"
    }))
    .is_err());
    assert!(crate::infra_swift::infra_test_for_json_impl(json!({
        "config": missing,
        "tfFile": "main.tf"
    }))
    .is_err());
    assert!(crate::infra_swift::swift_importers_json_impl(json!({
        "config": missing,
        "file": "Sources/App/Main.swift"
    }))
    .is_err());
    assert!(crate::infra_swift::swift_test_targets_json_impl(json!({
        "config": missing,
        "file": "Sources/App/Main.swift"
    }))
    .is_err());
}

#[test]
fn cli_parity_json_impls_reject_invalid_options_and_missing_documents() {
    let bad = json!({ "unknownField": true });
    let missing = missing_config();

    assert!(crate::fetches_json_impl(bad.clone()).is_err());
    assert!(crate::fetches_json_impl(json!({ "config": missing })).is_err());

    assert!(crate::resolve_config_json_impl(bad.clone()).is_err());
    assert!(crate::resolve_config_json_impl(json!({
        "config": missing,
        "view": "resolved"
    }))
    .is_err());
    assert!(crate::resolve_config_json_impl(json!({
        "config": missing,
        "view": "declared"
    }))
    .is_err());

    assert!(crate::tests_targets_json_impl(bad.clone()).is_err());
    assert!(crate::tests_targets_json_impl(json!({})).is_err());
    assert!(crate::tests_targets_json_impl(json!({ "framework": "invalid-fw" })).is_err());
    assert!(crate::tests_targets_json_impl(json!({ "framework": "vitest" })).is_err());
    assert!(crate::tests_targets_json_impl(json!({
        "framework": "vitest",
        "files": ["src/a.ts"],
        "config": missing
    }))
    .is_err());

    assert!(crate::ci_impact_json_impl(bad.clone()).is_err());
    assert!(crate::ci_impact_json_impl(json!({ "config": missing, "files": ["a.ts"] })).is_err());
    assert!(crate::ci_env_json_impl(bad.clone()).is_err());
    assert!(crate::ci_env_json_impl(json!({})).is_err());
    assert!(crate::ci_env_json_impl(json!({ "var": "CI", "config": missing })).is_err());

    assert!(crate::ci_topology_impact_json_impl(bad.clone()).is_err());
    assert!(crate::ci_topology_impact_json_impl(json!({})).is_err());
    assert!(crate::ci_topology_impact_json_impl(json!({ "base": "HEAD" })).is_err());
    assert!(
        crate::ci_topology_impact_json_impl(json!({ "base": "HEAD", "head": "HEAD" })).is_err()
    );
    assert!(crate::ci_topology_impact_json_impl(json!({
        "root": "/nonexistent-no-mistakes-validation-root",
        "base": "HEAD",
        "head": "HEAD",
        "entryWorkflow": ".github/workflows/ci.yml"
    }))
    .is_err());

    assert!(crate::impacted_checks_json_impl(bad.clone()).is_err());
    assert!(crate::impacted_checks_json_impl(json!({
        "config": missing,
        "changedFiles": ["src/a.ts"]
    }))
    .is_err());

    assert!(crate::tests_why_json_impl(bad.clone()).is_err());
    assert!(crate::tests_why_json_impl(json!({
        "framework": "vitest",
        "test": "a.test.ts",
        "config": missing
    }))
    .is_err());

    for f in [
        crate::tests_comment_markdown_impl,
        crate::tests_graph_json_impl,
        crate::tests_graph_mermaid_impl,
    ] {
        assert!(f(bad.clone()).is_err());
        assert!(f(json!({})).is_err());
        assert!(f(json!({ "planJson": "{" })).is_err());
        assert!(f(json!({ "planJson": { "invalid": true } })).is_err());
        assert!(f(json!({ "plan": missing })).is_err());
    }

    assert!(crate::tests_audit_json_impl(bad).is_err());
    assert!(crate::tests_audit_json_impl(json!({})).is_err());
    assert!(crate::tests_audit_json_impl(json!({ "planJson": "{" })).is_err());
    assert!(crate::tests_audit_json_impl(json!({ "plan": missing })).is_err());
}
