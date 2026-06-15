use super::*;
use serde_json::json;
use std::path::PathBuf;

fn fixture_root(name: &str) -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
    .display()
    .to_string()
}

#[test]
fn infra_resource_refs_impl_returns_referencing_blocks() {
    let options = json!({
        "root": fixture_root("terraform-basic"),
        "address": "aws_route53_record.foo",
    })
    .to_string();
    let output = infra_resource_refs_json_impl(options).unwrap();
    assert!(output.contains("aws_lb.web"));
}

#[test]
fn infra_outputs_impl_returns_exports() {
    let options = json!({
        "root": fixture_root("terraform-basic"),
        "moduleDir": "infra/modules/network",
    })
    .to_string();
    let output = infra_outputs_json_impl(options).unwrap();
    assert!(output.contains("zone_id"));
}

#[test]
fn infra_test_for_impl_returns_covering_tests() {
    let options = json!({
        "root": fixture_root("terraform-basic"),
        "tfFile": "infra/envs/prod/main.tf",
    })
    .to_string();
    let output = infra_test_for_json_impl(options).unwrap();
    assert!(output.contains("network.test.mts"));
}

#[test]
fn infra_impls_require_their_arguments() {
    let options = json!({ "root": fixture_root("terraform-basic") }).to_string();
    assert!(infra_resource_refs_json_impl(options.clone()).is_err());
    assert!(infra_outputs_json_impl(options.clone()).is_err());
    assert!(infra_test_for_json_impl(options).is_err());
}

#[test]
fn swift_importers_impl_returns_importers() {
    let options = json!({
        "root": fixture_root("swift-test-plan"),
        "file": "swift-clients/core/Sources/VouchaAPI/Endpoint.swift",
    })
    .to_string();
    let output = swift_importers_json_impl(options).unwrap();
    assert!(output.contains("APIClient.swift"));
}

#[test]
fn swift_test_targets_impl_returns_targets() {
    let options = json!({
        "root": fixture_root("swift-test-plan"),
        "file": "swift-clients/core/Sources/VouchaAPI/Endpoint.swift",
    })
    .to_string();
    let output = swift_test_targets_json_impl(options).unwrap();
    assert!(output.contains("VouchaCoreTests"));
}

#[test]
fn swift_impls_require_file() {
    let options = json!({ "root": fixture_root("swift-test-plan") }).to_string();
    assert!(swift_importers_json_impl(options.clone()).is_err());
    assert!(swift_test_targets_json_impl(options).is_err());
}

#[test]
fn options_reject_unknown_fields() {
    let options = json!({ "root": ".", "bogus": true }).to_string();
    assert!(parse_options::<InfraOptions>(&options).is_err());
    assert!(parse_options::<SwiftOptions>(&options).is_err());
}
