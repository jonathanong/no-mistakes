use super::{collect_check_facts, CheckFactPlan};
use crate::codebase::postgres::{
    assign_prepared_embedded_sql_options, configure_prepared_postgres_plan,
};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use crate::diagnostics::{with_observer, InvocationObserver};

fn config(efforts: &[&str]) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    for effort in efforts {
        config.rules.push(RuleDef {
            rule: "postgres-require-query-annotation".into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!(
                "effort: {effort}\nimportSpecifier: '@bench/db'\nexecutorNames: [read]"
            ))
            .unwrap(),
            ..Default::default()
        });
    }
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        std::path::Path::new("."),
        &mut config,
        &[],
        &crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(
            crate::codebase::ts_source::FileInventory::from_paths(&[]),
        )),
    )
    .unwrap();
    config
}

fn plan(config: &NoMistakesConfig) -> CheckFactPlan {
    let mut plan = CheckFactPlan::default();
    configure_prepared_postgres_plan(config, &mut plan).unwrap();
    assign_prepared_embedded_sql_options(config, &mut plan).unwrap();
    plan
}

#[test]
fn configured_low_annotation_skips_composition_in_the_real_shared_fact_pass() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join("append-literal-arguments.mts");
    let mut expected_spans = None;
    for efforts in [vec!["low"], vec!["max"], vec!["low", "max"]] {
        let plan = plan(&config(&efforts));
        let head_only = efforts == ["low"];
        assert_eq!(
            plan.embedded_sql_head_calls_only.len(),
            usize::from(head_only)
        );
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![path.clone()], plan)
        });
        let file = facts.ts.get(&path).unwrap();
        let embedded = &file.embedded_sql[0].1;
        assert!(!embedded.call_spans.is_empty());
        if let Some(spans) = &expected_spans {
            assert_eq!(&embedded.call_spans, spans);
        } else {
            expected_spans = Some(embedded.call_spans.clone());
        }
        let composed = observer
            .snapshot()
            .work
            .get("postgres.sql.variants_composed")
            .copied()
            .unwrap_or(0);
        if head_only {
            assert_eq!(
                composed, 0,
                "head-only collection must never compose SQL variants"
            );
            assert!(embedded.calls.iter().all(|call| call.variants.is_empty()));
            assert!(file.query_annotation.is_empty());
            assert!(file
                .query_annotation_heads
                .as_ref()
                .unwrap()
                .calls
                .values()
                .any(|proof| matches!(
                    proof,
                    crate::codebase::postgres::query_annotation::heads::HeadProof::Literal
                )));
        } else {
            assert!(
                composed > 0,
                "the full-demand control must exercise real composition work"
            );
            assert!(embedded.calls.iter().any(|call| !call.variants.is_empty()));
        }
    }
}

#[test]
fn different_profile_full_sql_demand_preserves_the_low_profile_fast_path() {
    let mut config = config(&["low"]);
    config.rules.push(RuleDef {
        rule: "postgres-explicit-columns".into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("importSpecifier: '@other/db'\nexecutorNames: [read]")
            .unwrap(),
        ..Default::default()
    });
    let plan = plan(&config);
    assert_eq!(plan.embedded_sql_options.len(), 2);
    assert_eq!(plan.embedded_sql_head_calls_only.len(), 1);
}
