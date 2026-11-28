use super::*;
use crate::statements::StatementPolicySources;
fn sites(facts: &SqlStatementFileFacts) -> Vec<usize> {
    let mut lines = facts
        .grouping_queries
        .iter()
        .flat_map(|query| query.sites.iter().map(|site| site.0))
        .collect::<Vec<_>>();
    lines.sort();
    lines
}

#[test]
fn grouping_retains_complete_siblings_around_an_unsupported_chunk() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-grouping-not-aggregation/fixture/recovery/raw/recovery-controls.sql"));
    let normalized = crate::parse::PreparedSql::new(source);
    let recovered =
        crate::parse::parse_postgres_sql_lenient_with_sources(source, normalized.normalized());
    assert!(
        recovered.iter().any(|statement| !statement.complete
            && statement.recovery_failure.is_some()
            && matches!(
                &statement.statement,
                sqlparser::ast::Statement::Discard { .. }
            )),
        "{recovered:?}"
    );
    let facts = extract_sql_statement_facts_with_policy(
        source,
        false,
        None,
        SqlLocationDemand::Full,
        StatementPolicySources {
            collect_grouping: true,
            ..Default::default()
        },
        false,
    );
    assert!(facts.parse_failed);
    assert_eq!(sites(&facts), [2, 6]);
    assert_eq!(facts.grouping_queries.len(), 2);
}

#[test]
fn borrowed_completeness_gates_query_roots_without_changing_other_facts() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres-facts/grouping/fixture/completeness.sql"
    ));
    let complete: Vec<bool> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres-facts/grouping/fixture/completeness.json"
    )))
    .unwrap();
    let prepared = crate::parse::PreparedSql::new(sql);
    let statements = prepared.parse_policy().unwrap();
    assert_eq!(statements.len(), complete.len());
    let project = |statement_complete: Option<&[bool]>| {
        extract_from_parsed_with_recovered_placeholders(
            sql,
            &prepared,
            &statements,
            true,
            false,
            None,
            StatementPolicySources {
                collect_grouping: true,
                statement_complete,
                ..Default::default()
            },
        )
    };
    let mut ungated = project(None);
    let mut gated = project(Some(&complete));
    assert_eq!(sites(&ungated), [1, 2, 3]);
    assert_eq!(sites(&gated), [1, 3]);
    assert_eq!(gated.grouping_queries.len(), 2);
    assert!(gated.parse_failed);
    gated.grouping_queries.clear();
    ungated.grouping_queries.clear();
    assert_eq!(gated, ungated);
}
