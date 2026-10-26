use super::{run_all_with_options, AnalysisOptions};
use no_mistakes::config::v2::Effort;
use std::path::{Path, PathBuf};

const RULE: &str = "github-actions-download-integrity";

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/download-integrity-effort/fixture"),
    )
}

fn run(root: &Path, config: Option<&str>, effort: Option<Effort>) -> super::CheckResults {
    crate::test_support::policy_advisories(
        run_all_with_options(
            root.to_path_buf(),
            config.map(|name| root.join(name)),
            None,
            false,
            false,
            AnalysisOptions {
                effort,
                include_sql_verdicts: false,
            },
        )
        .unwrap(),
        "github-actions-download-integrity",
    )
}

fn shape(row: &no_mistakes::codebase::rules::RuleFinding) -> bool {
    row.message.contains("unanalyzable at effort low")
}

#[test]
fn default_is_explicit_max_on_existing_and_new_fixtures() {
    let legacy = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-download-integrity/fixture/fail");
    for (root, default_config, max_config) in [
        (legacy, None, None),
        (fixture(), Some("default.yml"), Some("max.yml")),
    ] {
        let default = run(&root, default_config, None);
        let max = run(&root, max_config, Some(Effort::Max));
        assert!(!default.rules.is_empty());
        assert_eq!(
            serde_json::to_value(&default.rules).unwrap(),
            serde_json::to_value(&max.rules).unwrap()
        );
        assert_eq!(default.advisories, max.advisories);
        assert!(max.rules.iter().all(|row| !shape(row)));
    }
}

#[test]
fn low_routes_uncertain_commands_once_and_preserves_definite_findings() {
    let root = fixture();
    let max = run(&root, Some("max.yml"), None);
    assert_eq!(max.rules.len(), 4, "{:?}", max.rules);
    for (policy, findings, advisories) in [("report", 4, 0), ("advise", 2, 2), ("ignore", 2, 0)] {
        let low = run(&root, Some(&format!("low-{policy}.yml")), None);
        assert!(low.analysis_fallbacks.is_empty());
        assert_eq!(low.analysis_warning_count, 0);
        assert_eq!(low.rules.len(), findings, "{policy}: {:?}", low.rules);
        assert_eq!(low.advisories.len(), advisories, "{policy}");
        assert_eq!(low.rules.iter().filter(|row| !shape(row)).count(), 2);
        for finding in low.rules.iter().filter(|row| !shape(row)) {
            assert!(max.rules.contains(finding), "{policy}: {finding:?}");
        }
        let shapes = low
            .rules
            .iter()
            .chain(&low.advisories)
            .filter(|row| shape(row))
            .collect::<Vec<_>>();
        assert_eq!(shapes.len(), if policy == "ignore" { 0 } else { 2 });
        for row in shapes {
            assert_eq!(row.rule, RULE);
            assert!(row.message.to_ascii_lowercase().contains("literal"));
            assert!(row.message.to_ascii_lowercase().contains("url"));
            assert!(matches!(
                row.file.as_str(),
                ".github/workflows/ci.yml" | ".github/actions/fetch/action.yml"
            ));
            assert!(row.line > 1, "{row:?}");
        }
        // Extra suppressed and trusted/verified fixture calls must not leak
        // into either channel, including when only advisories are emitted.
        assert!(low
            .rules
            .iter()
            .chain(&low.advisories)
            .all(|row| !row.file.ends_with("disabled.yml")));
    }
}

#[test]
fn trusted_dynamic_host_adds_uncertainty_while_definite_low_findings_stay_a_subset() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/download-integrity-effort/trusted-host-subset"),
    );
    let max = run(&root, Some("max.yml"), None);
    let low = run(&root, Some("low-report.yml"), None);
    assert_eq!(max.rules.len(), 2, "{:?}", max.rules);
    assert_eq!(low.rules.len(), 3, "{:?}", low.rules);
    assert!(max.advisories.is_empty() && low.advisories.is_empty());
    assert!(low.analysis_fallbacks.is_empty());
    assert_eq!(low.analysis_warning_count, 0);
    let definite: Vec<_> = low.rules.iter().filter(|finding| !shape(finding)).collect();
    assert_eq!(definite.len(), 1);
    assert!(max.rules.contains(definite[0]));
    let shapes: Vec<_> = low.rules.iter().filter(|finding| shape(finding)).collect();
    assert_eq!(shapes.len(), 2);
    let trusted = shapes
        .iter()
        .find(|finding| finding.message.contains("step 3 "))
        .unwrap();
    assert_eq!(trusted.file, ".github/workflows/ci.yml");
    assert!(trusted.message.contains("$BASE_URL/repos/example/project"));
    // Max proves this assignment is trusted; low reports its unresolved URL shape.
    assert!(max
        .rules
        .iter()
        .all(|finding| !finding.message.contains("step 3 ")));
    assert!(max
        .rules
        .iter()
        .any(|finding| finding.message.contains("step 2 ")
            && finding
                .message
                .contains("https://untrusted.example.invalid/assigned.tgz")));
}

#[test]
fn invocation_max_overrides_the_rule_low_setting() {
    let root = fixture();
    let forced = run(&root, Some("low-report.yml"), Some(Effort::Max));
    let max = run(&root, Some("max.yml"), None);
    assert_eq!(
        serde_json::to_value(&forced.rules).unwrap(),
        serde_json::to_value(&max.rules).unwrap()
    );
    assert!(forced.advisories.is_empty());
}

#[test]
fn advisory_only_is_non_failing_and_mixed_applications_keep_their_policies() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/download-integrity-effort/uncertainty-only"),
    );
    for (policy, findings, advisories) in [("report", 1, 0), ("advise", 0, 1), ("ignore", 0, 0)] {
        let report = run(&root, Some(&format!("low-{policy}.yml")), None);
        assert_eq!(report.rules.len(), findings);
        assert_eq!(report.advisories.len(), advisories);
    }
    let mixed = run(&fixture(), Some("mixed.yml"), None);
    assert_eq!(mixed.rules.len(), 3, "{:?}", mixed.rules);
    assert_eq!(mixed.advisories.len(), 1);
    assert_eq!(mixed.advisories[0].file, ".github/workflows/ci.yml");
    assert!(mixed.rules.iter().all(|row| !shape(row)));
}
