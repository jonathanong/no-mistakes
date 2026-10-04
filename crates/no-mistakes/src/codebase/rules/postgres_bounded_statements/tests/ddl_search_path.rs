use super::offenders;
use crate::codebase::postgres::{
    extract_sql_statement_facts, statements::SqlBoundItemKind, SchemaCatalog,
};

fn assert_candidates(sql: &str, expected: &[bool], unbounded: &[usize]) {
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let candidates = facts
        .bounds
        .iter()
        .map(|fact| {
            let item = &fact.query.items[0];
            item.possible_temporary.is_some() || matches!(item.kind, SqlBoundItemKind::Opaque)
        })
        .collect::<Vec<_>>();
    assert_eq!(candidates, expected);
    let catalog = SchemaCatalog::from_json(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-search-path-evidence.json"))).unwrap();
    let indices = facts
        .bounds
        .iter()
        .enumerate()
        .filter_map(|(index, fact)| (!offenders(fact, &catalog).is_empty()).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(indices, unbounded);
}

#[test]
fn ddl_search_path_excludes_unreachable_qualified_dependencies() {
    assert_candidates(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-ddl-search-path.sql")), &[false, true, true, true, false, true, false, false, false, false, false], &[0, 4, 6, 7, 8, 9, 10]);
}

#[test]
fn ddl_search_path_proof_restores_at_commit_and_rollback() {
    assert_candidates(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-ddl-search-path-local.sql")), &[true, false, true, false], &[1, 3]);
}

#[test]
fn ddl_search_path_does_not_guess_default_numeric_or_empty_values() {
    let facts = extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-ddl-search-path-unknown.sql")));
    assert!(!facts.parse_failed);
    let candidates = facts
        .bounds
        .iter()
        .map(|fact| {
            let item = &fact.query.items[0];
            item.possible_temporary.is_some() || matches!(item.kind, SqlBoundItemKind::Opaque)
        })
        .collect::<Vec<_>>();
    assert_eq!(candidates, [false, false, false, false, true, true]);
}

#[test]
fn ddl_search_path_targets_do_not_filter_cascaded_graph_nodes() {
    assert_candidates(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-ddl-search-path-chain.sql")), &[true, true, false, false], &[2, 3]);
}
