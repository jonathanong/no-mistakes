use super::*;
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::config::v2::{Effort, Unanalyzable};

fn prepared(effort: Effort, names: &[&str]) -> (PathBuf, NoMistakesConfig, CheckFactMap) {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/no-nextjs-caching/empty-facts"),
    );
    let files = names.iter().map(|name| root.join(name)).collect::<Vec<_>>();
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("normal.yml"))).unwrap();
    config.effort_override = Some(effort);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        &root,
        &mut config,
        &files,
        &sources,
    )
    .unwrap();
    let mut plan = CheckFactPlan {
        source: true,
        ..Default::default()
    };
    configure_effort_plan(&config, &mut plan).unwrap();
    let facts = collect_check_facts(&root, files, plan);
    (root, config, facts)
}

#[test]
fn complete_empty_facts_skip_aggregate_scope_checks() {
    for effort in [Effort::Max, Effort::Low] {
        let (root, config, facts) = prepared(effort, &["empty.ts", "disabled.ts"]);
        for defer in [false, true] {
            take_scope_checks();
            let report =
                diagnostics_with_facts_for_aggregate(&root, &config, &facts, None, defer).unwrap();
            assert!(report.findings.is_empty() && report.advisories.is_empty());
            // Count actual scope evaluations, not a duplicate eligibility predicate.
            assert_eq!(take_scope_checks(), 0, "{effort:?}, deferred={defer}");
        }
        let mut invalid = config;
        invalid.rules[0].include = vec!["[".into()];
        let error = diagnostics_with_facts_for_aggregate(&root, &invalid, &facts, None, false)
            .err()
            .unwrap();
        assert!(error.to_string().contains("invalid glob"));
    }
}

#[test]
fn empty_fact_fast_path_preserves_missing_source_and_projection_errors() {
    for effort in [Effort::Max, Effort::Low] {
        for missing_source in [false, true] {
            let (root, config, mut facts) = prepared(effort, &["empty.ts"]);
            let file =
                std::sync::Arc::get_mut(facts.ts.get_mut(&root.join("empty.ts")).unwrap()).unwrap();
            if missing_source {
                file.source = None;
            } else if effort == Effort::Low {
                file.nextjs_caching_low = None;
            } else {
                file.nextjs_caching = None;
            }
            for defer in [false, true] {
                let error =
                    diagnostics_with_facts_for_aggregate(&root, &config, &facts, None, defer)
                        .err()
                        .unwrap();
                let expected = if missing_source {
                    "requires source facts"
                } else {
                    "requires Next.js caching facts"
                };
                assert!(error.to_string().contains(expected), "{error}");
            }
        }
    }
}

#[test]
fn nonempty_low_unanalyzable_facts_keep_scope_checks_even_when_ignored() {
    for policy in [Unanalyzable::Ignore, Unanalyzable::Advise] {
        let (root, mut config, facts) = prepared(Effort::Low, &["uncertain.ts"]);
        config.rules[0].options["unanalyzable"] = serde_yaml::to_value(policy).unwrap();
        let files = facts.files().to_vec();
        let sources = crate::codebase::rules::source_store_for_files(&files);
        crate::codebase::rules::analysis_policy::prepare_rule_analysis(
            &root,
            &mut config,
            &files,
            &sources,
        )
        .unwrap();
        assert!(!facts
            .ts
            .values()
            .next()
            .unwrap()
            .nextjs_caching_low
            .as_ref()
            .unwrap()
            .is_empty());
        for defer in [false, true] {
            take_scope_checks();
            let report =
                diagnostics_with_facts_for_aggregate(&root, &config, &facts, None, defer).unwrap();
            assert_eq!(take_scope_checks(), 1);
            assert!(report.findings.is_empty());
            assert_eq!(
                report.advisories.len(),
                usize::from(policy == Unanalyzable::Advise)
            );
        }
    }
}
