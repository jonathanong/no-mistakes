use super::*;
use crate::config::v2::{Effort, Unanalyzable};

fn run(
    effort: Option<Effort>,
    policy: Unanalyzable,
    transitive: bool,
) -> crate::codebase::rules::analysis_policy::AnalysisDiagnostics {
    let root = fixture("effort");
    let name = if effort == Some(Effort::Low) {
        ".no-mistakes.low.yml"
    } else {
        ".no-mistakes.max.yml"
    };
    let mut config = crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap();
    config.effort_override = effort;
    if effort.is_none() {
        config.effort = None;
    }
    config.unanalyzable = Some(policy);
    if transitive {
        config.rules.truncate(1);
        config.rules[0].options =
            serde_yaml::from_str("{ include: [Child.tsx], stories: ['*.stories.tsx'] }").unwrap();
    }
    run_config(&root, config)
}

fn run_config(
    root: &std::path::Path,
    mut config: NoMistakesConfig,
) -> crate::codebase::rules::analysis_policy::AnalysisDiagnostics {
    let files = crate::codebase::ts_source::discover_files(root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        root,
        &mut config,
        &files,
        &sources,
    )
    .unwrap();
    let mut plan = CheckFactPlan {
        react: true,
        symbols: true,
        storybook: true,
        source: true,
        ..Default::default()
    };
    configure_effort_plan(&config, &mut plan).unwrap();
    let facts = crate::codebase::check_facts::collect_check_facts(root, files, plan);
    runner::check_diagnostics_with_resolver(
        root,
        &config,
        &facts,
        &empty_resolver(root),
        None,
        false,
        &sources,
    )
    .unwrap()
}

#[test]
fn low_preserves_definite_findings_and_routes_only_residuals() {
    let max = run(Some(Effort::Max), Unanalyzable::Report, false);
    assert_eq!(max.findings.len(), 3, "{:?}", max.findings);
    assert_eq!(
        max.findings,
        run(None, Unanalyzable::Report, false).findings
    );
    for policy in [
        Unanalyzable::Report,
        Unanalyzable::Advise,
        Unanalyzable::Ignore,
    ] {
        let low = run(Some(Effort::Low), policy, false);
        let definite: Vec<_> = low
            .findings
            .iter()
            .filter(|finding| finding.target.as_deref() != Some("unanalyzable"))
            .collect();
        assert_eq!(definite.len(), 1, "{:?}", low.findings);
        assert!(definite
            .iter()
            .all(|finding| max.findings.contains(finding)));
        assert_eq!(
            low.findings.len(),
            if policy == Unanalyzable::Report { 3 } else { 1 }
        );
        assert_eq!(
            low.advisories.len(),
            if policy == Unanalyzable::Advise { 2 } else { 0 }
        );
        assert!(max.findings.len() >= low.findings.len());
        for finding in low.findings.iter().chain(&low.advisories) {
            assert!(!finding.file.contains("Disabled"), "{finding:?}");
            if finding.target.as_deref() == Some("unanalyzable") {
                assert!(finding.message.contains("effort low"));
                assert!(finding
                    .message
                    .contains("Import and use the component directly"));
            }
        }
    }
}

#[test]
fn transitive_coverage_is_uncertainty_rather_than_a_definite_low_violation() {
    assert!(run(Some(Effort::Max), Unanalyzable::Report, true)
        .findings
        .is_empty());
    for policy in [Unanalyzable::Advise, Unanalyzable::Ignore] {
        let low = run(Some(Effort::Low), policy, true);
        assert!(low.findings.is_empty(), "{:?}", low.findings);
        assert_eq!(
            low.advisories.len(),
            if policy == Unanalyzable::Advise { 1 } else { 0 }
        );
    }
}

#[test]
fn low_skips_transitive_graph_and_dynamic_boundary_work() {
    for effort in [Effort::Low, Effort::Max] {
        let observer = crate::diagnostics::InvocationObserver::new(true);
        crate::diagnostics::with_observer(Some(observer.clone()), || {
            run(Some(effort), Unanalyzable::Ignore, false);
        });
        let work = observer.snapshot().work;
        for counter in [
            "storybook.transitive_coverage.scans",
            "storybook.dynamic_boundary.scans",
        ] {
            assert_eq!(
                work.get(counter).copied().unwrap_or(0),
                if effort == Effort::Low { 0 } else { 2 }
            );
        }
    }
}

#[test]
fn standalone_rule_honors_freshly_loaded_low_policy() {
    let root = fixture("effort");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.low-ignore.yml")))
            .unwrap();
    assert!(config
        .rules
        .iter()
        .all(|rule| rule.resolved_analysis.is_none()));
    assert_eq!(
        check(&root, &config, None).unwrap(),
        run(Some(Effort::Low), Unanalyzable::Ignore, false).findings
    );
    assert!(config
        .rules
        .iter()
        .all(|rule| rule.resolved_analysis.is_none()));
    assert_eq!(
        crate::codebase::rules::run_check(
            &root,
            Some(&root.join(".no-mistakes.low-ignore.yml")),
            None
        )
        .unwrap()
        .len(),
        1
    );
}

#[test]
fn implicit_boundary_exemptions_are_residuals_and_colocated_tests_still_clear() {
    let root = fixture("effort/plain");
    for effort in [Effort::Low, Effort::Max] {
        let mut cfg = config("includeAllReactNamedExports: true");
        cfg.effort = Some(effort);
        cfg.unanalyzable = Some(Unanalyzable::Advise);
        let result = run_config(&root, cfg);
        assert!(result.findings.is_empty());
        assert_eq!(
            result.advisories.len(),
            if effort == Effort::Low { 1 } else { 0 }
        );
        let mut cfg = config("{ include: ['*.tsx'], allow_colocated_tests: true }");
        cfg.effort = Some(effort);
        let result = run_config(&root, cfg);
        assert!(result.findings.is_empty());
        assert!(result.advisories.is_empty());
    }
}

#[test]
fn effort_demand_skips_dynamic_facts_only_for_low_only_storybook() {
    let root = fixture("effort");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    for mode in ["low", "max", "mixed", "other-consumer"] {
        let mut cfg =
            crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.low.yml")))
                .unwrap();
        if mode == "max" {
            cfg.effort = Some(Effort::Max);
        }
        if mode == "mixed" {
            cfg.rules[0]
                .options
                .as_mapping_mut()
                .unwrap()
                .insert("effort".into(), "max".into());
        }
        crate::codebase::rules::analysis_policy::prepare_rule_analysis(
            &root, &mut cfg, &files, &sources,
        )
        .unwrap();
        let mut plan = CheckFactPlan {
            dynamic_imports: mode == "other-consumer",
            ..Default::default()
        };
        configure_effort_plan(&cfg, &mut plan).unwrap();
        let facts = crate::codebase::check_facts::collect_check_facts(&root, files.clone(), plan);
        assert!(
            facts
                .ts
                .values()
                .all(|file| file.dynamic_imports.is_some() == (mode != "low")),
            "{mode}"
        );
    }
    let mut plan = CheckFactPlan {
        dynamic_imports: true,
        ..Default::default()
    };
    configure_effort_plan(&NoMistakesConfig::default(), &mut plan).unwrap();
    assert!(plan.dynamic_imports);
}
