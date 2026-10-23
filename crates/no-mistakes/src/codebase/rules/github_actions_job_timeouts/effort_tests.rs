use super::*;
use crate::codebase::rules::analysis_policy::prepare_rule_analysis;
use crate::config::v2::Effort;

fn prepared(name: &str) -> (PathBuf, NoMistakesConfig, Vec<PathBuf>) {
    let root = super::tests::fixture("effort-policy");
    let mut config = crate::config::v2::load_v2_config(&root, Some(Path::new(name))).unwrap();
    let files = [
        "ci",
        "other",
        "suppressed-file",
        "suppressed-line",
        "suppressed-next",
    ]
    .map(|name| root.join(format!(".github/workflows/{name}.yml")))
    .to_vec();
    let sources = super::super::source_store_for_files(&files);
    prepare_rule_analysis(&root, &mut config, &files, &sources).unwrap();
    (root, config, files)
}

fn job_findings(name: &str) -> (NoMistakesConfig, Vec<RuleFinding>) {
    let (root, config, files) = prepared(name);
    let findings = check_with_files(&root, &config, &files).unwrap();
    (config, findings)
}

#[test]
fn certified_low_omits_only_partner_owned_steps_and_keeps_job_bounds() {
    let (config, low) = job_findings("low.yml");
    let (_, max) = job_findings("max.yml");
    assert!(
        config.analysis_warnings.is_empty(),
        "{:?}",
        config.analysis_warnings
    );
    assert!(config.analysis_fallbacks.is_empty());
    assert_eq!(
        config
            .effective_rule_analysis(&config.rules[0])
            .unwrap()
            .effort,
        Effort::Low
    );
    assert_eq!(max.len(), low.len() + 2, "max={max:?}; low={low:?}");
    assert!(low.iter().all(|finding| max.contains(finding)));
    for needle in [
        "metadata-only",
        "tighter",
        "step timeout sum",
        "no timeout-minutes",
        "no supported literal upper bound",
    ] {
        assert!(
            low.iter().any(|finding| finding.message.contains(needle)),
            "{needle}: {low:?}"
        );
    }
    assert!(!low
        .iter()
        .any(|finding| finding.message.contains("step \"dynamic\"")));
    assert!(!low
        .iter()
        .any(|finding| finding.message.contains("step \"over\"")));
    let (root, config, files) = prepared("low.yml");
    let partner =
        super::super::github_actions_step_timeouts::check_with_files(&root, &config, &files)
            .unwrap();
    assert!(partner
        .iter()
        .any(|finding| finding.message.contains("over the 5-minute cap")));
    assert!(partner
        .iter()
        .any(|finding| finding.message.contains("positive literal number")));
}

#[test]
fn missing_disabled_narrowed_allowed_and_suppressed_partners_fall_back() {
    for name in [
        "missing",
        "disabled",
        "narrowed",
        "excluded",
        "allowed",
        "unknown",
        "unsupported-tests",
        "suppressed-file",
        "suppressed-line",
        "suppressed-next",
    ] {
        let (config, findings) = job_findings(&format!("{name}.yml"));
        assert_eq!(
            config
                .effective_rule_analysis(&config.rules[0])
                .unwrap()
                .effort,
            Effort::Max,
            "{name}"
        );
        assert_eq!(
            config.analysis_fallbacks.len(),
            1,
            "{name}: {:?}",
            config.analysis_warnings
        );
        assert!(config
            .analysis_warnings
            .iter()
            .any(|warning| warning.contains("github-actions-step-timeouts")
                && warning.contains("falling back to max")));
        assert!(
            findings
                .iter()
                .any(|finding| finding.message.contains("step \"over\"")),
            "{name}: {findings:?}"
        );
    }
}

#[test]
fn all_certified_caps_are_combined_conservatively_and_defaults_are_clamped() {
    let (_, max) = job_findings("max.yml");
    for name in [
        "multiple.yml",
        "default-cap.yml",
        "clamped-cap.yml",
        "default-options.yml",
    ] {
        let (config, findings) = job_findings(name);
        assert!(config.analysis_fallbacks.is_empty(), "{name}");
        assert!(
            findings.iter().all(|finding| max.contains(finding)),
            "{name}"
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.message.contains("step \"over\"")),
            "{name}"
        );
    }
    let (_, low) = job_findings("low.yml");
    let (_, included) = job_findings("options-include.yml");
    assert_eq!(included, low);
    let (_, zero) = job_findings("zero-cap.yml");
    assert!(!zero
        .iter()
        .any(|finding| finding.message.contains("step \"tighter\"")));
}

#[test]
fn explicit_max_and_unprepared_direct_configs_preserve_existing_findings() {
    let (_, max) = job_findings("max.yml");
    assert_eq!(job_findings("explicit-max.yml").1, max);
    let (root, _, files) = prepared("low.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(Path::new("low.yml"))).unwrap();
    assert_eq!(check_with_files(&root, &config, &files).unwrap(), max);
}

#[test]
fn full_check_prepares_partner_coverage_and_honors_effort_overrides() {
    let saved =
        crate::test_support::materialize_saved_fixture(&super::tests::fixture("effort-policy"));
    let report = |name: &str, effort| {
        crate::check_runner::run_all_with_options(
            saved.path().to_path_buf(),
            Some(name.into()),
            None,
            false,
            false,
            crate::check_runner::AnalysisOptions {
                effort,
                include_sql_verdicts: false,
            },
        )
        .unwrap()
    };
    let low = report("low.yml", None);
    let max = report("low.yml", Some(Effort::Max));
    let public =
        super::super::run_filesystem_rules(saved.path(), Some(&saved.path().join("low.yml")))
            .unwrap();
    // The standalone phase has no prepared policy and conservatively retains max.
    assert_eq!(public, max.rules);
    assert!(low.analysis_fallbacks.is_empty());
    assert!(low.rules.iter().all(|finding| max.rules.contains(finding)));
    assert_eq!(max.rules.len(), low.rules.len() + 2);
    assert_eq!(report("explicit-max.yml", None).rules, max.rules);
    for name in ["missing.yml", "narrowed.yml", "suppressed-next.yml"] {
        let fallback = report(name, None);
        assert_eq!(fallback.analysis_fallbacks.len(), 1, "{name}");
        assert!(
            fallback
                .rules
                .iter()
                .any(|finding| finding.message.contains("step \"over\"")),
            "{name}"
        );
    }
}
