use super::super::PostgresSqlStatementKind;
use super::{facts, fixture};

#[test]
fn procedural_body_occurrences_retain_origin_and_honest_control_flow_diagnostics() {
    let source = fixture("procedural.sql");
    let facts = facts("procedural.sql");
    assert_eq!(facts.statements.len(), 8);
    assert_eq!(facts.diagnostics.len(), 1);
    let blocks = facts
        .statements
        .iter()
        .filter_map(|statement| match &statement.facts {
            PostgresSqlStatementKind::DoBlock { block } => Some(block),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 7);
    assert!(blocks[0].complete);
    assert!(blocks[1].complete);
    let nested = &blocks[0].statements[0];
    assert_eq!(
        nested.sql,
        "ALTER TABLE \"Café\" ADD COLUMN \"雪\" integer;"
    );
    assert_eq!(
        &source[nested.span.start.offset..nested.span.end.offset],
        nested.sql
    );
    assert_eq!(nested.span.start.line, 2);
    for block in &blocks[2..5] {
        assert!(!block.complete);
        assert!(block.statements.is_empty());
        assert_eq!(block.diagnostics.len(), 1);
    }
    assert!(!blocks[5].complete);
    assert_eq!(blocks[5].statements.len(), 1);
    assert!(matches!(
        blocks[5].statements[0].facts,
        PostgresSqlStatementKind::CreateTable { .. }
    ));
    assert!(!blocks[6].complete);
    assert!(blocks[6].statements.is_empty());
}

#[test]
fn deeply_nested_procedural_programs_report_a_bounded_source_boundary() {
    let facts = facts("procedural-depth.sql");
    assert!(facts.diagnostics.is_empty());
    let mut statement = &facts.statements[0];
    for _ in 0..64 {
        let PostgresSqlStatementKind::DoBlock { block } = &statement.facts else {
            panic!()
        };
        statement = &block.statements[0];
    }
    let PostgresSqlStatementKind::DoBlock { block } = &statement.facts else {
        panic!()
    };
    assert!(!block.complete);
    assert!(block.statements.is_empty());
    assert!(block.diagnostics[0].message.contains("safety limit"));
}

#[test]
fn malformed_procedural_envelopes_preserve_neighbors_and_report_body_boundaries() {
    let facts = facts("procedural-invalid.sql");
    assert_eq!(facts.diagnostics.len(), 5);
    assert_eq!(facts.statements.len(), 9);
    let blocks = facts
        .statements
        .iter()
        .filter_map(|statement| match &statement.facts {
            PostgresSqlStatementKind::DoBlock { block } => Some(block),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 4);
    for block in blocks {
        assert!(!block.complete);
        assert!(block.statements.is_empty());
        assert_eq!(block.diagnostics.len(), 1);
        assert!(block.diagnostics[0].span.is_some());
    }
}

#[test]
fn procedural_projection_rejects_tokens_borrowed_from_a_different_source_owner() {
    use super::super::{locations::Locations, procedural, PostgresSqlSource};
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};
    let sql = fixture("unsupported.sql");
    let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&sql);
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens);
    let foreign = PostgresSqlSource {
        sql: fixture("procedural-foreign-source.sql"),
        file_name: None,
    };
    assert!(
        procedural::collect(&mut parser, &foreign, &Locations::new(&foreign.sql), 0)
            .unwrap_err()
            .contains("original source")
    );
    let prepared = crate::codebase::postgres::parse::prepare_postgres_tokens(&sql);
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(prepared.tokens);
    let empty = fixture("empty.sql");
    assert!(
        procedural::collect(&mut parser, &foreign, &Locations::new(&empty), 0)
            .unwrap_err()
            .contains("source span")
    );
}
