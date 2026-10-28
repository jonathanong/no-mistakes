use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

fn saved_fixture() -> tempfile::TempDir {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    crate::test_support::materialize_saved_fixture(&saved)
}

fn decisions(root: &std::path::Path) -> Vec<HeadProof> {
    let path = root.join("two-arm-assignment.mts");
    let facts = collect_check_facts(root, vec![path.clone()], plan(root, &config(&["low"])));
    let file = facts.ts.get(&path).unwrap();
    file.embedded_sql[0]
        .1
        .call_spans
        .iter()
        .map(|span| file.query_annotation_heads.as_ref().unwrap().calls[span])
        .collect()
}

#[test]
fn branch_assignment_head_only_request_does_zero_evaluator_work() {
    let fixture = saved_fixture();
    let root = fixture.path();
    let path = root.join("two-arm-assignment.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(root, vec![path.clone()], plan(root, &config(&["low"])))
    });
    let file = facts.ts.get(&path).unwrap();
    let spans = &file.embedded_sql[0].1.call_spans;
    let heads = file.query_annotation_heads.as_ref().unwrap();
    assert_eq!(spans.len(), 16);
    assert_eq!(heads.calls[&spans[0]], HeadProof::Literal);
    assert_eq!(heads.calls[&spans[2]], HeadProof::Missing);
    assert_eq!(heads.calls[&spans[3]], HeadProof::Unproven);
    assert_eq!(heads.calls[&spans[4]], HeadProof::Unproven);
    assert!(file.query_annotation.is_empty());
    for counter in [
        "postgres.annotation.head_syntax_visits",
        "postgres.annotation.head_binding_indexes",
    ] {
        assert_eq!(observer.snapshot().work.get(counter).copied(), Some(1));
    }
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

#[test]
fn branch_assignment_heads_are_stable_when_max_demand_is_added() {
    let fixture = saved_fixture();
    let root = fixture.path();
    let low = decisions(root);
    let path = root.join("two-arm-assignment.mts");
    let mixed_config = config(&["low", "max"]);
    let facts = collect_check_facts(root, vec![path.clone()], plan(root, &mixed_config));
    let file = facts.ts.get(&path).unwrap();
    let mixed = file.embedded_sql[0]
        .1
        .call_spans
        .iter()
        .map(|span| file.query_annotation_heads.as_ref().unwrap().calls[span])
        .collect::<Vec<_>>();
    assert_eq!(mixed, low);
    let max_config = config(&["max"]);
    let observer = InvocationObserver::new(true);
    let max = with_observer(Some(observer.clone()), || {
        collect_check_facts(root, vec![path.clone()], plan(root, &max_config))
    });
    let max_file = max.ts.get(&path).unwrap();
    assert_eq!(
        max_file.embedded_sql[0].1.call_spans,
        file.embedded_sql[0].1.call_spans
    );
    assert!(
        observer
            .snapshot()
            .work
            .get("postgres.sql.variants_composed")
            .copied()
            .unwrap_or(0)
            > 0
    );
    assert_eq!(decisions(root), low);
}
