use super::{run_all_with_options, AnalysisOptions};
use no_mistakes::config::v2::Effort;
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/finite-set-effort/fixture"),
    )
}

fn run(root: &Path, config: &str, effort: Option<Effort>) -> super::CheckResults {
    crate::test_support::without_hygiene_advisories(
        run_all_with_options(
            root.to_path_buf(),
            Some(root.join(config)),
            None,
            false,
            false,
            AnalysisOptions {
                effort,
                include_sql_verdicts: false,
            },
        )
        .unwrap(),
    )
}

fn unknown(row: &no_mistakes::codebase::rules::RuleFinding) -> bool {
    row.message.contains("unanalyzable at effort low")
}

#[test]
fn finite_set_effort_default_is_max_and_override_preserves_output() {
    let root = fixture();
    let max = run(&root, "max.yml", None);
    assert_eq!(max.rules.len(), 4, "{:?}", max.rules);
    for result in [
        run(&root, "default.yml", None),
        run(&root, "low-advise.yml", Some(Effort::Max)),
    ] {
        assert_eq!(result.rules, max.rules);
        assert!(result.advisories.is_empty());
        assert!(result.analysis_fallbacks.is_empty());
    }
}

#[test]
fn finite_set_effort_routes_uncertainty_and_preserves_definite_subset() {
    let root = fixture();
    let max = run(&root, "max.yml", None);
    for (config, findings, advisories) in [
        ("low-report.yml", 4, 0),
        ("low-advise.yml", 1, 3),
        ("low-ignore.yml", 1, 0),
    ] {
        let result = run(&root, config, None);
        assert_eq!(result.rules.len(), findings, "{config}: {:?}", result.rules);
        assert_eq!(result.advisories.len(), advisories, "{config}");
        assert_eq!(result.analysis_warning_count, 0);
        assert!(result.analysis_fallbacks.is_empty());
        let definite: Vec<_> = result.rules.iter().filter(|row| !unknown(row)).collect();
        assert_eq!(definite.len(), 1);
        assert!(definite.into_iter().all(|row| max.rules.contains(row)));
        for row in result
            .rules
            .iter()
            .chain(&result.advisories)
            .filter(|row| unknown(row))
        {
            assert!(row.message.contains("max"), "{row:?}");
        }
    }
}

#[test]
fn finite_set_effort_mixed_applications_and_custom_messages() {
    let root = fixture();
    let mixed = run(&root, "mixed.yml", None);
    assert_eq!(mixed.rules.len(), 3, "{:?}", mixed.rules);
    assert_eq!(mixed.advisories.len(), 1);
    let custom = run(&root, "custom-message.yml", None);
    assert_eq!(custom.rules.len(), 1);
    assert_eq!(custom.advisories.len(), 3);
    for row in &custom.rules {
        assert!(row.message.starts_with("Inventory policy:"), "{row:?}");
    }
    assert!(custom.advisories.iter().all(unknown));
}

#[test]
fn finite_set_effort_suppression_and_ignore_do_not_enable_incomplete_comparisons() {
    let root = fixture();
    let suppressed = run(&root, "low-suppression.yml", None);
    assert_eq!(suppressed.rules.len(), 1, "{:?}", suppressed.rules);
    assert!(suppressed.advisories.is_empty());
    assert!(!unknown(&suppressed.rules[0]));
    let ignored = run(&root, "low-min-size-ignore.yml", None);
    assert!(ignored.rules.is_empty() && ignored.advisories.is_empty());
}

#[test]
fn finite_set_effort_total_count_can_exceed_max_for_compatible_source_sets() {
    let root = fixture();
    for (config, findings, advisories) in [
        ("compatible-max-report.yml", 0, 0),
        ("compatible-low-report.yml", 1, 0),
        ("compatible-low-advise.yml", 0, 1),
        ("compatible-low-ignore.yml", 0, 0),
    ] {
        let result = run(&root, config, None);
        assert_eq!(result.rules.len(), findings, "{config}: {:?}", result.rules);
        assert_eq!(result.advisories.len(), advisories, "{config}");
        assert!(result.rules.iter().chain(&result.advisories).all(unknown));
    }
}

#[test]
fn finite_set_effort_propagates_source_and_remaining_rule_errors() {
    let root = fixture();
    for (config, expected) in [
        ("missing-source-ignore.yml", "absent.ts"),
        ("remaining-rule-error.yml", "banned-paths"),
    ] {
        let error = run_all_with_options(
            root.clone(),
            Some(root.join(config)),
            None,
            false,
            false,
            AnalysisOptions {
                effort: None,
                include_sql_verdicts: false,
            },
        )
        .err()
        .expect("invalid saved input must remain an error");
        assert!(error.to_string().contains(expected), "{config}: {error:#}");
    }
}
