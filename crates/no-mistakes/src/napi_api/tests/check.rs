use super::*;
use serde_json::json;

fn static_check_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check")
        .join(name)
}

fn baseline_and_audit(name: &str) -> (serde_json::Value, serde_json::Value) {
    let root = static_check_fixture(name);
    let baseline: serde_json::Value =
        serde_json::from_str(&check_json_impl(json!({ "root": root }).to_string()).unwrap())
            .unwrap();
    let audit: serde_json::Value = serde_json::from_str(
        &check_json_impl(json!({ "root": root, "includeSuppressed": true }).to_string()).unwrap(),
    )
    .unwrap();
    let mut comparable = audit.clone();
    comparable
        .as_object_mut()
        .expect("check report is an object")
        .remove("suppressed");
    assert_eq!(baseline, comparable, "audit changed a visible report field");
    (baseline, audit)
}

fn assert_suppression(audit: &serde_json::Value, expected: &serde_json::Value) {
    let domain = expected["domain"].as_str().unwrap();
    let rule = expected["rule"].as_str().unwrap();
    let file = expected["file"].as_str().unwrap();
    let line = expected["line"].as_u64().unwrap();
    let finding = audit["suppressed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| {
            finding["domain"] == domain
                && finding["rule"] == rule
                && finding["file"] == file
                && finding["line"] == line
        })
        .unwrap_or_else(|| panic!("missing suppression {domain}/{rule} {file}:{line}: {audit}"));
    assert_eq!(finding["reason"], expected["reason"]);
    assert_eq!(finding["directive"]["kind"], expected["directiveKind"]);
    assert_eq!(finding["directive"]["line"], expected["directiveLine"]);
}

