use super::*;
use crate::config::v2::{EffectiveRuleAnalysis, Unanalyzable};

const ADOPTERS: &[RuleEffortSupport] = &[RuleEffortSupport {
    rule: "test-effort-adopter",
    partners: &["test-partner"],
    partner_scope: file_only_scope,
}];
fn file_only_scope(_: &RuleDef, partner: &RuleDef) -> bool {
    partner.tests == Default::default()
        && partner.options.as_mapping().is_none_or(|options| {
            options.keys().all(|key| {
                matches!(
                    key.as_str(),
                    Some("include" | "exclude" | "effort" | "unanalyzable" | "unanalyzableSql")
                )
            })
        })
}
fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/rule-analysis-policy/fixture")
}
fn prepare(name: &str, support: &[RuleEffortSupport]) -> NoMistakesConfig {
    let root = fixture();
    let mut config = crate::config::v2::load_v2_config(&root, Some(Path::new(name))).unwrap();
    let mut files = vec![root.join("src/covered.ts"), root.join("src/outside.ts")];
    if name == "suppressed.yml" {
        files.extend(
            [
                "suppressed-file.ts",
                "suppressed-line.ts",
                "suppressed-next.ts",
            ]
            .map(|name| root.join("src").join(name)),
        );
    }
    let sources = super::super::source_store_for_files(&files);
    prepare_rule_analysis_with_support(&root, &mut config, &files, &sources, support).unwrap();
    config
}

const PARTNER_NOTE: &str = "test-partner: requested effort low; effective effort max because this rule has no low implementation";
const ADOPTER_FALLBACK: &str = "test-effort-adopter: effort low requires enabled partner `test-partner` covering this application's scope; falling back to max";
fn assert_partner_fallback(config: &NoMistakesConfig) {
    assert_eq!(
        config.analysis_fallbacks,
        vec![AnalysisFallback {
            rule: "test-effort-adopter".into(),
            partner: "test-partner".into(),
            requested: Effort::Low,
            effective: Effort::Max,
        }]
    );
}

#[test]
fn low_adopter_requires_union_of_enabled_partner_scopes() {
    let config = prepare("covered.yml", ADOPTERS);
    assert_eq!(
        config.effective_rule_analysis(&config.rules[0]).unwrap(),
        EffectiveRuleAnalysis {
            effort: Effort::Low,
            unanalyzable: Unanalyzable::Advise,
        }
    );
    assert_eq!(config.analysis_warnings, [PARTNER_NOTE, PARTNER_NOTE]);
    assert!(config.analysis_fallbacks.is_empty());
    for name in ["missing.yml", "outside.yml"] {
        let config = prepare(name, ADOPTERS);
        assert_eq!(
            config
                .effective_rule_analysis(&config.rules[0])
                .unwrap()
                .effort,
            Effort::Max
        );
        let expected = if name == "missing.yml" {
            vec![ADOPTER_FALLBACK]
        } else {
            vec![ADOPTER_FALLBACK, PARTNER_NOTE]
        };
        assert_eq!(config.analysis_warnings, expected, "{name}");
        assert_partner_fallback(&config);
    }
}

#[test]
fn non_adopters_report_requested_low_while_defaults_stay_quiet() {
    let config = prepare("covered.yml", &[]);
    assert_eq!(
        config
            .effective_rule_analysis(&config.rules[0])
            .unwrap()
            .effort,
        Effort::Max
    );
    assert_eq!(config.analysis_warnings, [
        "selected-application: requested effort low; effective effort max because this rule has no low implementation",
        PARTNER_NOTE,
        PARTNER_NOTE,
    ]);
    assert!(config.analysis_fallbacks.is_empty());
    let config = prepare("default.yml", ADOPTERS);
    assert!(config.analysis_warnings.is_empty());
    assert!(config.analysis_fallbacks.is_empty());
}

#[test]
fn partner_options_filters_and_project_roots_are_enforced() {
    let config = prepare("option-filters.yml", ADOPTERS);
    assert_eq!(
        config
            .effective_rule_analysis(&config.rules[0])
            .unwrap()
            .effort,
        Effort::Low
    );
    assert_eq!(config.analysis_warnings, [PARTNER_NOTE]);
    assert!(config.analysis_fallbacks.is_empty());
    let config = prepare("wrong-project.yml", ADOPTERS);
    assert_eq!(
        config
            .effective_rule_analysis(&config.rules[0])
            .unwrap()
            .effort,
        Effort::Max
    );
    assert_eq!(config.analysis_warnings, [ADOPTER_FALLBACK, PARTNER_NOTE]);
    assert_partner_fallback(&config);
}

#[path = "adopter_tests.rs"]
mod adopter_tests;

#[test]
fn unknown_semantic_partner_scope_and_suppressions_fall_back_before_facts() {
    for name in ["semantic-scope.yml", "suppressed.yml"] {
        let config = prepare(name, ADOPTERS);
        assert_eq!(
            config
                .effective_rule_analysis(&config.rules[0])
                .unwrap()
                .effort,
            Effort::Max
        );
        assert_eq!(config.analysis_warnings, [ADOPTER_FALLBACK, PARTNER_NOTE]);
        assert_partner_fallback(&config);
    }
}

#[test]
fn partner_validation_reads_sources_only_when_low_is_effective() {
    let root = fixture();
    let files = [root.join("src/covered.ts"), root.join("src/outside.ts")];
    for (name, override_effort, expected_reads) in [
        ("default.yml", None, 0),
        ("covered.yml", Some(Effort::Max), 0),
        ("covered.yml", None, 2),
    ] {
        let mut config = crate::config::v2::load_v2_config(&root, Some(Path::new(name))).unwrap();
        config.effort_override = override_effort;
        let sources = super::super::source_store_for_files(&files);
        prepare_rule_analysis_with_support(&root, &mut config, &files, &sources, ADOPTERS).unwrap();
        assert_eq!(sources.physical_read_count(), expected_reads, "{name}");
    }
}

#[test]
fn every_relevant_partner_suppression_form_disables_the_low_assumption() {
    let root = fixture();
    for name in [
        "suppressed-file.ts",
        "suppressed-line.ts",
        "suppressed-next.ts",
    ] {
        let mut config =
            crate::config::v2::load_v2_config(&root, Some(Path::new("suppressed.yml"))).unwrap();
        let files = [root.join("src").join(name)];
        let sources = super::super::source_store_for_files(&files);
        prepare_rule_analysis_with_support(&root, &mut config, &files, &sources, ADOPTERS).unwrap();
        assert_eq!(
            config
                .effective_rule_analysis(&config.rules[0])
                .unwrap()
                .effort,
            Effort::Max,
            "{name}"
        );
        assert_eq!(config.analysis_warnings, [ADOPTER_FALLBACK, PARTNER_NOTE]);
        assert_partner_fallback(&config);
        assert_eq!(sources.physical_read_count(), 1);
    }
}
