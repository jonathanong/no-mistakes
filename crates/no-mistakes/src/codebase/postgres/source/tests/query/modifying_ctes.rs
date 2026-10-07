use super::*;

#[test]
fn data_modifying_ctes_are_typed_ordered_and_share_query_provenance() {
    let source = fixture("query-modifying-ctes.sql");
    let queries = queries("query-modifying-ctes.sql");
    let q = &queries[0];
    assert!(q.complete, "{:?}", q.unsupported);
    assert_eq!(q.nested_statements.len(), 4);
    let kinds: Vec<_> = q
        .nested_statements
        .iter()
        .map(|s| match &s.facts {
            PostgresSqlQueryStatementKind::Insert { .. } => "insert",
            PostgresSqlQueryStatementKind::Update { .. } => "update",
            PostgresSqlQueryStatementKind::Delete { .. } => "delete",
            PostgresSqlQueryStatementKind::Merge { .. } => "merge",
            _ => "unsupported",
        })
        .collect();
    assert_eq!(kinds, ["insert", "update", "delete", "merge"]);
    for (ordinal, statement) in q.nested_statements.iter().enumerate() {
        assert_eq!(statement.ordinal, ordinal);
        assert!(statement.complete);
        let cte = &q.ctes[statement.cte_id.unwrap()];
        assert_eq!(cte.query_scope_id, statement.query_scope_id);
        assert_eq!(statement.parent_scope_id, Some(cte.owner_scope_id));
        let span = statement.span.as_ref().unwrap();
        assert_eq!(&source[span.start.offset..span.end.offset], statement.sql);
        assert!(span.start.offset < span.end.offset);
    }
    assert!(q.nested_statements[0].sql.contains("/* kept */"));
    let PostgresSqlQueryStatementKind::Insert { insert } = &q.nested_statements[0].facts else {
        panic!()
    };
    assert_eq!(insert.table.as_ref().unwrap().parts[0].identity, "Schéma");
    assert!(insert.table.as_ref().unwrap().parts[0].quoted);
    assert!(
        matches!(insert.source, PostgresSqlCteInsertSource::Values { ref rows, .. } if rows.len() == 2)
    );
    let PostgresSqlQueryStatementKind::Update { update } = &q.nested_statements[1].facts else {
        panic!()
    };
    assert_eq!(update.assignments.len(), 1);
    assert_eq!(update.target_relation_ids.len(), 1);
    assert_eq!(update.from_relation_ids.len(), 1);
    assert!(update.predicate.is_some());
    let PostgresSqlQueryStatementKind::Delete { delete } = &q.nested_statements[2].facts else {
        panic!()
    };
    assert_eq!(delete.using_relation_ids.len(), 1);
    assert!(delete.predicate.is_some());
    let PostgresSqlQueryStatementKind::Merge { merge } = &q.nested_statements[3].facts else {
        panic!()
    };
    assert_eq!(merge.clauses.len(), 4);
    assert_eq!(q.nested_statements[3].returning.len(), 1);
    let merge_scope = q.nested_statements[3].query_scope_id;
    let merge_equalities: Vec<_> = q
        .equalities
        .iter()
        .filter(|e| e.scope_id == merge_scope)
        .collect();
    assert_eq!(merge_equalities.len(), 2);
    assert!(merge_equalities
        .iter()
        .all(|e| !e.context.mandatory && e.clause == PostgresSqlQueryClause::Other));
    assert!(q
        .equalities
        .iter()
        .any(|e| e.scope_id == q.nested_statements[1].query_scope_id && e.context.mandatory));
    assert!(matches!(
        merge.clauses[0].action,
        PostgresSqlMergeAction::Update { .. }
    ));
    assert!(matches!(
        merge.clauses[1].action,
        PostgresSqlMergeAction::Delete
    ));
    assert!(matches!(
        merge.clauses[2].action,
        PostgresSqlMergeAction::Insert { .. }
    ));
    assert!(matches!(
        merge.clauses[3].action,
        PostgresSqlMergeAction::DoNothing
    ));
    assert!(
        matches!(q.nested_statements[0].returning[0], PostgresSqlReturningItem::Expression { ref alias, .. } if alias.as_ref().unwrap().identity == "ID")
    );
    assert!(matches!(
        q.nested_statements[1].returning[0],
        PostgresSqlReturningItem::Wildcard {
            qualifier: Some(_),
            ..
        }
    ));
    assert!(matches!(
        q.nested_statements[2].returning[0],
        PostgresSqlReturningItem::Wildcard {
            qualifier: None,
            ..
        }
    ));
    for ids in [
        &update.target_relation_ids,
        &update.from_relation_ids,
        &delete.target_relation_ids,
        &delete.using_relation_ids,
        &merge.target_relation_ids,
        &merge.source_relation_ids,
    ] {
        assert!(ids.iter().all(|id| *id < q.relations.len()));
    }
}

