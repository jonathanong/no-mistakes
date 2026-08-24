use super::super::extract_migration_facts;

#[test]
fn dynamic_plpgsql_sql_contributes_add_column_and_statement_facts() {
    let facts = extract_migration_facts(include_str!(
        "../../../../../../../fixtures/rules/postgres-dynamic-sql/fixture.sql"
    ));
    assert_eq!(
        facts
            .add_columns
            .iter()
            .map(|column| (
                column.table_name.as_str(),
                column.column_name.as_str(),
                column.line
            ))
            .collect::<Vec<_>>(),
        [
            ("direct_posts", "direct_status", 48),
            ("function_posts", "direct_status", 78),
            ("procedure_posts", "direct_status", 84),
            ("posts", "status", 22),
            ("comments", "visible", 39),
            ("accounts", "generated", 62),
        ],
        "{facts:?}"
    );
    assert_eq!(
        facts.indexes[0].name.as_deref(),
        Some("dynamic_posts_status_idx")
    );
    assert_eq!(facts.indexes[0].line, 28);
    assert_eq!(facts.foreign_keys[0].table_name, "posts");
    assert_eq!(facts.foreign_keys[0].referenced_table_name, "users");
    assert_eq!(facts.foreign_keys[0].line, 30);
    assert_eq!(
        facts
            .dropped_indexes
            .iter()
            .map(|index| (index.name.as_str(), index.line))
            .collect::<Vec<_>>(),
        [("obsolete", 54), ("dynamic_identifier", 68)]
    );
}

#[test]
fn routine_bodies_recover_every_statement_policy_kind() {
    let facts = extract_migration_facts(include_str!(
        "../../../../../../../fixtures/rules/postgres-routine-statement-policy/fixture.sql"
    ));
    let counts = facts.statement_kinds.iter().fold(
        std::collections::BTreeMap::<&str, usize>::new(),
        |mut counts, statement| {
            *counts.entry(statement.kind.as_str()).or_default() += 1;
            counts
        },
    );
    assert_eq!(
        counts,
        std::collections::BTreeMap::from([
            ("ALTER TABLE", 3),
            ("CREATE INDEX", 3),
            ("CREATE TABLE", 3),
            ("CREATE VIEW", 3),
            ("DROP INDEX", 3),
            ("DROP VIEW", 3),
            ("TRUNCATE", 3),
        ]),
        "{facts:#?}"
    );
}

#[test]
fn routine_string_forms_preserve_schema_facts_and_physical_lines() {
    let facts = extract_migration_facts(include_str!(
        "../../../../../../../fixtures/rules/postgres-routine-string-literals/fixture.sql"
    ));
    assert_eq!(
        facts
            .tables
            .iter()
            .map(|table| table.table_name.as_str())
            .collect::<Vec<_>>(),
        [
            "custom_escape",
            "concatenated_boundary",
            "escaped_same_line",
            "physical_next_line",
            "default_surrogate",
            "dynamic_identifier"
        ],
        "{facts:#?}"
    );
    assert_eq!(
        facts
            .statement_kinds
            .iter()
            .filter(|statement| statement.kind == "CREATE TABLE")
            .map(|statement| statement.line)
            .collect::<Vec<_>>(),
        [1, 4, 15, 16, 18, 8],
        "{facts:#?}"
    );
    assert_eq!(
        facts
            .add_columns
            .iter()
            .map(|column| (column.table_name.as_str(), column.line))
            .collect::<Vec<_>>(),
        [("surrounding_posts", 20), ("dollar_posts", 22)],
        "{facts:#?}"
    );
}
