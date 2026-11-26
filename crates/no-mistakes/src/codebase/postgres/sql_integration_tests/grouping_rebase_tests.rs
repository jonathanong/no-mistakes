use crate::codebase::postgres::statements::{
    extract_sql_statement_facts_with_policy, SqlFactSite, SqlLocationDemand, SqlStatementFileFacts,
    StatementPolicySources,
};

const SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/postgres-facts/grouping/fixture/executed.sql"
));

fn extract(enabled: bool) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_policy(
        SQL,
        false,
        None,
        SqlLocationDemand::Full,
        StatementPolicySources {
            collect_grouping: enabled,
            ..Default::default()
        },
        false,
    )
}

#[test]
fn grouping_positions_keep_sql_identity_when_embedded_locations_are_rebased() {
    use crate::codebase::postgres::{EmbeddedSqlCall, EmbeddedSqlKind};
    let mut facts = extract(true);
    let call = EmbeddedSqlCall {
        line: 20,
        kind: EmbeddedSqlKind::Inline,
        sql_text: Some(SQL.to_string()),
        ..Default::default()
    };
    crate::codebase::postgres::collect::dml::rebase_embedded_lines(&mut facts, &call, false);
    for (query, fact) in facts.grouping_queries.iter().enumerate() {
        for (select, &(line, column)) in fact.sites.iter().enumerate() {
            let position = facts
                .variant_locations
                .as_ref()
                .unwrap()
                .position(SqlFactSite::Grouping(query, select))
                .unwrap();
            assert_eq!((position.sql_line, position.sql_column), (line, column));
            assert_eq!(position.source_line, line + 19);
        }
    }
}