#[test]
fn unreferenced_and_nested_modifying_ctes_keep_existing_reachability_semantics() {
    let q = queries("query-modifying-ctes.sql");
    assert!(q[1].complete);
    assert_eq!(q[1].nested_statements.len(), 1);
    assert!(!q[1].ctes[0].referenced);
    assert!(!q[1].ctes[0].used);
    assert!(q[1].nested_statements[0].returning.is_empty());
    let PostgresSqlQueryStatementKind::Insert { insert } = &q[1].nested_statements[0].facts else {
        panic!()
    };
    assert!(matches!(
        insert.source,
        PostgresSqlCteInsertSource::DefaultValues
    ));
    let nested = &q[2];
    assert!(nested.complete, "{:?}", nested.unsupported);
    assert_eq!(nested.nested_statements.len(), 3);
    assert_eq!(
        nested
            .nested_statements
            .iter()
            .map(|s| nested.ctes[s.cte_id.unwrap()].name.identity.as_str())
            .collect::<Vec<_>>(),
        ["inner_cte", "final_insert", "deeper"]
    );
    for s in &nested.nested_statements {
        assert!(s.query_scope_id < nested.scopes.len());
        assert!(s.parent_scope_id.unwrap() < nested.scopes.len());
    }
    let PostgresSqlQueryStatementKind::Insert { insert } = &nested.nested_statements[1].facts
    else {
        panic!()
    };
    assert!(
        matches!(insert.source, PostgresSqlCteInsertSource::Select { query_scope_id, .. } if query_scope_id < nested.scopes.len())
    );
    let PostgresSqlQueryStatementKind::Update { update } = &q[3].nested_statements[0].facts else {
        panic!()
    };
    assert_eq!(update.assignments[0].columns.len(), 2);
    let PostgresSqlQueryStatementKind::Insert { insert } = &q[3].nested_statements[1].facts else {
        panic!()
    };
    assert!(matches!(
        insert.on_conflict.as_ref().unwrap().action,
        PostgresSqlConflictAction::DoNothing
    ));
}

#[test]
fn unsupported_children_preserve_typed_payloads_and_make_query_incomplete() {
    let q = queries("query-modifying-ctes-unsupported.sql");
    assert!(q.iter().all(|q| !q.complete));
    for query in &q[1..] {
        assert_eq!(query.nested_statements.len(), 1);
        assert!(!query.nested_statements[0].complete);
        assert!(query.nested_statements[0]
            .unsupported
            .iter()
            .any(|u| u.reason == "INSERT VALUES query modifiers"));
    }
    assert_eq!(q[0].nested_statements.len(), 2);
    assert!(q[0]
        .nested_statements
        .iter()
        .all(|s| !s.complete && !s.unsupported.is_empty()));
    assert!(matches!(
        q[0].nested_statements[0].facts,
        PostgresSqlQueryStatementKind::Insert { .. }
    ));
    assert!(matches!(
        q[0].nested_statements[0].returning[0],
        PostgresSqlReturningItem::Expression { .. }
    ));
    let PostgresSqlQueryStatementKind::Merge { merge } = &q[0].nested_statements[1].facts else {
        panic!()
    };
    assert_eq!(merge.clauses.len(), 2);
}

