use super::*;
#[test]
fn edge_shapes_keep_visibility_and_completeness_explicit() {
    let facts = facts("query-edge-cases.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let queries: Vec<_> = facts
        .statements
        .into_iter()
        .filter_map(|s| {
            if let PostgresSqlStatementKind::Select { query } = s.facts {
                Some(query)
            } else {
                None
            }
        })
        .collect();
    assert!(queries[0]
        .exists
        .iter()
        .all(|e| e.context.under_or && !e.context.mandatory));
    assert!(queries[1]
        .equalities
        .iter()
        .any(|e| e.context.under_other && !e.context.mandatory));
    assert_eq!(
        queries[2].equalities[0].right.as_ref().unwrap().resolution,
        PostgresSqlQueryColumnResolution::Unknown
    );
    assert!(queries[3].columns.iter().all(|c| c.relation_id == Some(0)));
    assert_eq!(queries[4].relations.len(), 2);
    assert_eq!(
        queries[5].relations.last().unwrap().column_aliases[0].identity,
        "renamed"
    );
    assert!(queries.iter().any(|q| !q.complete));
    assert_eq!(
        queries.last().unwrap().equalities[0]
            .left
            .as_ref()
            .unwrap()
            .resolution,
        PostgresSqlQueryColumnResolution::Unknown
    );
}

#[test]
fn parser_extensions_keep_projection_limits_and_unsupported_facts_explicit() {
    // Parse a saved fixture with the broader dialect to exercise defensive AST handling.
    // The public source API continues to use PostgreSQL and rejects these syntax extensions.
    let sql = fixture("query-dialect-extensions.sql");
    let ast =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::GenericDialect {}, &sql).unwrap();
    let locations = super::super::super::locations::Locations::new(&sql);
    let queries: Vec<_> = ast
        .iter()
        .filter_map(|s| {
            if let sqlparser::ast::Statement::Query(query) = s {
                Some(super::super::super::query::project(query, &locations))
            } else {
                None
            }
        })
        .collect();
    assert!(queries[0].complete);
    assert!(queries[1].complete);
    assert!(queries[2..]
        .iter()
        .all(|q| !q.complete && !q.unsupported.is_empty()));
}

#[test]
fn non_postgres_ordering_ast_is_explicitly_unsupported() {
    let sql = fixture("query-ordering-all.sql");
    let ast =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::DuckDbDialect {}, &sql).unwrap();
    let locations = super::super::super::locations::Locations::new(&sql);
    let sqlparser::ast::Statement::Query(query) = &ast[0] else {
        panic!("query fixture");
    };
    let facts = super::super::super::query::project(query, &locations);
    assert!(!facts.complete);
    assert_eq!(facts.unsupported[0].reason, "ordering form");
}