#[test]
fn check_json_reports_tracked_artifacts_below_source_skip_directories() {
    let fixture = crate::test_support::materialize_gitignore_fixture("banned-paths-source-skips");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let output = check_json_impl(json!({ "root": fixture.path() }).to_string()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let files = value["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "banned-paths")
        .map(|finding| finding["file"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        files,
        vec![
            "build/blocked.patch",
            "dist/blocked.patch",
            "fixtures/blocked.patch",
            "nested/blocked.patch",
            "target/blocked.patch",
        ]
    );
}

#[test]
fn check_json_returns_global_check_report() {
    let options = json!({
        "root": fixture_root("unique-exports-basic"),
        "config": ".no-mistakes.yml",
        "tsconfig": "tsconfig.json"
    })
    .to_string();
    let output = check_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["codebase"].as_array().unwrap().iter().any(|finding| {
        finding["rule"] == "unique-exports" && finding["exportName"] == "shared"
    }));
    assert!(value["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn check_json_optionally_accounts_for_suppressed_ordinary_rule_findings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-accounting");
    let baseline = check_json_impl(json!({ "root": root }).to_string()).unwrap();
    let baseline: serde_json::Value = serde_json::from_str(&baseline).unwrap();
    assert!(baseline.get("suppressed").is_none());
    assert!(baseline["codebase"].as_array().is_some_and(Vec::is_empty));

    let audit = check_json_impl(
        json!({
            "root": root,
            "includeSuppressed": true,
        })
        .to_string(),
    )
    .unwrap();
    let audit: serde_json::Value = serde_json::from_str(&audit).unwrap();
    assert_eq!(audit["codebase"], json!([]));
    assert_eq!(audit["suppressed"].as_array().unwrap().len(), 2);
    assert_eq!(audit["suppressed"][0]["domain"], "codebase");
    assert_eq!(audit["suppressed"][0]["rule"], "unique-exports");
    assert_eq!(audit["suppressed"][0]["line"], 2);
    assert_eq!(audit["suppressed"][0]["directive"]["kind"], "nextLine");
    assert_eq!(audit["suppressed"][0]["directive"]["line"], 1);
    assert_eq!(audit["suppressed"][1]["file"], "src/c.ts");
    assert_eq!(audit["suppressed"][1]["line"], 1);
    assert_eq!(audit["suppressed"][1]["directive"]["kind"], "line");
    assert_eq!(audit["suppressed"][1]["directive"]["line"], 1);
}

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
    let (_, audit) = baseline_and_audit("aggregate-agents-md-max-size");
    assert_suppression(
        &audit,
        &json!({
            "domain": "filesystem",
            "rule": "agents-md-max-size",
            "file": "AGENTS.md",
            "line": 1,
            "directiveKind": "file",
            "directiveLine": 1,
            "reason": "3 lines (max 2) - trim to keep agent context lean",
        }),
    );
}

#[test]
fn check_json_preserves_storybook_file_and_component_reports_when_auditing_suppression() {
    let (_, audit) = baseline_and_audit("aggregate-require-storybook-stories");
    assert_suppression(
        &audit,
        &json!({
            "domain": "rules",
            "rule": "require-storybook-stories",
            "file": "web/components/ComponentSuppressed.tsx",
            "line": 2,
            "directiveKind": "nextLine",
            "directiveLine": 1,
            "reason": "React component `ComponentSuppressed` is selected for Storybook coverage but no reachable story imports it or a parent component that renders it. Add a Storybook story, add an accepted colocated test when `allow_colocated_tests` is enabled, render it through a covered parent component, exclude it from `require-storybook-stories`, or add a documented no-mistakes disable comment.",
        }),
    );
    assert_suppression(
        &audit,
        &json!({
            "domain": "rules",
            "rule": "require-storybook-stories",
            "file": "web/components/FileSuppressed.tsx",
            "line": 2,
            "directiveKind": "file",
            "directiveLine": 1,
            "reason": "React component `FileSuppressed` is selected for Storybook coverage but no reachable story imports it or a parent component that renders it. Add a Storybook story, add an accepted colocated test when `allow_colocated_tests` is enabled, render it through a covered parent component, exclude it from `require-storybook-stories`, or add a documented no-mistakes disable comment.",
        }),
    );
}

#[test]
fn check_json_audit_mode_includes_an_empty_suppression_array() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-runner/empty");
    let baseline = check_json_impl(json!({ "root": root }).to_string()).unwrap();
    let baseline: serde_json::Value = serde_json::from_str(&baseline).unwrap();
    assert!(baseline.get("suppressed").is_none());

    let audit =
        check_json_impl(json!({ "root": root, "includeSuppressed": true }).to_string()).unwrap();
    let audit: serde_json::Value = serde_json::from_str(&audit).unwrap();
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
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check")
            .join(fixture);
        let output =
            check_json_impl(json!({ "root": root, "includeSuppressed": true }).to_string())
                .unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(
            value["suppressed"]
                .as_array()
                .is_some_and(|findings| findings.iter().any(|finding| {
                    finding["domain"] == domain
                        && finding["rule"] == rule
                        && finding["directive"]["kind"] == directive_kind
                })),
            "{fixture}: {value}"
        );
    }
}

#[test]
fn check_json_records_react_next_line_directive_at_the_fetch_location() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/suppression-react");
    let output =
        check_json_impl(json!({ "root": root, "includeSuppressed": true }).to_string()).unwrap();
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
fn check_json_keeps_unsuppressed_duplicate_when_suppressed_export_sorts_first() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-unique-canonical");
    let baseline: serde_json::Value =
        serde_json::from_str(&check_json_impl(json!({ "root": root }).to_string()).unwrap())
            .unwrap();
    assert!(baseline["codebase"].as_array().is_some_and(Vec::is_empty));

    let audit: serde_json::Value = serde_json::from_str(
        &check_json_impl(json!({ "root": root, "includeSuppressed": true }).to_string()).unwrap(),
    )
    .unwrap();
    assert!(audit["codebase"].as_array().is_some_and(Vec::is_empty));
    assert!(audit["suppressed"].as_array().is_some_and(|items| items
        .iter()
        .any(|item| { item["rule"] == "unique-exports" && item["file"] == "src/a.ts" })));
}

