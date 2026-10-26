use super::*;
use crate::codebase::rules::analysis_policy::{prepare_rule_analysis, AnalysisDiagnostics};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::{schema::RuleDef, schema::RuleScope, Effort, Unanalyzable};
use std::sync::Arc;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/effort-low"),
    )
}

fn inputs(
    effort: Option<Effort>,
    policy: Unanalyzable,
) -> (NoMistakesConfig, Vec<PathBuf>, Arc<SourceStore>) {
    let root = root();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = super::super::source_store_for_files(&files);
    let mut config = NoMistakesConfig {
        effort,
        unanalyzable: Some(policy),
        ..Default::default()
    };
    config.rules.push(RuleDef {
        rule: RULE_ID.into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(&std::fs::read_to_string(root.join("options.yml")).unwrap())
            .unwrap(),
        ..Default::default()
    });
    prepare_rule_analysis(&root, &mut config, &files, &sources).unwrap();
    (config, files, sources)
}

fn run(
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    deferred: bool,
) -> AnalysisDiagnostics {
    diagnostics_with_files_sources_and_facts(&root(), config, files, sources, None, deferred)
        .unwrap()
}

#[test]
fn low_routes_only_unsupported_extractions_through_the_policy() {
    for (policy, failures, advisories) in [
        (Unanalyzable::Report, 3, 0),
        (Unanalyzable::Advise, 1, 2),
        (Unanalyzable::Ignore, 1, 0),
    ] {
        let (config, files, sources) = inputs(Some(Effort::Low), policy);
        let output = run(&config, &files, &sources, false);
        assert_eq!(output.findings.len(), failures);
        assert_eq!(output.advisories.len(), advisories);
        // The legacy Vec contract must not promote non-failing advice to failures.
        // Both-channel consumers use the diagnostics entry point exercised above.
        assert_eq!(
            check_with_files_sources_and_facts(&root(), &config, &files, &sources, None).unwrap(),
            output.findings
        );
        assert!(output.findings.iter().any(|row| row
            .message
            .contains("all contains `billing` but partial does not")));
        assert!(!output
            .findings
            .iter()
            .chain(&output.advisories)
            .any(|row| row.message.contains("minSize")));
        assert_eq!(
            config
                .effective_rule_analysis(&config.rules[0])
                .unwrap()
                .effort,
            Effort::Low
        );
        assert!(config.analysis_warnings.is_empty());
    }
}

#[test]
fn low_avoids_extraction_and_call_site_demand_with_max_as_positive_control() {
    for (effort, extractions, demand) in [(Effort::Low, 3, 0), (Effort::Max, 5, 1)] {
        let (config, files, sources) = inputs(Some(effort), Unanalyzable::Report);
        let observer = crate::diagnostics::InvocationObserver::new(true);
        crate::diagnostics::with_observer(Some(observer.clone()), || {
            run(&config, &files, &sources, true);
        });
        assert_eq!(
            observer.snapshot().work["finite_set.extractions"],
            extractions
        );
        assert_eq!(
            try_required_call_site_fact_files(&root(), &config)
                .unwrap()
                .len(),
            demand
        );
        assert_eq!(
            required_call_site_fact_files(&root(), &config).len(),
            demand
        );
    }
}

#[test]
fn default_and_explicit_max_outputs_match_and_supported_low_results_match_max() {
    let (default, files, sources) = inputs(None, Unanalyzable::Report);
    let (max, _, _) = inputs(Some(Effort::Max), Unanalyzable::Report);
    assert_eq!(
        run(&default, &files, &sources, true).findings,
        run(&max, &files, &sources, true).findings
    );
    let (mut low, _, _) = inputs(Some(Effort::Low), Unanalyzable::Report);
    let mut max = max;
    for config in [&mut low, &mut max] {
        config.rules[0].options["sets"]
            .as_sequence_mut()
            .unwrap()
            .truncate(3);
    }
    assert_eq!(
        run(&low, &files, &sources, true).findings,
        run(&max, &files, &sources, true).findings
    );
    assert_eq!(
        check_with_files_sources_and_facts(&root(), &max, &files, &sources, None).unwrap(),
        run(&max, &files, &sources, true).findings
    );
}

