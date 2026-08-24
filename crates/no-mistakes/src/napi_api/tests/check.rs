use super::*;
use serde_json::json;

#[path = "check_suppression.rs"]
mod check_suppression;
#[path = "check_suppression_audit.rs"]
mod check_suppression_audit;

fn static_check_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check")
        .join(name)
}

fn baseline_and_audit(name: &str) -> (serde_json::Value, serde_json::Value) {
    let root = static_check_fixture(name);
    let baseline: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root }).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    let audit: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "includeSuppressed": true }).to_string(),
        ))
        .unwrap(),
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
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": fixture.path() }).to_string(),
    ))
    .unwrap();
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
    let output = check_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
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
    let baseline = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root }).to_string(),
    ))
    .unwrap();
    let baseline: serde_json::Value = serde_json::from_str(&baseline).unwrap();
    assert!(baseline.get("suppressed").is_none());
    assert!(baseline["codebase"].as_array().is_some_and(Vec::is_empty));

    let audit = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "includeSuppressed": true,
        })
        .to_string(),
    ))
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
fn check_json_returns_warnings_for_skipped_configured_check() {
    let options = json!({
        "root": fixture_root("test-no-unmocked-dynamic-imports-unknown-vitest-project"),
    })
    .to_string();
    let output = check_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
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
    let output = check_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
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
fn check_json_rejects_invalid_rule_option_types_with_the_cli_diagnostic() {
    let root = static_check_fixture("invalid-rule-options");
    let error = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root }).to_string(),
    ))
    .expect_err("invalid configured rule options must reject the N-API check");
    let message = error.reason;

    assert!(message.contains(
        "invalid options for rule `postgres-no-add-column` application `strict SQL migrations`"
    ));
    assert!(message.contains("options.sqlInclude"));
    assert!(message.contains("boolean"));
    assert!(message.contains("expected a sequence"));
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
    let output = check_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["rules"].as_array().unwrap().iter().any(|finding| {
        finding["rule"] == "markdown-link-display-text"
            && finding["file"] == "docs/bad.md"
            && finding["target"] == "news-story-clusters.md"
    }));
}

#[test]
fn check_json_honors_release_age_group_validation_suppression() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/pnpm-release-age-policy/fixture/suppression");
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root, "includeSuppressed": true }).to_string(),
    ))
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["rules"].as_array().unwrap().is_empty(), "{value}");
    assert!(value["suppressed"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| {
            finding["rule"] == "pnpm-release-age-policy" && finding["file"] == "pnpm-workspace.yaml"
        }));
}

#[test]
fn check_json_enforces_exact_postgres_add_column_migration_allowlist() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-add-column/fixture/mismatch");
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root, "config": ".no-mistakes.yml" }).to_string(),
    ))
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let findings = value["rules"].as_array().unwrap();

    assert_eq!(findings.len(), 2, "{value:#?}");
    assert!(findings.iter().any(|finding| {
        finding["rule"] == "postgres-no-add-column"
            && finding["target"] == "posts.status"
            && finding["message"].as_str().is_some_and(|message| {
                message.contains("does not match an allowedMigrations entry")
            })
    }));
    assert!(findings.iter().any(|finding| {
        finding["rule"] == "postgres-no-add-column"
            && finding["message"].as_str().is_some_and(|message| {
                message.contains("stale postgres-no-add-column allowedMigrations entry")
            })
    }));
}

#[test]
fn check_json_reports_both_markdown_rule_ids() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/markdown-report/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": fixture.path(), "config": ".no-mistakes.yml" }).to_string(),
    ))
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