#[test]
fn check_json_does_not_hide_later_react_fetch_after_first_is_suppressed() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-react-multiple");
    let output =
        check_json_impl(json!({ "root": root, "includeSuppressed": true }).to_string()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(!value["react"].as_array().unwrap().is_empty(), "{value}");
    assert!(value["suppressed"].as_array().is_some_and(|items| items
        .iter()
        .any(|item| { item["domain"] == "react" && item["line"] == 3 })));
}

#[test]
fn check_json_accounts_for_suppressed_combined_rust_rule() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-rust-combined");
    let baseline: serde_json::Value =
        serde_json::from_str(&check_json_impl(json!({ "root": root }).to_string()).unwrap())
            .unwrap();
    assert!(baseline["rules"].as_array().is_some_and(Vec::is_empty));

    let audit: serde_json::Value = serde_json::from_str(
        &check_json_impl(json!({ "root": root, "includeSuppressed": true }).to_string()).unwrap(),
    )
    .unwrap();
    assert!(audit["rules"].as_array().is_some_and(Vec::is_empty));
    assert!(audit["suppressed"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| {
            item["domain"] == "filesystem" && item["rule"] == "rust-no-inline-allows"
        })));
}

#[test]
fn check_json_returns_warnings_for_skipped_configured_check() {
    let options = json!({
        "root": fixture_root("test-no-unmocked-dynamic-imports-unknown-vitest-project"),
    })
    .to_string();
    let output = check_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["warnings"].as_array().unwrap().iter().any(|warning| {
        warning
            .as_str()
            .is_some_and(|warning| warning.contains("unknown vitest project web"))
    }));
    assert_eq!(value["rules"].as_array().map(Vec::len), Some(0));
}

#[test]
fn check_json_returns_non_blocking_agent_doc_advisories() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/agents-md-max-size/fixture/advisory");
    let options = json!({
        "root": root,
        "config": ".no-mistakes.yml"
    })
    .to_string();
    let output = check_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["rules"].as_array().map(Vec::len), Some(0));
    assert!(value["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .any(|advisory| {
            advisory["rule"] == "agents-md-max-size"
                && advisory["file"] == "CLAUDE.md"
                && advisory["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("8 remaining"))
        }));
}

#[test]
fn check_json_returns_migrated_filesystem_rules() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/markdown-link-display-text/fixture");
    let options = json!({
        "root": root,
        "config": ".no-mistakes.yml"
    })
    .to_string();
    let output = check_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["rules"].as_array().unwrap().iter().any(|finding| {
        finding["rule"] == "markdown-link-display-text"
            && finding["file"] == "docs/bad.md"
            && finding["target"] == "news-story-clusters.md"
    }));
}

#[test]
fn check_json_reports_both_markdown_rule_ids() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/markdown-report/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let output = check_json_impl(
        json!({ "root": fixture.path(), "config": ".no-mistakes.yml" }).to_string(),
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let findings = value["rules"].as_array().unwrap();
    let rule_ids = findings
        .iter()
        .filter_map(|finding| finding["rule"].as_str())
        .collect::<Vec<_>>();

    assert!(rule_ids.contains(&"markdown-reachability"), "{value:#?}");
    assert!(
        rule_ids.contains(&"markdown-structure-budget"),
        "{value:#?}"
    );
    for rule_id in ["markdown-reachability", "markdown-structure-budget"] {
        let finding = findings
            .iter()
            .find(|finding| finding["rule"] == rule_id)
            .unwrap_or_else(|| panic!("missing {rule_id}: {value:#?}"));
        assert!(finding["file"].is_string(), "{finding:#?}");
        assert!(finding["line"].is_u64(), "{finding:#?}");
        assert!(finding["message"].is_string(), "{finding:#?}");
    }
}
