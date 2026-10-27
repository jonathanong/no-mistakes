use super::*;
use crate::config::v2::{Effort, Unanalyzable};

// Output cases moved to cli_rra_snapshot_require_storybook_stories.

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
