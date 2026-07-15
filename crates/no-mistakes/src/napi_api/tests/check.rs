use super::*;
use serde_json::json;

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
