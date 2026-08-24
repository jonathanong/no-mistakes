use super::*;
use serde_json::json;

#[test]
fn check_json_preserves_nextjs_caching_report_when_auditing_suppression() {
    let (_, audit) = baseline_and_audit("aggregate-nextjs-no-caching");
    assert_suppression(
        &audit,
        &json!({
            "domain": "rules",
            "rule": "nextjs-no-caching",
            "file": "web/app/page.ts",
            "line": 3,
            "directiveKind": "nextLine",
            "directiveLine": 2,
            "reason": "fetch cache: \"force-cache\" is disabled; use uncached request-time data",
        }),
    );
}

#[test]
fn check_json_preserves_nextjs_api_report_when_auditing_suppression() {
    let (_, audit) = baseline_and_audit("aggregate-nextjs-no-api-routes");
    assert_suppression(
        &audit,
        &json!({
            "domain": "rules",
            "rule": "nextjs-no-api-routes",
            "file": "web/pages/api/legacy.ts",
            "line": 1,
            "directiveKind": "line",
            "directiveLine": 1,
            "reason": "Next.js API/server routes are disabled; move server endpoints out of the Next.js app",
        }),
    );
}

#[test]
fn check_json_preserves_direct_and_reachable_dynamic_import_reports_when_auditing() {
    let (_, audit) = baseline_and_audit("aggregate-test-no-unmocked-dynamic-imports");
    assert_suppression(
        &audit,
        &json!({
            "domain": "rules",
            "rule": "test-no-unmocked-dynamic-imports",
            "file": "src/reachable.mts",
            "line": 3,
            "directiveKind": "nextLine",
            "directiveLine": 2,
            "reason": "dynamic import dependency `src/leaf.mts` must be mocked",
        }),
    );
    assert_suppression(
        &audit,
        &json!({
            "domain": "rules",
            "rule": "test-no-unmocked-dynamic-imports",
            "file": "tests/direct.test.mts",
            "line": 5,
            "directiveKind": "nextLine",
            "directiveLine": 4,
            "reason": "dynamic import dependency `src/leaf.mts` must be mocked",
        }),
    );
}

#[test]
fn check_json_preserves_server_boundary_report_when_auditing_suppression() {
    let (_, audit) = baseline_and_audit("aggregate-server-route-client-boundary");
    assert_suppression(
        &audit,
        &json!({
            "domain": "rules",
            "rule": "server-route-client-boundary",
            "file": "backend/api/client.ts",
            "line": 4,
            "directiveKind": "file",
            "directiveLine": 1,
            "reason": "client HTTP call is in a server route folder; move request clients out of route definition folders or narrow server route globs so AST route extraction stays unambiguous",
        }),
    );
}

#[test]
fn check_json_preserves_agents_size_report_when_auditing_suppression() {
    let (baseline, audit) = baseline_and_audit("aggregate-agents-md-max-size");
    assert!(baseline["rules"].as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["rule"] == "agents-md-max-size"
                && item["file"] == "AGENTS.md"
                && item["message"] == "3 lines (max 2) - trim to keep agent context lean"
        })
    }));
    assert_eq!(audit["suppressed"], json!([]));
}

#[test]
fn check_json_accounts_for_react_queue_and_integration_adapters() {
    let fixtures = [
        ("suppression-react", "react", "assert-no-fetch", "nextLine"),
        ("suppression-queues", "queues", "queues-check", "file"),
        (
            "suppression-filesystem",
            "filesystem",
            "no-empty-or-comments-only-files",
            "file",
        ),
        (
            "suppression-integration",
            "integration",
            "integration-test-no-mocks",
            "file",
        ),
    ];
    for (fixture, domain, rule, directive_kind) in fixtures {
        let (baseline, audit) = baseline_and_audit(fixture);
        let result_field = if domain == "filesystem" {
            "rules"
        } else {
            domain
        };
        assert!(
            baseline[result_field].as_array().is_some_and(Vec::is_empty),
            "default check must filter {domain} directives: {baseline}"
        );
        assert!(
            audit["suppressed"]
                .as_array()
                .is_some_and(|findings| findings.iter().any(|finding| {
                    finding["domain"] == domain
                        && finding["rule"] == rule
                        && finding["directive"]["kind"] == directive_kind
                })),
            "{fixture}: {audit}"
        );
    }
}

