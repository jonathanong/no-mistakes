use super::{facts, fixture};
use crate::codebase::postgres::source::*;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

fn insert(facts: &PostgresSqlStatementKind) -> &PostgresSqlInsert {
    let PostgresSqlStatementKind::Insert { insert } = facts else {
        panic!("INSERT expected")
    };
    insert
}

fn lineage(sources: &PostgresSqlInsertColumnSources) -> Vec<(String, Vec<String>)> {
    let PostgresSqlInsertColumnSources::Mapped { columns, .. } = sources else {
        panic!("mapped lineage expected, got {sources:?}")
    };
    columns
        .iter()
        .map(|column| {
            (
                column
                    .column
                    .parts
                    .iter()
                    .map(|part| part.identity.clone())
                    .collect::<Vec<_>>()
                    .join("."),
                column
                    .sources
                    .iter()
                    .map(|source| match source {
                        PostgresSqlInsertSourceExpression::Values { expression, .. }
                        | PostgresSqlInsertSourceExpression::Select { expression, .. } => {
                            expression.sql.clone()
                        }
                    })
                    .collect(),
            )
        })
        .collect()
}

fn nested_insert(statement: &PostgresSqlQueryStatement) -> &PostgresSqlCteInsert {
    let PostgresSqlQueryStatementKind::Insert { insert } = &statement.facts else {
        panic!("nested INSERT expected")
    };
    insert
}

