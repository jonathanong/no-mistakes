//! Actual caller/ownership construction work, including cached rejection.
use super::{
    collect_check_facts, config, plan, saved, with_observer, HeadProof, InvocationObserver,
};

#[test]
fn returned_parameter_caller_ownership_is_memoized_for_all_paired_reads() {
    let fixture = saved();
    let root = fixture.path();
    let factory = root.join("src/factories.mts");
    for (name, expected, wanted) in [
        ("many.mts", HeadProof::Literal, 128),
        ("many-negative.mts", HeadProof::Unproven, 129),
    ] {
        let query = root.join("src").join(name);
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(
                root,
                vec![query.clone(), factory.clone()],
                plan(root, &config(&["low"])),
            )
        });
        let file = facts.ts.get(&query).unwrap();
        assert_eq!(file.embedded_sql[0].1.call_spans.len(), 64);
        assert!(file.embedded_sql[0].1.call_spans.iter().all(|span| file
            .query_annotation_heads
            .as_ref()
            .unwrap()
            .calls[span]
            == expected));
        let work = observer.snapshot().work;
        assert_eq!(
            work.get("postgres.annotation.returned_caller_owner_censuses")
                .copied(),
            Some(1)
        );
        assert_eq!(
            work.get("postgres.annotation.returned_caller_import_censuses")
                .copied(),
            Some(2)
        );
        assert_eq!(
            work.get("postgres.annotation.returned_caller_wanted_owners")
                .copied(),
            Some(1)
        );
        assert_eq!(
            work.get("postgres.annotation.returned_caller_wanted_references")
                .copied(),
            Some(wanted)
        );
        let visits = work["postgres.annotation.returned_caller_reference_visits"];
        if expected == HeadProof::Literal {
            assert_eq!(visits, 128);
        } else {
            assert!(visits <= 129);
        }
    }
}

#[test]
fn ordinary_calls_do_not_construct_returned_branch_indexes() {
    let fixture = saved();
    let root = fixture.path();
    let query = root.join("src/ordinary.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(root, vec![query.clone()], plan(root, &config(&["low"])))
    });
    let file = facts.ts.get(&query).unwrap();
    assert_eq!(file.embedded_sql[0].1.call_spans.len(), 1);
    assert_eq!(
        file.query_annotation_heads.as_ref().unwrap().calls[&file.embedded_sql[0].1.call_spans[0]],
        HeadProof::Literal
    );
    for metric in [
        "postgres.annotation.returned_caller_indexes",
        "postgres.annotation.returned_caller_module_indexes",
        "postgres.annotation.returned_caller_wanted_lookup_visits",
        "postgres.annotation.returned_caller_owner_censuses",
    ] {
        assert_eq!(
            observer.snapshot().work.get(metric).copied().unwrap_or(0),
            0
        );
    }
}

#[test]
fn unused_conditional_results_do_not_construct_caller_projections() {
    let fixture = saved();
    let root = fixture.path();
    let query = root.join("src/unused.mts");
    let factory = root.join("src/factories.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(
            root,
            vec![query.clone(), factory],
            plan(root, &config(&["low"])),
        )
    });
    let file = facts.ts.get(&query).unwrap();
    assert_eq!(file.embedded_sql[0].1.call_spans.len(), 1);
    assert_eq!(
        file.query_annotation_heads.as_ref().unwrap().calls[&file.embedded_sql[0].1.call_spans[0]],
        HeadProof::Literal
    );
    for metric in [
        "postgres.annotation.returned_caller_indexes",
        "postgres.annotation.returned_caller_module_indexes",
        "postgres.annotation.returned_caller_wanted_lookup_visits",
        "postgres.annotation.returned_caller_owner_censuses",
    ] {
        assert_eq!(
            observer.snapshot().work.get(metric).copied().unwrap_or(0),
            0
        );
    }
}
