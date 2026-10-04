use super::{facts, SqlBoundItemKind};

#[test]
fn scalar_case_boundaries_do_not_claim_caller_values_or_hide_table_reads() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/scalar-case-projection.sql"));
    let facts = facts(sql);
    let derived = |index: usize| {
        let SqlBoundItemKind::Query(query) = &facts[index].query.items[1].kind else {
            panic!("derived query expected");
        };
        query
    };
    let scalar = derived(0);
    assert!(!scalar.capped);
    // Scalar cardinality is separate from proving which values a function returns.
    assert!(scalar.outputs.iter().all(|output| !output.caller_sized));
    assert!(!scalar
        .items
        .iter()
        .any(|item| matches!(item.kind, SqlBoundItemKind::Opaque)));
    let dependent = derived(8);
    assert!(!dependent.capped);
    assert!(dependent
        .items
        .iter()
        .any(|item| matches!(&item.kind, SqlBoundItemKind::Table(name) if name == "orders")));
    assert!(derived(13).capped);
}
