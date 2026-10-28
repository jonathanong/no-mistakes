use super::{collect_check_facts, config, plan};

#[test]
fn direct_imported_string_heads_use_prepared_value_exports() {
    use crate::codebase::postgres::query_annotation::heads::HeadProof;
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("imported-string-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let caller = root.join("src/query.mts");
    let facts = collect_check_facts(
        root,
        vec![
            caller.clone(),
            root.join("src/values.mts"),
            root.join("src/reexport.mts"),
            root.join("src/ambiguous.mts"),
        ],
        plan(root, &config(&["low"])),
    );
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
            (14, Some(HeadProof::Unproven)),
            (15, Some(HeadProof::Unproven)),
            (16, Some(HeadProof::Unproven)),
            (17, Some(HeadProof::Unproven)),
            (20, Some(HeadProof::Unproven)),
            (22, Some(HeadProof::Unproven)),
        ]
    );
    assert!(file.query_annotation.is_empty());
}
