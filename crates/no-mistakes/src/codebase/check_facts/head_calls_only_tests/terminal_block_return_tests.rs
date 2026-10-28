use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

fn expected(source: &str) -> Vec<HeadProof> {
    source
        .lines()
        .filter_map(|line| line.split_once("// proof: "))
        .map(|(_, proof)| match proof {
            "Literal" => HeadProof::Literal,
            "Missing" => HeadProof::Missing,
            "Unproven" => HeadProof::Unproven,
            _ => panic!("unknown saved oracle {proof}"),
        })
        .collect()
}
#[test]
fn terminal_branch_blocks_share_local_imported_and_mixed_prepared_head_proofs() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("terminal-block-returns");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let paths = [root.join("src/query.mts"), root.join("src/helpers.mts")];
    let source = std::fs::read_to_string(&paths[0]).unwrap();
    let expected = expected(&source);
    assert_eq!(expected.len(), 18);
    let mut max_calls = None;
    for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, paths.to_vec(), plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&paths[0]).unwrap();
        let embedded = &file.embedded_sql[0].1;
        if efforts == ["max"] {
            max_calls = Some(format!("{:?}", embedded.calls));
            continue;
        }
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let proofs = embedded
            .call_spans
            .iter()
            .map(|span| heads.calls[span])
            .collect::<Vec<_>>();
        assert_eq!(proofs, expected);
        let work = observer.snapshot().work;
        assert_eq!(work.get("postgres.annotation.head_syntax_visits"), Some(&2));
        assert_eq!(
            work.get("postgres.annotation.head_binding_indexes"),
            Some(&2)
        );
        if efforts == ["low"] {
            assert!(file.query_annotation.is_empty());
            assert!(embedded.calls.iter().all(|call| call.variants.is_empty()));
            assert_eq!(
                work.get("postgres.sql.variants_composed")
                    .copied()
                    .unwrap_or(0),
                0
            );
        } else {
            assert_eq!(Some(format!("{:?}", embedded.calls)), max_calls);
        }
    }
}