#[test]
fn suppressed_unknown_sets_never_participate_in_comparisons() {
    for file in ["src/suppressed.ts", "src/line-suppressed.ts"] {
        for policy in [
            Unanalyzable::Report,
            Unanalyzable::Advise,
            Unanalyzable::Ignore,
        ] {
            let (mut config, files, sources) = inputs(Some(Effort::Low), policy);
            let sets = config.rules[0].options["sets"].as_sequence_mut().unwrap();
            sets.truncate(4);
            sets[3]["file"] = file.into();
            // If skipped extraction were treated as empty, this would invent a mismatch.
            let output = run(&config, &files, &sources, false);
            assert_eq!(output.findings.len(), 1);
            assert!(output.advisories.is_empty());
        }
    }
}

#[test]
fn unsupported_shapes_still_require_readable_sources_and_valid_call_targets() {
    let (mut config, files, sources) = inputs(Some(Effort::Low), Unanalyzable::Ignore);
    config.rules[0].options["sets"].as_sequence_mut().unwrap()[3]["file"] = "missing.ts".into();
    let error =
        diagnostics_with_files_sources_and_facts(&root(), &config, &files, &sources, None, true)
            .err()
            .unwrap();
    assert!(error.to_string().contains("missing.ts"));

    let (mut config, files, sources) = inputs(Some(Effort::Low), Unanalyzable::Ignore);
    config.rules[0].options["sets"].as_sequence_mut().unwrap()[4]["target"] = "".into();
    let output = run(&config, &files, &sources, true);
    assert_eq!(output.findings.len(), 2);
    assert!(output
        .findings
        .iter()
        .any(|row| row.message.contains("requires a non-empty target")));
}

#[test]
fn each_other_extractor_kind_is_explicitly_unknown_including_empty_names() {
    let kinds = [
        "ts-string-union",
        "ts-const-object-keys",
        "ts-const-object-property",
        "ts-array-literal",
        "ts-const-array-property",
        "ts-call-first-string-argument",
        "markdown-table-code-cells",
        "sql-enum",
        "unknown-kind",
    ];
    for kind in kinds {
        let (mut config, files, sources) = inputs(Some(Effort::Low), Unanalyzable::Advise);
        let sets = config.rules[0].options["sets"].as_sequence_mut().unwrap();
        sets.truncate(4);
        sets[3]["kind"] = kind.into();
        let output = run(&config, &files, &sources, true);
        assert_eq!(output.advisories.len(), 1, "{kind}");
        assert!(output.advisories[0].message.contains(kind));
        config.rules[0].options["sets"].as_sequence_mut().unwrap()[3]["name"] = "".into();
        assert!(run(&config, &files, &sources, true).advisories.is_empty());
    }
}

#[test]
fn mixed_applications_keep_their_own_effort_and_demand() {
    let (mut config, files, sources) = inputs(Some(Effort::Low), Unanalyzable::Advise);
    let mut max = config.rules[0].clone();
    max.name = Some("maximum".into());
    max.options["effort"] = "max".into();
    config.rules.push(max);
    prepare_rule_analysis(&root(), &mut config, &files, &sources).unwrap();
    assert_eq!(
        try_required_call_site_fact_files(&root(), &config).unwrap(),
        vec![root().join("src/registry.ts")]
    );
    let output = run(&config, &files, &sources, true);
    assert_eq!(output.advisories.len(), 2);
    assert!(output
        .findings
        .iter()
        .any(|row| row.message.contains("minSize is 99")));
    assert_eq!(
        output
            .findings
            .iter()
            .filter(|row| row
                .message
                .contains("all contains `billing` but partial does not"))
            .count(),
        1
    );
}
