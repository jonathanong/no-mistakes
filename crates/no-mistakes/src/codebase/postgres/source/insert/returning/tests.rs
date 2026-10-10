use super::cte_column_sources;
use crate::codebase::postgres::source::{locations::Locations, types::*};
use sqlparser::ast::Spanned;

#[test]
fn delegated_source_variant_is_not_used_for_an_unsupported_cte_source() {
    let locations = Locations::new("INSERT INTO t(id) VALUES (1) RETURNING id");
    let sources = cte_column_sources(
        &[],
        None,
        &PostgresSqlCteInsertSource::Unsupported {
            reason: "INSERT source".into(),
        },
        &[],
        &locations,
    );
    assert!(matches!(
        sources,
        PostgresSqlInsertColumnSources::Unsupported {
            reason: PostgresSqlInsertColumnSourcesReason::ColumnsOmitted,
            ..
        }
    ));
    let defaults = cte_column_sources(
        &[],
        None,
        &PostgresSqlCteInsertSource::DefaultValues,
        &[],
        &locations,
    );
    assert!(matches!(
        defaults,
        PostgresSqlInsertColumnSources::Unsupported {
            reason: PostgresSqlInsertColumnSourcesReason::ColumnsOmitted,
            ..
        }
    ));
}

#[test]
fn prepared_statement_tokens_keep_returning_cte_call_trivia() {
    let sql = "INSERT INTO t(id) SELECT now /*keep*/ () RETURNING id";
    let mut parser = sqlparser::parser::Parser::new(&sqlparser::dialect::PostgreSqlDialect {})
        .try_with_sql(sql)
        .expect("sql");
    let sqlparser::ast::Statement::Insert(insert) = parser.parse_statement().expect("statement")
    else {
        panic!("INSERT expected");
    };
    let mut tokens = Vec::new();
    for index in 0.. {
        let token = parser.token_at(index);
        if matches!(token.token, sqlparser::tokenizer::Token::EOF) {
            break;
        }
        tokens.push(token.clone());
    }
    let locations = Locations::new(sql);
    let source = PostgresSqlCteInsertSource::Select {
        query_scope_id: 0,
        span: locations.span(insert.source.as_ref().expect("source").span()),
    };
    let kept = cte_column_sources(
        &insert.columns,
        insert.source.as_deref(),
        &source,
        &tokens,
        &locations,
    );
    assert_eq!(source_text(&kept, sql), "now /*keep*/ ()");
    let dropped = cte_column_sources(
        &insert.columns,
        insert.source.as_deref(),
        &source,
        &[],
        &locations,
    );
    assert!(matches!(
        dropped,
        PostgresSqlInsertColumnSources::Mapped {
            complete: false,
            ..
        }
    ));
}

fn source_text(sources: &PostgresSqlInsertColumnSources, sql: &str) -> String {
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
    let span = expression.span.as_ref().expect("source span");
    sql[span.start.offset..span.end.offset].to_string()
}
