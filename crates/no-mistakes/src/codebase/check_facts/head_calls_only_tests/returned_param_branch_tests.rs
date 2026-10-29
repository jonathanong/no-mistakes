//! Only a complete same-factory branch set may project the paired native fields.
use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};
fn saved() -> tempfile::TempDir {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("returned-param-branches");
    crate::test_support::materialize_saved_fixture(&root)
}
#[test]
fn returned_parameter_branches_join_all_outcomes_without_low_evaluation() {
    let fixture = saved();
    let root = fixture.path();
    let query = root.join("src/query.mts");
    let factory = root.join("src/factories.mts");
    let source = std::fs::read_to_string(&query).unwrap();
    let expected = source
        .lines()
        .filter_map(|line| line.split_once("// expect: "))
        .flat_map(|(_, values)| values.split(','))
        .map(|value| match value {
            "Literal" => HeadProof::Literal,
            "Missing" => HeadProof::Missing,
            "Unproven" => HeadProof::Unproven,
            _ => panic!("{value}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(expected.len(), 35);
    let mut low = None;
    let mut max_calls = None;
    for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(
                root,
                vec![query.clone(), factory.clone()],
                plan(root, &config(&efforts)),
            )
        });
        let file = facts.ts.get(&query).unwrap();
        let embedded = &file.embedded_sql[0].1;
        let work = observer.snapshot().work;
        if efforts == ["max"] {
            max_calls = Some(format!("{:?}", embedded.calls));
            assert_eq!(
                work.get("postgres.annotation.returned_caller_module_indexes")
                    .copied()
                    .unwrap_or(0),
                0
            );
            assert_eq!(
                work.get("postgres.annotation.returned_caller_indexes")
                    .copied()
                    .unwrap_or(0),
                0
            );
            continue;
        }
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let decisions = embedded
            .call_spans
            .iter()
            .map(|span| heads.calls[span])
            .collect::<Vec<_>>();
        assert_eq!(decisions, expected);
        assert!(facts
            .ts
            .get(&factory)
            .unwrap()
            .query_annotation_heads
            .as_ref()
            .unwrap()
            .exported_returned_params
            .contains("factory"));
        if efforts == ["low"] {
            assert!(file.query_annotation.is_empty());
            assert_eq!(
                work.get("postgres.sql.variants_composed")
                    .copied()
                    .unwrap_or(0),
                0
            );
            low = Some(decisions);
        } else {
            assert_eq!(Some(decisions), low);
            assert_eq!(Some(format!("{:?}", embedded.calls)), max_calls);
        }
    }
}
#[test]
fn returned_parameter_branches_reject_mutated_imports_dynamic_and_sloppy_sources() {
    let fixture = saved();
    let root = fixture.path();
    let factory = root.join("src/factories.mts");
    let sloppy = root.join("src/sloppy.cjs");
    for name in [
        "mutated-callee.mts",
        "mutated-tag.mts",
        "dynamic.mts",
        "sloppy-query.mts",
    ] {
        let query = root.join("src").join(name);
        let facts = collect_check_facts(
            root,
            vec![query.clone(), factory.clone(), sloppy.clone()],
            plan(root, &config(&["low"])),
        );
        let file = facts.ts.get(&query).unwrap();
        assert_eq!(file.embedded_sql[0].1.call_spans.len(), 1);
        assert_eq!(
            file.query_annotation_heads.as_ref().unwrap().calls
                [&file.embedded_sql[0].1.call_spans[0]],
            HeadProof::Unproven,
            "{name}"
        );
        assert!(facts
            .ts
            .get(&sloppy)
            .unwrap()
            .query_annotation_heads
            .as_ref()
            .is_none_or(|heads| heads.exported_returned_params.is_empty()));
    }
}

#[path = "returned_param_branch_work_tests.rs"]
mod work;
