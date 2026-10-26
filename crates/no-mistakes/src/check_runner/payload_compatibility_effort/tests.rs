use super::{run_all_with_options, AnalysisOptions};
use no_mistakes::config::v2::Effort;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/payload-compatibility-effort")
            .join(name),
    )
}

fn run(root: &Path, config: &str, effort: Option<Effort>) -> super::CheckResults {
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
    .unwrap()
}

fn shape(row: &no_mistakes::codebase::rules::RuleFinding) -> bool {
    row.message.contains("unanalyzable at effort low")
}

#[test]
fn payload_compatibility_default_is_max_and_invocation_max_overrides_low() {
    let root = fixture("fixture");
    let max = run(&root, "max.yml", None);
    assert_eq!(max.rules.len(), 3, "{:?}", max.rules);
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
fn payload_compatibility_policy_routes_shapes_and_preserves_definite_subset() {
    let root = fixture("fixture");
    let max = run(&root, "max.yml", None);
    for (config, findings, advisories) in [
        ("low-report.yml", 4, 0),
        ("low-advise.yml", 2, 2),
        ("low-ignore.yml", 2, 0),
    ] {
        let low = run(&root, config, None);
        assert_eq!(low.rules.len(), findings, "{config}: {:?}", low.rules);
        assert_eq!(low.advisories.len(), advisories, "{config}");
        assert_eq!(low.analysis_warning_count, 0);
        assert!(low.analysis_fallbacks.is_empty());
        let definite: Vec<_> = low.rules.iter().filter(|row| !shape(row)).collect();
        assert_eq!(definite.len(), 2);
        for row in definite {
            assert!(max.rules.contains(row), "{config}: {row:?}");
            assert!(row.message.contains("incompatible"));
        }
        let unknown: Vec<_> = low
            .rules
            .iter()
            .chain(&low.advisories)
            .filter(|row| shape(row))
            .collect();
        assert_eq!(
            unknown.len(),
            if config == "low-ignore.yml" { 0 } else { 2 }
        );
        for row in unknown {
            assert!(row.message.contains("unproven"));
            assert!(row.message.contains("max"));
            assert!(row.line > 0);
        }
    }
}

#[test]
fn payload_compatibility_mixed_applications_and_custom_messages_keep_both_channels() {
    let root = fixture("fixture");
    let mixed = run(&root, "mixed.yml", None);
    assert_eq!(mixed.rules.len(), 3, "{:?}", mixed.rules);
    assert_eq!(mixed.advisories.len(), 1);
    assert_eq!(
        mixed.advisories[0].target.as_deref(),
        Some("nested-compatible")
    );
    let custom = run(&root, "custom-message.yml", None);
    assert_eq!(custom.rules.len(), 2);
    assert_eq!(custom.advisories.len(), 2);
    for row in custom.rules.iter().chain(&custom.advisories) {
        assert!(row.message.starts_with("Payload comparison:"), "{row:?}");
        assert!(row.message.contains(if shape(row) {
            "unproven"
        } else {
            "incompatible"
        }));
    }
}

#[test]
fn payload_compatibility_uncertainty_only_does_not_become_definite() {
    let root = fixture("uncertainty-only/fixture");
    for config in ["max.yml", "default.yml"] {
        let max = run(&root, config, None);
        assert!(max.rules.is_empty() && max.advisories.is_empty());
    }
    for (config, findings, advisories) in [
        ("low-report.yml", 1, 0),
        ("low-advise.yml", 0, 1),
        ("low-ignore.yml", 0, 0),
    ] {
        let low = run(&root, config, None);
        assert_eq!(low.rules.len(), findings);
        assert_eq!(low.advisories.len(), advisories);
        assert!(low.rules.iter().chain(&low.advisories).all(shape));
    }
}

#[test]
fn payload_compatibility_ignore_cannot_hide_consumer_integrity_scope_or_pointer_errors() {
    let root = fixture("hard-inputs/fixture");
    for config in [
        "duplicate-low-ignore.yml",
        "precision-low-ignore.yml",
        "missing-low-ignore.yml",
        "excluded-low-ignore.yml",
        "pointer-low-ignore.yml",
    ] {
        let report = run(&root, config, None);
        assert_eq!(report.rules.len(), 1, "{config}: {:?}", report.rules);
        assert!(report.advisories.is_empty());
        assert!(!shape(&report.rules[0]));
        assert!(report.rules[0].message.contains("unproven"));
        assert!(report.rules[0].message.contains("consumer"));
    }
    let compatible = run(&root, "compatible-low-ignore.yml", None);
    assert!(compatible.rules.is_empty() && compatible.advisories.is_empty());
}
