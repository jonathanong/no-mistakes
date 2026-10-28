use super::{collect_check_facts, config, plan};
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn imported_append_only_helpers_require_every_prepared_effect() {
    use crate::codebase::postgres::query_annotation::heads::HeadProof;
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("append-effects");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let caller = root.join("src/query.mts");
    let target = root.join("src/helpers.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(
            root,
            vec![caller.clone(), target],
            plan(root, &config(&["low"])),
        )
    });
    let file = facts.ts.get(&caller).unwrap();
    let heads = file.query_annotation_heads.as_ref().unwrap();
    let source = std::fs::read_to_string(&caller).unwrap();
    for (call, span) in file.embedded_sql[0]
        .1
        .calls
        .iter()
        .zip(&file.embedded_sql[0].1.call_spans)
    {
        let line = source.lines().nth(call.line as usize - 1).unwrap();
        let expected = if line.contains("// accepted") {
            HeadProof::Literal
        } else {
            assert!(line.contains("// unanalyzable"), "{line}");
            HeadProof::Unproven
        };
        assert_eq!(heads.calls.get(span).copied(), Some(expected), "{line}");
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
fn mixed_effort_keeps_append_only_effect_out_of_max_fallback() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("append-effects");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let caller = root.join("src/query.mts");
    let facts = collect_check_facts(
        root,
        vec![caller.clone(), root.join("src/helpers.mts")],
        plan(root, &config(&["low", "max"])),
    );
    let file = facts.ts.get(&caller).unwrap();
    let heads = file.query_annotation_heads.as_ref().unwrap();
    let source = std::fs::read_to_string(&caller).unwrap();
    let first_after_helper = file.embedded_sql[0]
        .1
        .call_spans
        .iter()
        .copied()
        .find(|span| span.0 == source.find("read(q); // accepted").unwrap() as u32)
        .unwrap();
    assert!(heads.accepts(first_after_helper));
    assert!(!heads.accepts_trusted_append(first_after_helper));
}
