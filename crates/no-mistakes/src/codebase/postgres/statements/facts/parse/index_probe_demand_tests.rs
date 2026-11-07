use super::*;

#[test]
fn index_probe_evidence_is_opt_in_and_preserves_other_facts() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-required-predicates/fixture/index-probes/constant.sql"
    ));
    let extract = |enabled| {
        extract_sql_statement_facts_with_json_writes(
            sql,
            false,
            None,
            SqlLocationDemand::Full,
            false,
            enabled,
        )
    };
    let without = extract(false);
    assert!(without.index_probes.is_empty());
    let mut with = extract(true);
    assert_eq!(with.index_probes.len(), 2);
    assert_eq!(with.index_probes[&(1, 1)], ["topic_id"]);
    assert_eq!(with.index_probes[&(2, 1)], ["id"]);
    with.index_probes.clear();
    assert_eq!(with, without);
}