#[test]
fn broader_dialect_modifiers_keep_typed_children_explicitly_incomplete() {
    let sql = fixture("query-modifying-cte-extensions.sql");
    let ast =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::GenericDialect {}, &sql).unwrap();
    let locations = super::super::super::locations::Locations::new(&sql);
    let q: Vec<_> = ast
        .iter()
        .map(|s| {
            let sqlparser::ast::Statement::Query(query) = s else {
                panic!()
            };
            super::super::super::query::project(query, &locations)
        })
        .collect();
    assert_eq!(q.len(), 6);
    assert!(q
        .iter()
        .all(|q| !q.complete && q.nested_statements.len() == 1));
    assert!(q.iter().all(
        |q| !q.nested_statements[0].complete && !q.nested_statements[0].unsupported.is_empty()
    ));
    assert!(matches!(
        q[2].nested_statements[0].returning[0],
        PostgresSqlReturningItem::Unsupported { .. }
    ));

    let PostgresSqlQueryStatementKind::Merge { merge } = &q[3].nested_statements[0].facts else {
        panic!()
    };
    assert!(merge
        .clauses
        .iter()
        .all(|c| matches!(c.action, PostgresSqlMergeAction::Unsupported { .. })));
    let PostgresSqlQueryStatementKind::Insert { insert } = &q[4].nested_statements[0].facts else {
        panic!()
    };
    assert!(!insert.diagnostics.is_empty());
}

#[test]
fn deeply_nested_query_projection_is_bounded_even_with_a_relaxed_parser_limit() {
    let sql = fixture("query-nesting-limit.sql");
    let ast = sqlparser::parser::Parser::new(&sqlparser::dialect::PostgreSqlDialect {})
        .with_recursion_limit(1024)
        .try_with_sql(&sql)
        .unwrap()
        .parse_statements()
        .unwrap();
    let locations = super::super::super::locations::Locations::new(&sql);
    let sqlparser::ast::Statement::Query(query) = &ast[0] else {
        panic!()
    };
    let q = super::super::super::query::project(query, &locations);
    assert!(!q.complete);
    assert!(q
        .unsupported
        .iter()
        .any(|u| u.reason == "query nesting limit"));
}

#[test]
fn unexpected_cte_body_and_missing_source_span_are_explicit_incomplete_children() {
    let sql = fixture("query-cte-projection-invalid.sql");
    let mut ast =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::PostgreSqlDialect {}, &sql)
            .unwrap();
    let unsupported = ast.pop().unwrap();
    let sqlparser::ast::Statement::Query(query) = &mut ast[0] else {
        panic!()
    };
    *query.with.as_mut().unwrap().cte_tables[0].query.body =
        sqlparser::ast::SetExpr::Insert(unsupported);
    let locations = super::super::super::locations::Locations::new(&sql);
    let q = super::super::super::query::project(query, &locations);
    assert!(!q.complete);
    let child = &q.nested_statements[0];
    assert!(!child.complete);
    assert!(child.span.is_none());
    assert!(child.sql.is_empty());
    assert_eq!(child.unsupported.len(), 2);
    assert!(matches!(
        child.facts,
        PostgresSqlQueryStatementKind::Unsupported { .. }
    ));
}

#[test]
fn rejected_cte_conflict_predicate_has_a_source_diagnostic_and_keeps_the_neighbor() {
    let sql = fixture("query-cte-conflict-rejected.sql");
    let facts = facts("query-cte-conflict-rejected.sql");
    assert_eq!(facts.diagnostics.len(), 1);
    assert!(facts.diagnostics[0].message.contains("sql parser error"));
    assert_eq!(facts.statements.len(), 1);
    assert_eq!(facts.statements[0].sql, "SELECT 42 AS neighbor;");
    assert!(matches!(
        facts.statements[0].facts,
        PostgresSqlStatementKind::Select { .. }
    ));
    let diagnostic = facts.diagnostics[0].span.as_ref().unwrap();
    assert!(sql[diagnostic.start.offset..diagnostic.end.offset].contains("ON CONFLICT"));
}
