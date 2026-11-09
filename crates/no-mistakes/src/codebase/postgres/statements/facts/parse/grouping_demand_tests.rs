use super::*;
use crate::codebase::postgres::statements::facts::locations::work;
use crate::codebase::postgres::statements::{SqlFactSite, StatementPolicySources};

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
fn grouping_roots_are_opt_in_and_leave_unrelated_facts_unchanged() {
    work::begin();
    let without = extract(false);
    assert_eq!(work::finish().grouping_roots, 0);
    assert!(without.grouping_queries.is_empty());
    work::begin();
    let mut with = extract(true);
    assert_eq!(work::finish().grouping_roots, 4);
    assert_eq!(with.grouping_queries.len(), 4);
    let mut sites: Vec<_> = with
        .grouping_queries
        .iter()
        .flat_map(|query| &query.sites)
        .copied()
        .collect();
    sites.sort();
    assert_eq!(
        sites,
        [(2, 3), (4, 3), (6, 1), (7, 15), (8, 27), (9, 21), (11, 17)]
    );
    for (query, fact) in with.grouping_queries.iter().enumerate() {
        for (select, &(line, column)) in fact.sites.iter().enumerate() {
            let position = with
                .variant_locations
                .as_ref()
                .unwrap()
                .position(SqlFactSite::Grouping(query, select))
                .unwrap();
            assert_eq!((position.sql_line, position.sql_column), (line, column));
        }
    }
    with.grouping_queries.clear();
    with.variant_locations
        .as_mut()
        .unwrap()
        .positions
        .retain(|site, _| !matches!(site, SqlFactSite::Grouping(_, _)));
    assert_eq!(with, without);
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
