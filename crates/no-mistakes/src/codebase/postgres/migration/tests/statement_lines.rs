use super::extract_migration_facts;

#[test]
fn repeated_identifiers_keep_their_own_statement_lines() {
    let facts = extract_migration_facts(include_str!(
        "../../../../../../../fixtures/rules/postgres-migration-statement-lines/fixture.sql"
    ));
    // Repeated identifiers deliberately occur in separate tables and statements.
    assert_eq!(
        facts
            .foreign_keys
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [4, 8]
    );
    assert_eq!(
        facts
            .not_valid_constraints
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [10, 14]
    );
    assert_eq!(
        facts
            .validated_constraints
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [18, 20]
    );
    assert_eq!(
        facts
            .add_columns
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [22, 24]
    );
    assert_eq!(
        facts
            .unnamed_constraints
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [26, 27]
    );
    assert_eq!(
        facts
            .indexes
            .iter()
            .filter(|fact| fact.name.is_some())
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [30, 32]
    );
    assert_eq!(
        facts
            .dropped_indexes
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [34, 36]
    );
    assert_eq!(
        facts
            .dropped_tables
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [38, 40]
    );
    assert_eq!(
        facts
            .statement_kinds
            .iter()
            .filter(|fact| fact.kind == "CREATE TABLE")
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [2, 6]
    );
}

#[test]
fn wrapped_statements_keep_their_physical_heads() {
    let facts = extract_migration_facts(include_str!(
        "../../../../../../../fixtures/rules/postgres-migration-statement-lines/wrappers.sql"
    ));
    assert_eq!(
        facts
            .foreign_keys
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [4, 6]
    );
    assert_eq!(
        facts
            .add_columns
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [9, 10]
    );
}

#[test]
fn unsupported_statements_do_not_shift_parsed_statement_lines() {
    let facts = extract_migration_facts(include_str!(
        "../../../../../../../fixtures/rules/postgres-migration-statement-lines/recovery.sql"
    ));
    assert_eq!(
        facts
            .indexes
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [3, 6]
    );
    assert_eq!(
        facts
            .dropped_indexes
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [9]
    );
    assert_eq!(
        facts
            .dropped_tables
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [12]
    );
    assert!(!facts.unanalyzable_statements.is_empty());
}
