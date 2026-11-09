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

#[test]
fn generated_parameter_sites_are_exact_shared_and_opt_in() {
    use sqlparser::ast::{visit_expressions, Expr};
    use std::{ops::ControlFlow, sync::Arc};
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres-facts/grouping/fixture/parameter-sites.sql"
    ));
    let positions: Vec<(u32, u32)> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres-facts/grouping/fixture/parameter-sites.json"
    )))
    .unwrap();
    let extract = |enabled| {
        extract_sql_statement_facts_with_policy(
            sql,
            false,
            Some(&positions),
            SqlLocationDemand::None,
            StatementPolicySources {
                collect_grouping: enabled,
                ..Default::default()
            },
            false,
        )
    };
    work::begin();
    let without = extract(false);
    assert_eq!(work::finish().grouping_parameter_copies, 0);
    assert!(without.grouping_queries.is_empty());
    work::begin();
    let with = extract(true);
    let counts = work::finish();
    assert_eq!(counts.grouping_parameter_copies, 1);
    assert_eq!(counts.grouping_roots, 2);
    let roots = &with.grouping_queries;
    assert!(Arc::ptr_eq(
        &roots[0].parameter_sites,
        &roots[1].parameter_sites
    ));
    for (index, root) in roots.iter().enumerate() {
        assert_eq!(root.parameter_sites.as_ref(), positions);
        let mut identifiers = Vec::new();
        let _: ControlFlow<()> = visit_expressions(&root.query, |expr| {
            if let Expr::Identifier(id) = expr {
                if id.value == "sql_placeholder_1" {
                    let site = (id.span.start.line as u32, id.span.start.column as u32);
                    identifiers.push((site.0, site.1, root.parameter_sites.contains(&site)));
                }
            }
            ControlFlow::Continue(())
        });
        let line = index as u32 + 1;
        assert_eq!(identifiers, [(line, 8, false), (line, 48, true)]);
    }
}

#[test]
fn grouping_facts_survive_shared_body_and_lock_projections() {
    let expected = extract(true).grouping_queries;
    for lock_scope in [false, true] {
        let facts = extract_sql_statement_facts_with_policy(
            SQL,
            false,
            None,
            SqlLocationDemand::Full,
            StatementPolicySources {
                collect_grouping: true,
                collect_body_bounds: true,
                ..Default::default()
            },
            lock_scope,
        );
        assert!(!facts.parse_failed);
        assert_eq!(facts.grouping_queries, expected);
    }
}
