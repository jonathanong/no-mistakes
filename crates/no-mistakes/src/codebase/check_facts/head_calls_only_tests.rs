use super::{collect_check_facts, CheckFactPlan};
use crate::codebase::postgres::{
    assign_prepared_embedded_sql_options, configure_prepared_postgres_plan,
};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use crate::diagnostics::{with_observer, InvocationObserver};

#[path = "head_calls_only_tests/async_return_fact_tests.rs"]
mod async_return_fact_tests;
#[path = "head_calls_only_tests/callback_head_tests.rs"]
mod callback_head_tests;
#[path = "head_calls_only_tests/callback_more_tests.rs"]
mod callback_more_tests;
#[path = "head_calls_only_tests/finite_comment_fact_tests.rs"]
mod finite_comment_fact_tests;
#[path = "head_calls_only_tests/namespace_fact_tests.rs"]
mod namespace_fact_tests;
#[path = "head_calls_only_tests/returned_param_fact_tests.rs"]
mod returned_param_fact_tests;
#[path = "head_calls_only_tests/returned_prefix_fact_tests.rs"]
mod returned_prefix_fact_tests;
#[path = "head_calls_only_tests/second_effect_fact_tests.rs"]
mod second_effect_fact_tests;
#[path = "head_calls_only_tests/text_field_fact_tests.rs"]
mod text_field_fact_tests;

#[path = "head_calls_only_tests/support.rs"]
mod support;
use support::{config, plan};

#[test]
fn configured_low_annotation_skips_composition_in_the_real_shared_fact_pass() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join("append-literal-arguments.mts");
    let mut expected_spans = None;
    for efforts in [vec!["low"], vec!["max"], vec!["low", "max"]] {
        let plan = plan(root, &config(&efforts));
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
    let plan = plan(std::path::Path::new("."), &config);
    assert_eq!(plan.embedded_sql_options.len(), 2);
    assert_eq!(plan.embedded_sql_head_calls_only.len(), 1);
}

#[test]
fn direct_imported_helper_heads_project_from_prepared_source_facts() {
    use crate::codebase::postgres::query_annotation::heads::HeadProof;
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("imported-helper-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let caller = root.join("src/query.mts");
    let target = root.join("src/helpers.mts");
    let reexport = root.join("src/reexport.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(
            root,
            vec![caller.clone(), target, reexport],
            plan(root, &config(&["low"])),
        )
    });
    let file = facts.ts.get(&caller).expect("caller prepared");
    let heads = file.query_annotation_heads.as_ref().expect("low heads");
    let statuses = file.embedded_sql[0]
        .1
        .calls
        .iter()
        .zip(&file.embedded_sql[0].1.call_spans)
        .map(|(call, span)| (call.line, heads.calls.get(span).copied()))
        .collect::<Vec<_>>();
    assert_eq!(
        statuses,
        vec![
            (8, Some(HeadProof::Literal)),
            (9, Some(HeadProof::Literal)),
            (10, Some(HeadProof::Literal)),
            (11, Some(HeadProof::Missing)),
            (12, Some(HeadProof::Unproven)),
            (13, Some(HeadProof::Unproven)),
            // Both explicit return paths now carry a fixed named head.
            (14, Some(HeadProof::Literal)),
            (15, Some(HeadProof::Unproven)),
            (16, Some(HeadProof::Unproven)),
            (17, Some(HeadProof::Unproven)),
            (20, Some(HeadProof::Unproven)),
            (22, Some(HeadProof::Unproven)),
            (23, Some(HeadProof::Unproven)),
            (24, Some(HeadProof::Unproven)),
            (25, Some(HeadProof::Literal)),
        ]
    );
    assert!(file.query_annotation.is_empty());
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("postgres.sql.variants_composed")
            .copied()
            .unwrap_or(0),
        0
    );
}

#[path = "head_calls_only_tests/variant_parity_tests.rs"]
mod variant_parity;

#[path = "head_calls_only_tests/captured_effect_fact_tests.rs"]
mod captured_effect_facts;
#[path = "head_calls_only_tests/effect_composition_fact_tests.rs"]
mod effect_composition_facts;
#[path = "head_calls_only_tests/effect_default_fact_tests.rs"]
mod effect_default_facts;
#[path = "head_calls_only_tests/effect_fact_tests.rs"]
mod effect_facts;

#[path = "head_calls_only_tests/value_fact_tests.rs"]
mod value_facts;

#[path = "head_calls_only_tests/record_fact_tests.rs"]
mod record_facts;

#[path = "head_calls_only_tests/branch_assignment_fact_tests.rs"]
mod branch_assignment_facts;
#[path = "head_calls_only_tests/builder_fact_tests.rs"]
mod builder_facts;
#[path = "head_calls_only_tests/captured_fact_tests.rs"]
mod captured_facts;
#[path = "head_calls_only_tests/relative_pending_tests.rs"]
mod relative_pending;

#[path = "head_calls_only_tests/callback_export_payload_tests.rs"]
mod callback_export_payload_tests;
