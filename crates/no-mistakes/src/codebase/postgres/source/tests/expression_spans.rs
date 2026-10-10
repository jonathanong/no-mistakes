use super::super::{
    PostgresSqlExpression, PostgresSqlExpressionRoot, PostgresSqlInsertColumnSources,
    PostgresSqlInsertSource, PostgresSqlInsertSourceExpression, PostgresSqlStatementKind,
};
use super::{facts, fixture};

#[test]
fn expression_spans_cover_rendered_sql_bytes() {
    let source = fixture("expression-span-bytes.sql");
    let parsed = facts("expression-span-bytes.sql");
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.statements.len(), 6);

    let PostgresSqlStatementKind::DoBlock { block } = &parsed.statements[0].facts else {
        panic!("dollar-quoted DO body");
    };
    let PostgresSqlStatementKind::LiteralExecute { execute } = &block.statements[0].facts else {
        panic!("EXECUTE USING");
    };
    assert_eq!(execute.using[0].sql, "now()");
    assert_expression(&source, &execute.using[0]);

    let expressions = statement_expressions(&parsed.statements[1]);
    let coalesce = expressions
        .iter()
        .find(|expr| expr.sql == "coalesce(now(), now())")
        .expect("INSERT source expression");
    assert_expression(&source, coalesce);
    assert!(
        coalesce
            .children
            .iter()
            .any(|child| child.sql == "now()" && child.span.is_some()),
        "nested call span"
    );

    let literal = statement_expressions(&parsed.statements[2])
        .into_iter()
        .find(|expr| expr.sql.contains('雪'))
        .expect("unicode literal");
    assert_expression(&source, &literal);
    let snow = literal.span.as_ref().unwrap();
    assert!(source.is_char_boundary(snow.start.offset));
    assert!(source[snow.start.offset..snow.end.offset].contains('雪'));

    let qualified = statement_expressions(&parsed.statements[3])
        .into_iter()
        .find(|expr| expr.sql == "schema.col")
        .expect("qualified reference");
    assert_expression(&source, &qualified);

    let selected = statement_expressions(&parsed.statements[4])
        .into_iter()
        .find(|expr| expr.sql == "coalesce(now(), 1)")
        .expect("SELECT source expression");
    assert_expression(&source, &selected);

    let rows = statement_expressions(&parsed.statements[5]);
    assert!(rows.iter().any(|expr| expr.sql == "1"));
    assert!(rows.iter().any(|expr| expr.sql.contains('雪')));
    for expr in &rows {
        assert_expression(&source, expr);
    }
}

fn statement_expressions(
    statement: &super::super::PostgresSqlStatement,
) -> Vec<PostgresSqlExpression> {
    let PostgresSqlStatementKind::Insert { insert } = &statement.facts else {
        return Vec::new();
    };
    let mut expressions = Vec::new();
    if let PostgresSqlInsertSource::Values { rows, .. } = &insert.source {
        for row in rows {
            expressions.extend(row.iter().cloned());
        }
    }
    if let PostgresSqlInsertColumnSources::Mapped { columns, .. } = &insert.column_sources {
        for column in columns {
            for source in &column.sources {
                match source {
                    PostgresSqlInsertSourceExpression::Values { expression, .. }
                    | PostgresSqlInsertSourceExpression::Select { expression, .. } => {
                        expressions.push(expression.clone());
                    }
                }
            }
        }
    }
    expressions
}

fn assert_expression(source: &str, expr: &PostgresSqlExpression) {
    assert_span(source, expr.span.as_ref(), &expr.sql);
    for child in &expr.children {
        assert_span(source, child.span.as_ref(), &child.sql);
    }
    assert_root(source, &expr.root);
}

fn assert_root(source: &str, root: &PostgresSqlExpressionRoot) {
    match root {
        PostgresSqlExpressionRoot::FunctionCall { arguments, .. } => {
            for argument in arguments {
                assert_span(source, argument.span.as_ref(), &argument.sql);
                assert_root(source, &argument.root);
            }
        }
        PostgresSqlExpressionRoot::Parenthesized { expression }
        | PostgresSqlExpressionRoot::Cast { expression, .. }
        | PostgresSqlExpressionRoot::Unary { expression, .. } => assert_root(source, expression),
        _ => {}
    }
}

fn assert_span(source: &str, span: Option<&super::super::PostgresSqlSpan>, sql: &str) {
    let Some(span) = span else {
        return;
    };
    assert!(
        source.is_char_boundary(span.start.offset) && source.is_char_boundary(span.end.offset),
        "{sql}"
    );
    assert_eq!(
        &source[span.start.offset..span.end.offset],
        sql,
        "span bytes for {sql}"
    );
}