#[test]
fn check_json_records_react_next_line_directive_at_the_fetch_location() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/suppression-react");
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root, "includeSuppressed": true }).to_string(),
    ))
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let finding = value["suppressed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["domain"] == "react")
        .unwrap_or_else(|| panic!("missing React suppression: {value}"));
    assert_eq!(finding["line"], 3);
    assert_eq!(finding["directive"]["kind"], "nextLine");
    assert_eq!(finding["directive"]["line"], 2);
}

#[test]
fn check_json_uses_filter_precedence_for_overlapping_directives() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-directive-precedence");
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root, "includeSuppressed": true }).to_string(),
    ))
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let finding = value["suppressed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["domain"] == "react")
        .unwrap_or_else(|| panic!("missing React suppression: {value}"));
    assert_eq!(finding["line"], 4);
    assert_eq!(finding["directive"]["kind"], "nextLine");
    assert_eq!(finding["directive"]["line"], 3);
}

#[test]
fn check_json_does_not_hide_later_react_fetch_after_first_is_suppressed() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-react-multiple");
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root, "includeSuppressed": true }).to_string(),
    ))
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(!value["react"].as_array().unwrap().is_empty(), "{value}");
    assert!(value["suppressed"]
        .as_array()
        .is_some_and(|items| items.iter().all(|item| item["domain"] != "react")));
    assert!(value["react"]
        .as_array()
        .is_some_and(|items| { items.iter().any(|item| item["file"] == "app/Fetcher.tsx") }));
}

#[test]
fn ordinary_check_keeps_later_react_component_after_earlier_component_is_suppressed() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-react-component-order");
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root }).to_string(),
    ))
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value.get("suppressed").is_none());
    assert!(
        value["react"].as_array().is_some_and(|findings| {
            findings.iter().any(|finding| {
                finding["file"] == "app/Later.tsx" && finding["rule"] == "assert-no-fetch"
            })
        }),
        "{value}"
    );
}

#[test]
fn check_json_records_one_react_suppression_per_component_after_all_fetches_are_hidden() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-react-all-multiple");
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root, "includeSuppressed": true }).to_string(),
    ))
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(value["react"].as_array().is_some_and(Vec::is_empty));
    let react_suppressions = value["suppressed"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["domain"] == "react")
        .collect::<Vec<_>>();
    assert_eq!(react_suppressions.len(), 2, "{value}");
    assert_eq!(
        react_suppressions
            .iter()
            .filter(|item| item["reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("component default@app/Fetcher.tsx")))
            .count(),
        1,
        "{value}"
    );
    assert!(react_suppressions.iter().any(|item| {
        item["file"] == "app/Child.tsx"
            && item["sourceFile"] == "app/Child.tsx"
            && item["line"] == 3
    }));
    // Fetcher also inherits Child, but its own suppressed fetch remains the
    // component diagnostic target instead of inheriting the child's line.
    assert!(react_suppressions.iter().any(|item| {
        item["file"] == "app/Fetcher.tsx"
            && item["sourceFile"] == "app/Fetcher.tsx"
            && item["line"] == 5
    }));
}

#[test]
fn check_json_accounts_for_suppressed_combined_rust_rule() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-rust-combined");
    let baseline: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root }).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    assert!(baseline["rules"].as_array().is_some_and(Vec::is_empty));

    let audit: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "includeSuppressed": true }).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    assert!(audit["rules"].as_array().is_some_and(Vec::is_empty));
    assert!(audit["suppressed"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| {
            item["domain"] == "filesystem" && item["rule"] == "rust-no-inline-allows"
        })));
}
