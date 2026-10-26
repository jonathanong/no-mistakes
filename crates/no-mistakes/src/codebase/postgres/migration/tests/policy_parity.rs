use super::extract_migration_facts;
use crate::codebase::postgres::{parse, statements};

#[test]
fn recovered_policy_lines_match_with_and_without_schema_projection() {
    let sql = include_str!(
        "../../../../../../../fixtures/rules/postgres-migration-statement-lines/policy-parity.sql"
    );
    let schema = extract_migration_facts(sql);
    let direct = statements::extract_sql_statement_facts(sql);
    assert_eq!(
        locations(&direct.statement_kinds),
        locations(&schema.statement_kinds)
    );
    let prepared = parse::PreparedSql::new(sql);
    let (located, functions) = parse::partition_function_sources(
        parse::parse_postgres_sql_lenient_with_sources(sql, prepared.normalized()),
    );
    let sources: Vec<_> = located
        .iter()
        .map(|located| {
            located
                .source
                .clone()
                .or_else(|| located.chunk_tokens.clone())
        })
        .collect();
    let parsed: Vec<_> = located
        .into_iter()
        .map(|located| located.statement)
        .collect();
    let collect = |schema| {
        statements::extract_from_parsed_with_recovered_placeholders(
            sql,
            &prepared,
            &parsed,
            true,
            false,
            None,
            statements::StatementPolicySources {
                tokens: Some(&sources),
                schema,
                functions: &functions,
                ..Default::default()
            },
        )
    };
    // Enabling the schema rule must not alter existing statement-only report lines.
    assert_eq!(
        locations(&collect(None).statement_kinds),
        locations(&collect(Some(&schema)).statement_kinds)
    );
    assert_eq!(
        schema
            .add_columns
            .iter()
            .map(|column| column.line)
            .collect::<Vec<_>>(),
        [5, 9, 16, 20]
    );
}

fn locations(facts: &[crate::codebase::postgres::SqlStatementKind]) -> Vec<(&str, usize)> {
    let mut locations: Vec<_> = facts
        .iter()
        .map(|fact| (fact.kind.as_str(), fact.line))
        .collect();
    locations.sort_unstable();
    locations
}
