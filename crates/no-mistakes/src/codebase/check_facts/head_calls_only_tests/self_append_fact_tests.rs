//! Saved identity-write lifetimes, with original max facts kept separate.
use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};
fn saved() -> tempfile::TempDir {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    crate::test_support::materialize_saved_fixture(&root)
}
#[test]
fn self_append_lifetimes_are_low_only_with_zero_evaluator_work() {
    let fixture = saved();
    let root = fixture.path();
    let path = root.join("self-append-assignment.mts");
    let observer = InvocationObserver::new(true);
    let low = with_observer(Some(observer.clone()), || {
        collect_check_facts(root, vec![path.clone()], plan(root, &config(&["low"])))
    });
    let file = low.ts.get(&path).unwrap();
    let embedded = &file.embedded_sql[0].1;
    let heads = file.query_annotation_heads.as_ref().unwrap();
    let decisions = embedded
        .call_spans
        .iter()
        .map(|span| heads.calls[span])
        .collect::<Vec<_>>();
    assert_eq!(decisions.len(), 34);
    assert_eq!(
        &decisions[..4],
        &[
            HeadProof::Literal,
            HeadProof::Literal,
            HeadProof::Literal,
            HeadProof::Missing
        ]
    );
    assert!(decisions[4..decisions.len() - 1]
        .iter()
        .all(|proof| *proof != HeadProof::Literal));
    assert_eq!(decisions.last(), Some(&HeadProof::Literal));
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
    let mixed = collect_check_facts(
        root,
        vec![path.clone()],
        plan(root, &config(&["low", "max"])),
    );
    let mixed_file = mixed.ts.get(&path).unwrap();
    assert_eq!(
        mixed_file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| mixed_file.query_annotation_heads.as_ref().unwrap().calls[span])
            .collect::<Vec<_>>(),
        decisions
    );
    let max_observer = InvocationObserver::new(true);
    let max = with_observer(Some(max_observer.clone()), || {
        collect_check_facts(root, vec![path.clone()], plan(root, &config(&["max"])))
    });
    let max_file = max.ts.get(&path).unwrap();
    assert_eq!(
        format!("{:?}", max_file.embedded_sql[0].1.calls),
        format!("{:?}", mixed_file.embedded_sql[0].1.calls)
    );
    assert_eq!(
        max_observer
            .snapshot()
            .work
            .get("postgres.annotation.self_append_owner_censuses")
            .copied()
            .unwrap_or(0),
        0
    );
}
#[test]
fn self_append_positive_and_negative_owner_censuses_are_memoized() {
    let fixture = saved();
    let root = fixture.path();
    for (name, expected) in [
        ("self-append-assignment-many.mts", HeadProof::Literal),
        (
            "self-append-assignment-negative-cache.mts",
            HeadProof::Unproven,
        ),
    ] {
        let path = root.join(name);
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![path.clone()], plan(root, &config(&["low"])))
        });
        let file = facts.ts.get(&path).unwrap();
        assert_eq!(file.embedded_sql[0].1.call_spans.len(), 64);
        assert!(file.embedded_sql[0].1.call_spans.iter().all(|span| file
            .query_annotation_heads
            .as_ref()
            .unwrap()
            .calls[span]
            == expected));
        let work = observer.snapshot().work;
        assert_eq!(
            work.get("postgres.annotation.self_append_owner_censuses")
                .copied(),
            Some(1)
        );
        assert_eq!(
            work.get("postgres.annotation.self_append_tag_censuses")
                .copied(),
            Some(1)
        );
        assert_eq!(
            work.get("postgres.annotation.self_append_wanted_owner_visits")
                .copied(),
            Some(1)
        );
        assert_eq!(
            work.get("postgres.annotation.self_append_wanted_reference_visits")
                .copied(),
            Some(if expected == HeadProof::Literal {
                66
            } else {
                65
            })
        );
        let count = work
            .get("postgres.annotation.self_append_reference_visits")
            .copied()
            .unwrap();
        if expected == HeadProof::Literal {
            assert_eq!(count, 132);
        } else {
            assert!(count <= 130);
        }
    }
}
#[test]
fn self_append_dynamic_gap_keeps_the_exact_executor_unproven() {
    let fixture = saved();
    let root = fixture.path();
    let path = root.join("self-append-assignment-gap.mts");
    let facts = collect_check_facts(root, vec![path.clone()], plan(root, &config(&["low"])));
    let file = facts.ts.get(&path).unwrap();
    assert_eq!(file.embedded_sql[0].1.call_spans.len(), 1);
    assert_eq!(
        file.query_annotation_heads.as_ref().unwrap().calls[&file.embedded_sql[0].1.call_spans[0]],
        HeadProof::Unproven
    );
}

#[test]
fn self_append_indexes_skip_ordinary_native_consts_and_reject_written_imports() {
    let fixture = saved();
    let root = fixture.path();
    for (name, expected, censuses) in [
        ("self-append-assignment-ordinary.mts", HeadProof::Literal, 0),
        (
            "self-append-assignment-import-write.mts",
            HeadProof::Unproven,
            1,
        ),
    ] {
        let path = root.join(name);
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, vec![path.clone()], plan(root, &config(&["low"])))
        });
        let file = facts.ts.get(&path).unwrap();
        assert_eq!(file.embedded_sql[0].1.call_spans.len(), 1);
        assert_eq!(
            file.query_annotation_heads.as_ref().unwrap().calls
                [&file.embedded_sql[0].1.call_spans[0]],
            expected
        );
        let count = observer
            .snapshot()
            .work
            .get("postgres.annotation.self_append_owner_censuses")
            .copied()
            .unwrap_or(0);
        if censuses == 0 {
            assert_eq!(count, 0);
        } else {
            assert!(count <= 1);
        }
    }
}

#[test]
fn unused_and_custom_let_tags_do_not_expand_self_append_wanted_owners() {
    let fixture = saved();
    let root = fixture.path();
    let path = root.join("self-append-assignment-unused.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(root, vec![path.clone()], plan(root, &config(&["low"])))
    });
    let file = facts.ts.get(&path).unwrap();
    let spans = &file.embedded_sql[0].1.call_spans;
    assert_eq!(spans.len(), 2);
    let heads = file.query_annotation_heads.as_ref().unwrap();
    assert_eq!(heads.calls[&spans[0]], HeadProof::Literal);
    assert_eq!(heads.calls[&spans[1]], HeadProof::Unproven);
    let work = observer.snapshot().work;
    for metric in [
        "postgres.annotation.self_append_wanted_owner_visits",
        "postgres.annotation.self_append_wanted_reference_visits",
    ] {
        assert_eq!(work.get(metric).copied().unwrap_or(0), 0);
    }
}

#[test]
fn unconfigured_reads_do_not_construct_self_append_demand_lookup() {
    let fixture = saved();
    let root = fixture.path();
    let path = root.join("self-append-assignment-unused-only.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(root, vec![path.clone()], plan(root, &config(&["low"])))
    });
    let file = facts.ts.get(&path).unwrap();
    assert_eq!(file.embedded_sql[0].1.call_spans.len(), 1);
    assert_eq!(
        file.query_annotation_heads.as_ref().unwrap().calls[&file.embedded_sql[0].1.call_spans[0]],
        HeadProof::Literal
    );
    let work = observer.snapshot().work;
    for metric in [
        "postgres.annotation.self_append_wanted_owner_visits",
        "postgres.annotation.self_append_wanted_reference_visits",
        "postgres.annotation.self_append_wanted_lookup_visits",
    ] {
        assert_eq!(work.get(metric).copied().unwrap_or(0), 0);
    }
}
