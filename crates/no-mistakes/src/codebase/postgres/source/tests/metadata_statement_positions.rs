use super::{facts, fixture};
use crate::codebase::postgres::source::*;

fn without_spans(mut value: serde_json::Value) -> serde_json::Value {
    match &mut value {
        serde_json::Value::Object(fields) => {
            fields.remove("span");
            for value in fields.values_mut() {
                *value = without_spans(std::mem::take(value));
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                *value = without_spans(std::mem::take(value));
            }
        }
        _ => {}
    }
    value
}

#[test]
fn comment_relation_and_alias_queries_keep_baseline_wrapped_and_procedural_facts() {
    let sql = fixture("comment-join-alias-parity.sql");
    let result = facts("comment-join-alias-parity.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 13);
    for pair in result.statements[..10].chunks_exact(2) {
        let PostgresSqlStatementKind::Select { query: baseline } = &pair[0].facts else {
            panic!()
        };
        let PostgresSqlStatementKind::Wrapper { wrapper } = &pair[1].facts else {
            panic!()
        };
        let PostgresSqlStatementKind::Select { query } = &wrapper.statements[0].facts else {
            panic!()
        };
        assert_eq!(
            without_spans(serde_json::to_value(query).unwrap()),
            without_spans(serde_json::to_value(baseline).unwrap())
        );
        let child = &wrapper.statements[0];
        assert_eq!(
            child.sql,
            sql[child.span.start.offset..child.span.end.offset]
        );
        assert!(child.sql.contains("comment ON"));
    }
    let PostgresSqlStatementKind::Select { query: baseline } = &result.statements[0].facts else {
        panic!()
    };
    assert!(baseline.relations.iter().any(|relation| {
        relation
            .name
            .as_ref()
            .is_some_and(|name| name.sql == "comment")
    }));
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[10].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::Select { query } = &branches[0].statements[0].facts else {
        panic!()
    };
    assert_eq!(
        without_spans(serde_json::to_value(query).unwrap()),
        without_spans(serde_json::to_value(baseline).unwrap())
    );
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[11].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::Select { query } = &function.wrapper.statements[0].facts else {
        panic!()
    };
    assert_eq!(
        without_spans(serde_json::to_value(query).unwrap()),
        without_spans(serde_json::to_value(baseline).unwrap())
    );
    assert_eq!(result.statements[12].sql, "SELECT 99;");
    for statement in &result.statements {
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
    }
}

#[test]
fn begin_relation_alias_does_not_establish_a_prepared_comment_boundary() {
    let sql = fixture("comment-join-begin-control.sql");
    let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&sql);
    let parser = sqlparser::parser::Parser::new(&sqlparser::dialect::PostgreSqlDialect {})
        .with_tokens_with_locations(prepared.tokens);
    let locations = super::super::locations::Locations::new(&sql);
    let (_, comments) = super::super::metadata_preparation::prepare(parser, &locations);
    assert!(comments.is_empty());
}

#[test]
fn nested_wrapper_atomic_header_anchors_metadata_without_expanding_child_grammar() {
    let sql = fixture("comment-nested-wrapper-header.sql");
    let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&sql);
    let parser = sqlparser::parser::Parser::new(&sqlparser::dialect::PostgreSqlDialect {})
        .with_tokens_with_locations(prepared.tokens);
    let locations = super::super::locations::Locations::new(&sql);
    let (_, comments) = super::super::metadata_preparation::prepare(parser, &locations);
    assert_eq!(comments.len(), 2);
    let values = comments
        .values()
        .map(|comment| match &comment.facts {
            Some(Ok(PostgresSqlStatementKind::Comment { comment })) => {
                comment.comment.as_deref().unwrap()
            }
            _ => panic!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(values, ["nested explain", "nested prepare"]);
}
