use super::{collect_check_facts, config, plan};
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn closed_capture_uses_low_heads_with_zero_evaluator_work_in_both_demands() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("captured-prefix");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join("query.mts");
    let mut expected = None;
    for efforts in [vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let plan = plan(root, &config(&efforts));
        assert_eq!(
            plan.embedded_sql_head_calls_only.len(),
            usize::from(efforts == ["low"])
        );
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![path.clone()], plan)
        });
        let file = facts.ts.get(&path).unwrap();
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let source = std::fs::read_to_string(&path).unwrap();
        let decisions = file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| {
                let line = source[..span.0 as usize].lines().count();
                (line, heads.accepts(*span))
            })
            .collect::<Vec<_>>();
        assert!(decisions.iter().any(|(_, accepted)| *accepted));
        if let Some(prior) = &expected {
            assert_eq!(&decisions, prior);
        } else {
            expected = Some(decisions);
        }
        let composed = observer
            .snapshot()
            .work
            .get("postgres.sql.variants_composed")
            .copied()
            .unwrap_or(0);
        if efforts == ["low"] {
            assert_eq!(composed, 0);
            assert!(file.query_annotation.is_empty());
        }
        // Full demand still abstains on this captured runtime suffix; its
        // inability to compose SQL must not change the independent low head.
    }
}

#[test]
fn captured_low_prefix_does_not_expand_max_legacy_append_proof() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("captured-prefix");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join("query.mts");
    let legacy = |efforts: &[&str]| {
        let facts = collect_check_facts(root, vec![path.clone()], plan(root, &config(efforts)));
        let file = facts.ts.get(&path).unwrap();
        file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| {
                file.query_annotation_heads
                    .as_deref()
                    .is_some_and(|heads| heads.accepts_trusted_append(*span))
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(legacy(&["max"]), legacy(&["low", "max"]));
}
