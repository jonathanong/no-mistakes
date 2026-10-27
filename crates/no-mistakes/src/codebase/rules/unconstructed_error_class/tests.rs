use super::*;
use crate::codebase::rules::run_check;
#[cfg(unix)]
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

// Output cases moved to cli_rra_snapshot_unconstructed_error_class.

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/fixture"),
    )
}

/// `file:line name` for each finding, in report order.
#[cfg(unix)]
fn reported(findings: &[RuleFinding]) -> Vec<String> {
    findings
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.file,
                finding.line,
                finding.target.as_deref().unwrap_or("?")
            )
        })
        .collect()
}

fn diagnostic_report(root: &std::path::Path) -> serde_json::Value {
    let output = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({"root":root}).to_string(),
    ))
    .unwrap();
    serde_json::from_str(&output).unwrap()
}

fn reported_channel(report: &serde_json::Value, channel: &str) -> Vec<String> {
    report[channel]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            format!(
                "{}:{} {}",
                row["file"].as_str().unwrap(),
                row["line"],
                row["target"].as_str().unwrap()
            )
        })
        .collect()
}

#[test]
fn graph_plan_requests_calls_and_extends_when_configured() {
    let root = fixture();
    let configured =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let plan = graph_plan(&configured).expect("configured rule needs a graph");
    assert!(plan.calls && plan.extends);
    assert!(graph_plan(&NoMistakesConfig::default()).is_none());
}

/// A file the graph could not parse might hold the only construction of a
/// class, so the rule errors instead of reporting from an incomplete view.
#[test]
fn a_broken_source_file_is_a_rule_error_naming_the_file() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/parse-failure/fixture"),
    );
    let config = root.join("configs/one-broken-source.yml");
    let error = run_check(&root, Some(&config), None)
        .unwrap_err()
        .to_string();
    assert!(
        error.starts_with(
            "unconstructed-error-class: cannot prove error classes unconstructed: \
             `src/broken.ts` failed to parse: "
        ),
        "{error}"
    );
}

/// A test file's alias of a namespace keeps its classes quiet, so a broken test
/// file might hold the use that keeps `Errors.Dead` quiet: the rule errors
/// instead of reporting it.
#[test]
fn a_broken_test_file_is_a_rule_error_when_a_namespace_member_would_be_reported() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/unconstructed-error-class/test-escape-parse-failure/fixture",
        ),
    );
    let error = run_check(&root, None, None).unwrap_err().to_string();
    assert!(
        error.starts_with(
            "unconstructed-error-class: cannot prove error classes unconstructed: \
             `src/__tests__/alias.ts` failed to parse: "
        ),
        "{error}"
    );
}

include!("tests_modules.rs");
