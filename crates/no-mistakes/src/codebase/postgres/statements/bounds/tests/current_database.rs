use crate::codebase::postgres::{extract_sql_statement_facts, statements::SqlBoundItemKind};

#[test]
fn current_database_candidates_preserve_exact_identity_and_lifetime() {
    let facts = extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database.sql")));
    assert!(!facts.parse_failed);
    let candidates = facts
        .bounds
        .iter()
        .map(|fact| {
            let item = &fact.query.items[0];
            assert!(matches!(item.kind, SqlBoundItemKind::Table(_)));
            item.possible_temporary
                .as_ref()
                .and_then(|candidate| candidate.database_qualifier.as_deref())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        candidates,
        [
            Some("Audit.Database"),
            Some("Audit.Database"),
            None,
            Some("Audit.Database"),
            None,
            Some("Audit.Database"),
            Some("Audit.Database"),
            Some("Audit.Database"),
            None,
            None,
            Some("Audit.Database"),
            None,
            None,
            None,
            Some("Audit.Database"),
            Some("other")
        ]
    );
}

#[test]
fn conditional_view_sources_and_destinations_follow_database_identity() {
    let facts = extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-views.sql")));
    assert!(!facts.parse_failed);
    let candidates = facts
        .bounds
        .iter()
        .map(|fact| {
            fact.query.items[0]
                .possible_temporary
                .as_ref()
                .and_then(|candidate| candidate.database_qualifier.as_deref())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        candidates,
        [
            Some("Audit.Database"),
            Some("Audit.Database"),
            None,
            Some("Audit.Database"),
            None,
            Some("Audit.Database"),
            None,
            Some("auditdb"),
            None
        ]
    );
}

#[test]
fn conditional_temporary_partition_ownership_follows_database_identity() {
    let facts = extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-partitions.sql")));
    // GenericDialect rejects ATTACH/DETACH; the narrow statement recovery is intentional.
    assert!(facts.parse_failed);
    let candidates = facts
        .bounds
        .iter()
        .map(|fact| {
            fact.query.items[0]
                .possible_temporary
                .as_ref()
                .and_then(|candidate| candidate.database_qualifier.as_deref())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        candidates,
        [
            Some("Audit.Database"),
            Some("Audit.Database"),
            None,
            None,
            Some("Audit.Database"),
            None
        ]
    );
}

#[test]
fn database_conditions_survive_wrong_kind_materialized_drops() {
    let facts = extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-materialized.sql")));
    assert!(!facts.parse_failed);
    let candidates = facts
        .bounds
        .iter()
        .map(|fact| {
            fact.query.items[0]
                .possible_temporary
                .as_ref()
                .and_then(|candidate| candidate.database_qualifier.as_deref())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        candidates,
        [
            Some("Audit.Database"),
            Some("Audit.Database"),
            None,
            Some("Audit.Database"),
            None
        ]
    );
}

#[test]
fn physical_view_cascades_retire_conditional_temporary_dependents() {
    let facts = extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-physical-chain.sql")));
    assert!(!facts.parse_failed);
    let candidates = facts
        .bounds
        .iter()
        .map(|fact| {
            fact.query.items[0]
                .possible_temporary
                .as_ref()
                .and_then(|candidate| candidate.database_qualifier.as_deref())
        })
        .collect::<Vec<_>>();
    assert_eq!(candidates, [Some("Audit.Database"), None, None]);
}
