use super::{collect_check_facts, config, plan, NoMistakesConfig};
use crate::codebase::postgres::query_annotation::heads::HeadProof;

#[test]
fn finite_helper_suffix_variants_keep_low_head_proofs_under_mixed_effort() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let path = root.join("finite-variant-candidates.mts");
    let mut low_proofs = None;
    for efforts in [vec!["low"], vec!["low", "max"]] {
        let facts = collect_check_facts(root, vec![path.clone()], plan(root, &config(&efforts)));
        let file = facts.ts.get(&path).unwrap();
        let embedded = &file.embedded_sql[0].1;
        assert_eq!(embedded.call_spans.len(), 3);
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let proofs = embedded
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        assert_eq!(proofs, vec![Some(HeadProof::Literal); 3]);
        if let Some(low_proofs) = &low_proofs {
            assert_eq!(&proofs, low_proofs);
            assert!(embedded.calls.iter().any(|call| !call.variants.is_empty()));
        } else {
            low_proofs = Some(proofs);
            assert!(embedded.calls.iter().all(|call| call.variants.is_empty()));
        }
    }
}

#[test]
fn imported_helper_head_is_independent_of_another_applications_full_sql_demand() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("imported-helper-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let config_dir = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .parent()
        .unwrap()
        .join("configs");
    let collect = |name: &str| {
        let source = std::fs::read_to_string(config_dir.join(name)).unwrap();
        let mut config: NoMistakesConfig = serde_yaml::from_str(&source).unwrap();
        crate::codebase::rules::analysis_policy::prepare_rule_analysis(
            root,
            &mut config,
            &[],
            &crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(
                crate::codebase::ts_source::FileInventory::from_paths(&[]),
            )),
        )
        .unwrap();
        let plan = plan(root, &config);
        let head_only = plan.embedded_sql_head_calls_only.len();
        let caller = root.join("src/query.mts");
        let facts = collect_check_facts(
            root,
            vec![
                caller.clone(),
                root.join("src/helpers.mts"),
                root.join("src/reexport.mts"),
            ],
            plan,
        );
        let file = facts.ts.get(&caller).unwrap();
        let heads = file.query_annotation_heads.as_ref().unwrap();
        let calls = file.embedded_sql[0]
            .1
            .call_spans
            .iter()
            .map(|span| heads.calls.get(span).copied())
            .collect::<Vec<_>>();
        (head_only, calls)
    };
    let low = collect("imported-helper-heads-scoped-low.yml");
    let mixed = collect("imported-helper-heads-scoped-mixed.yml");
    assert_eq!(low.0, 1);
    assert_eq!(mixed.0, 0);
    assert_eq!(low.1, mixed.1);
}