#[test]
fn returning_inserts_keep_complete_source_facts_and_lineage() {
    let sql = fixture("insert-returning.sql");
    let result = facts("insert-returning.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 13);
    let values = insert(&result.statements[0].facts);
    assert!(values.complete, "{:?}", values.diagnostics);
    assert!(values.diagnostics.is_empty());
    assert!(matches!(
        values.on_conflict.as_ref().unwrap().action,
        PostgresSqlConflictAction::DoNothing
    ));
    assert_eq!(values.returning.len(), 1);
    assert!(matches!(
        values.returning[0],
        PostgresSqlReturningItem::Expression { ref alias, .. } if alias.is_none()
    ));
    let PostgresSqlInsertSource::Values { span, .. } = &values.source else {
        panic!("VALUES expected")
    };
    let span = span.as_ref().unwrap();
    // sqlparser's VALUES span covers the rows, not the VALUES keyword.
    assert_eq!(&sql[span.start.offset..span.end.offset], "(1)");
    assert_eq!(
        lineage(&values.column_sources),
        vec![("id".into(), vec!["1".into()])]
    );

    let rows = insert(&result.statements[1].facts);
    assert!(rows.complete);
    assert_eq!(rows.returning.len(), 2);
    assert!(matches!(
        &rows.returning[1],
        PostgresSqlReturningItem::Expression { alias: Some(alias), .. } if alias.identity == "Note"
    ));
    assert_eq!(
        lineage(&rows.column_sources),
        vec![
            ("id".into(), vec!["1".into(), "2".into()]),
            ("note".into(), vec!["'a'".into(), "'b'".into()])
        ]
    );

    let quoted = insert(&result.statements[2].facts);
    assert!(quoted.complete, "{:?}", quoted.diagnostics);
    assert!(quoted.table.as_ref().unwrap().parts[0].quoted);
    assert_eq!(quoted.columns[0].parts[0].identity, "ID");
    let PostgresSqlInsertSource::Select { span, .. } = &quoted.source else {
        panic!("SELECT expected")
    };
    let span = span.as_ref().unwrap();
    assert!(sql[span.start.offset..span.end.offset].starts_with("SELECT"));
    assert!(matches!(
        &quoted.returning[0],
        PostgresSqlReturningItem::Expression { alias: None, expression } if expression.sql == "\"ID\""
    ));

    let updated = insert(&result.statements[3].facts);
    assert!(updated.complete, "{:?}", updated.diagnostics);
    let PostgresSqlConflictAction::DoUpdate { assignments, .. } =
        &updated.on_conflict.as_ref().unwrap().action
    else {
        panic!("UPDATE expected")
    };
    assert_eq!(
        assignments[0].provenance,
        PostgresSqlInsertProvenance::ExcludedColumn
    );
    assert!(matches!(
        &updated.returning[0],
        PostgresSqlReturningItem::Expression { expression, .. } if expression.sql == "target.id"
    ));

    let defaults = insert(&result.statements[4].facts);
    assert!(defaults.complete, "{:?}", defaults.diagnostics);
    assert!(matches!(
        defaults.returning[0],
        PostgresSqlReturningItem::Wildcard {
            qualifier: None,
            ..
        }
    ));
    assert!(matches!(
        defaults.source,
        PostgresSqlInsertSource::DefaultValues
    ));
    assert!(matches!(
        defaults.column_sources,
        PostgresSqlInsertColumnSources::Unsupported {
            reason: PostgresSqlInsertColumnSourcesReason::ColumnsOmitted,
            ..
        }
    ));

    let plain = insert(&result.statements[5].facts);
    assert!(plain.complete);
    assert!(plain.returning.is_empty());
    assert_eq!(
        lineage(&plain.column_sources),
        vec![("id".into(), vec!["COALESCE(1, 2)".into()])]
    );

    let ordered = insert(&result.statements[6].facts);
    assert!(!ordered.complete);
    assert!(!ordered.diagnostics.is_empty());
    assert_eq!(ordered.returning.len(), 1);
    assert!(matches!(
        ordered.source,
        PostgresSqlInsertSource::Values { .. }
    ));

    let PostgresSqlStatementKind::Select { query } = &result.statements[7].facts else {
        panic!("CTE expected")
    };
    assert!(query.complete, "{:?}", query.unsupported);
    assert_eq!(query.nested_statements.len(), 2);
    for child in &query.nested_statements {
        assert!(child.complete, "{:?}", child.unsupported);
        assert!(!child.returning.is_empty());
        let insert = nested_insert(child);
        assert!(insert.column_sources.is_some());
        let span = child.span.as_ref().unwrap();
        assert_eq!(&sql[span.start.offset..span.end.offset], child.sql);
        assert!(child.sql.contains("RETURNING"));
    }
    let added = nested_insert(&query.nested_statements[0]);
    assert!(added.table.as_ref().unwrap().parts[0].quoted);
    assert_eq!(added.columns[0].parts[0].identity, "ID");
    assert!(matches!(
        added.on_conflict.as_ref().unwrap().action,
        PostgresSqlConflictAction::DoNothing
    ));
    assert!(matches!(
        added.source,
        PostgresSqlCteInsertSource::Values { .. }
    ));
    let copied = nested_insert(&query.nested_statements[1]);
    assert!(matches!(
        copied.source,
        PostgresSqlCteInsertSource::Select { .. }
    ));
    let copied_span = match &copied.source {
        PostgresSqlCteInsertSource::Select { span, .. } => span.as_ref().unwrap(),
        _ => panic!("SELECT source"),
    };
    assert!(sql[copied_span.start.offset..copied_span.end.offset].starts_with("SELECT"));

    let PostgresSqlStatementKind::Select { query } = &result.statements[8].facts else {
        panic!("plain CTE expected")
    };
    let plain_child = &query.nested_statements[0];
    assert!(plain_child.returning.is_empty());
    assert!(nested_insert(plain_child).column_sources.is_none());

    let star = insert(&result.statements[9].facts);
    assert!(star.complete, "{:?}", star.diagnostics);
    assert!(matches!(
        &star.returning[0],
        PostgresSqlReturningItem::Wildcard {
            qualifier: Some(name),
            ..
        } if name.parts[0].identity == "target"
    ));

    let direct = insert(&result.statements[0].facts);
    let PostgresSqlStatementKind::Select { query } = &result.statements[10].facts else {
        panic!("same-syntax CTE expected")
    };
    let child = &query.nested_statements[0];
    assert!(child.complete, "{:?}", child.unsupported);
    assert!(child
        .sql
        .contains("ON CONFLICT (id) DO NOTHING RETURNING id"));
    assert_eq!(
        lineage(nested_insert(child).column_sources.as_ref().unwrap()),
        lineage(&direct.column_sources)
    );
    let reparsed = super::super::parse_postgres_source(&PostgresSqlSource {
        sql: child.sql.clone(),
        file_name: None,
    });
    assert!(
        reparsed.diagnostics.is_empty(),
        "{:?}",
        reparsed.diagnostics
    );
    let reparsed = insert(&reparsed.statements[0].facts);
    assert!(reparsed.complete, "{:?}", reparsed.diagnostics);
    assert_eq!(
        lineage(&reparsed.column_sources),
        lineage(&direct.column_sources)
    );
    assert_eq!(reparsed.returning.len(), direct.returning.len());

    let PostgresSqlStatementKind::Select { query } = &result.statements[11].facts else {
        panic!("selected CTE expected")
    };
    let selected = nested_insert(&query.nested_statements[0]);
    assert_eq!(
        lineage(selected.column_sources.as_ref().unwrap()),
        lineage(&quoted.column_sources)
    );
    assert!(matches!(
        selected.source,
        PostgresSqlCteInsertSource::Select { .. }
    ));

    let PostgresSqlStatementKind::Select { query } = &result.statements[12].facts else {
        panic!("non-returning CTE expected")
    };
    assert!(nested_insert(&query.nested_statements[0])
        .column_sources
        .is_none());
    assert_eq!(
        lineage(&plain.column_sources),
        vec![("id".into(), vec!["COALESCE(1, 2)".into()])]
    );
}

#[test]
fn returning_function_span_keeps_comment_and_parentheses() {
    let sql = fixture("insert-returning-delimiters.sql");
    let result = facts("insert-returning-delimiters.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let value = insert(&result.statements[0].facts);
    assert!(value.complete, "{:?}", value.diagnostics);
    assert!(value.diagnostics.is_empty());
    let PostgresSqlReturningItem::Expression {
        expression,
        alias: None,
    } = &value.returning[0]
    else {
        panic!("expression expected, got {:?}", value.returning);
    };
    assert_eq!(expression.sql, "now()");
    let span = expression.span.as_ref().expect("returning span");
    assert_eq!(&sql[span.start.offset..span.end.offset], "now /*keep*/ ()");
}

#[test]
fn malformed_returning_stays_diagnostic_and_unsupported_shapes_stay_incomplete() {
    let rejected = super::super::parse_postgres_source(&PostgresSqlSource {
        sql: "INSERT INTO accounts VALUES (1) ON CONFLICT DO NOTHING RETURNING;".into(),
        file_name: None,
    });
    assert!(rejected.statements.is_empty());
    assert!(!rejected.diagnostics.is_empty());

    let sql = "INSERT INTO target (id) VALUES (1) RETURNING * EXCLUDE (id)";
    let statements = Parser::parse_sql(&GenericDialect {}, sql).unwrap();
    let locations = super::super::locations::Locations::new(sql);
    let sqlparser::ast::Statement::Insert(value) = &statements[0] else {
        panic!("INSERT expected")
    };
    let projected = super::super::insert::project(value, None, &locations, &[]);
    assert!(!projected.complete);
    assert!(!projected.diagnostics.is_empty());
    assert!(matches!(
        projected.returning[0],
        PostgresSqlReturningItem::Unsupported { .. }
    ));
    assert!(matches!(
        projected.source,
        PostgresSqlInsertSource::Values { .. }
    ));
    assert_eq!(
        lineage(&projected.column_sources),
        vec![("id".into(), vec!["1".into()])]
    );
}

#[test]
fn literal_execute_keeps_direct_and_nested_returning_lineage() {
    let result = facts("insert-returning-execute.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[0].facts else {
        panic!("DO expected")
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    assert_eq!(block.statements.len(), 2);
    let PostgresSqlStatementKind::LiteralExecute { execute } = &block.statements[0].facts else {
        panic!("direct EXECUTE expected")
    };
    assert!(execute.complete, "{:?}", execute.diagnostics);
    let direct = insert(&execute.statements[0].facts);
    assert!(direct.complete, "{:?}", direct.diagnostics);
    assert_eq!(
        lineage(&direct.column_sources),
        vec![("id".into(), vec!["1".into()])]
    );
    let child = &execute.statements[0];
    assert_eq!(
        &execute.decoded_sql[child.span.start.offset..child.span.end.offset],
        child.sql
    );
    let PostgresSqlStatementKind::LiteralExecute { execute } = &block.statements[1].facts else {
        panic!("CTE EXECUTE expected")
    };
    assert!(execute.complete, "{:?}", execute.diagnostics);
    let PostgresSqlStatementKind::Select { query } = &execute.statements[0].facts else {
        panic!("CTE query expected")
    };
    let child = &query.nested_statements[0];
    assert!(child.complete, "{:?}", child.unsupported);
    assert!(child.sql.contains("RETURNING id"));
    assert_eq!(
        lineage(nested_insert(child).column_sources.as_ref().unwrap()),
        vec![("id".into(), vec!["2".into()])]
    );
    let reparsed = super::super::parse_postgres_source(&PostgresSqlSource {
        sql: child.sql.clone(),
        file_name: None,
    });
    let reparsed = insert(&reparsed.statements[0].facts);
    assert!(reparsed.complete, "{:?}", reparsed.diagnostics);
    assert_eq!(
        lineage(&reparsed.column_sources),
        lineage(nested_insert(child).column_sources.as_ref().unwrap())
    );
}

#[test]
fn with_wrapped_values_keep_mapped_lineage_for_direct_and_cte_inserts() {
    let result = facts("insert-with-values-column-sources.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 4);
    let mapped = vec![("id".into(), vec!["1".into()])];

    let direct = insert(&result.statements[0].facts);
    assert!(!direct.complete, "{:?}", direct.diagnostics);
    assert!(matches!(
        direct.source,
        PostgresSqlInsertSource::Values { .. }
    ));
    assert_eq!(lineage(&direct.column_sources), mapped);

    let PostgresSqlStatementKind::Select { query } = &result.statements[1].facts else {
        panic!("RETURNING CTE expected")
    };
    let child = &query.nested_statements[0];
    let nested = nested_insert(child);
    assert!(matches!(
        nested.source,
        PostgresSqlCteInsertSource::Values { .. }
    ));
    assert_eq!(lineage(nested.column_sources.as_ref().unwrap()), mapped);
    assert!(child
        .unsupported
        .iter()
        .any(|item| item.reason == "INSERT VALUES query modifiers"));
    assert_source_seed(query, child.query_scope_id);

    let PostgresSqlStatementKind::Select { query } = &result.statements[2].facts else {
        panic!("non-RETURNING CTE expected")
    };
    let plain_child = &query.nested_statements[0];
    let plain = nested_insert(plain_child);
    assert!(plain.column_sources.is_none());
    assert!(matches!(
        plain.source,
        PostgresSqlCteInsertSource::Values { .. }
    ));
    assert_source_seed(query, plain_child.query_scope_id);

    let PostgresSqlStatementKind::Select { query } = &result.statements[3].facts else {
        panic!("SELECT source CTE expected")
    };
    let selected_child = &query.nested_statements[0];
    assert!(selected_child.complete, "{:?}", selected_child.unsupported);
    let selected = nested_insert(selected_child);
    assert!(matches!(
        selected.source,
        PostgresSqlCteInsertSource::Select { .. }
    ));
    assert_eq!(lineage(selected.column_sources.as_ref().unwrap()), mapped);
    assert_source_seed(query, selected_child.query_scope_id);
}

fn assert_source_seed(query: &PostgresSqlQuery, insert_scope: usize) {
    let seeds: Vec<_> = query
        .ctes
        .iter()
        .filter(|cte| cte.name.identity == "seed")
        .collect();
    assert_eq!(seeds.len(), 1);
    assert_eq!(
        query.scopes[seeds[0].owner_scope_id].parent_scope_id,
        Some(insert_scope)
    );
}

#[test]
fn incompletely_projected_returning_expression_marks_insert_incomplete() {
    let result = facts("insert-returning-incomplete.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 2);

    let represented = insert(&result.statements[0].facts);
    assert!(represented.complete, "{:?}", represented.diagnostics);
    assert!(represented.diagnostics.is_empty());
    assert!(matches!(
        &represented.returning[0],
        PostgresSqlReturningItem::Expression {
            alias: None,
            expression,
        } if expression.children_complete && expression.sql == "id + 1"
    ));
    assert_eq!(
        lineage(&represented.column_sources),
        vec![("id".into(), vec!["1".into()])]
    );

    let between = insert(&result.statements[1].facts);
    assert!(!between.complete);
    assert!(matches!(
        &between.returning[0],
        PostgresSqlReturningItem::Expression {
            alias: None,
            expression,
        } if !expression.children_complete && expression.sql.contains("BETWEEN")
    ));
    assert!(between.diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "INSERT facts contain unsupported or incompletely represented syntax"
    }));
    assert_eq!(
        lineage(&between.column_sources),
        vec![("id".into(), vec!["1".into()])]
    );
}

#[test]
fn returning_cte_column_sources_keep_the_same_call_trivia_as_a_direct_insert() {
    let sql = fixture("insert-returning-cte-spans.sql");
    let result = facts("insert-returning-cte-spans.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 3);
    let direct = insert(&result.statements[0].facts);
    assert!(direct.complete, "{:?}", direct.diagnostics);
    assert_eq!(call_trivia(&direct.column_sources, &sql), "now /*keep*/ ()");

    let PostgresSqlStatementKind::Select { query } = &result.statements[1].facts else {
        panic!("RETURNING CTE expected");
    };
    assert!(query.complete, "{:?}", query.unsupported);
    let child = &query.nested_statements[0];
    assert!(child.complete, "{:?}", child.unsupported);
    let nested = nested_insert(child);
    let sources = nested.column_sources.as_ref().expect("RETURNING lineage");
    assert_eq!(call_trivia(sources, &sql), "now /*keep*/ ()");
    assert_eq!(lineage(sources), lineage(&direct.column_sources));

    let PostgresSqlStatementKind::Select { query } = &result.statements[2].facts else {
        panic!("non-RETURNING CTE expected");
    };
    assert!(nested_insert(&query.nested_statements[0])
        .column_sources
        .is_none());
}

fn call_trivia(sources: &PostgresSqlInsertColumnSources, sql: &str) -> String {
    let PostgresSqlInsertColumnSources::Mapped {
        columns,
        complete: true,
    } = sources
    else {
        panic!("complete mapped lineage expected, got {sources:?}");
    };
    let PostgresSqlInsertSourceExpression::Select { expression, .. } = &columns[0].sources[0]
    else {
        panic!("SELECT source expected");
    };
    assert_eq!(expression.sql, "now()");
    let span = expression.span.as_ref().expect("source span");
    sql[span.start.offset..span.end.offset].to_string()
}
