use super::super::*;

#[test]
fn synthetic_copy_terminators_do_not_create_fabricated_source_positions() {
    let facts = facts("copy-source.sql");
    assert_eq!(facts.diagnostics.len(), 1);
    assert!(facts.diagnostics[0].message.contains("source boundary"));
    assert_eq!(facts.statements.len(), 1);
    assert_eq!(facts.statements[0].ordinal, 1);
    assert_eq!(facts.statements[0].span.start.line, 4);
    assert_eq!(facts.statements[0].sql, "CREATE TABLE after_copy (id int);");
}

#[test]
fn missing_delimiters_are_diagnostics_and_empty_statements_have_no_ordinal() {
    let facts = facts("delimiters.sql");
    assert_eq!(facts.diagnostics.len(), 1);
    assert_eq!(facts.statements.len(), 1);
    assert_eq!(facts.statements[0].ordinal, 1);
    assert_eq!(facts.statements[0].span.start.line, 3);
}

#[test]
fn procedural_consumer_bodies_preserve_occurrences_without_execution_claims() {
    let facts = facts("unsupported.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    assert_eq!(facts.statements.len(), 2);
    let PostgresSqlStatementKind::DoBlock { block } = &facts.statements[0].facts else {
        panic!()
    };
    assert!(block.complete);
    assert_eq!(block.statements.len(), 2);
    assert_eq!(block.statements[0].span.start.line, 3);
    assert_eq!(block.statements[1].span.start.line, 4);
    assert!(matches!(
        block.statements[0].facts,
        PostgresSqlStatementKind::AlterTable { .. }
    ));
    assert_eq!(facts.statements[1].span.start.line, 6);
}

#[test]
fn drop_and_recreate_sources_keep_order_and_structured_dependencies() {
    let facts = facts("drops.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::Drop { drop } = &facts.statements[0].facts else {
        panic!()
    };
    assert!(drop.if_exists && drop.cascade);
    let PostgresSqlStatementKind::Drop { drop } = &facts.statements[1].facts else {
        panic!()
    };
    assert!(drop.restrict);
    let PostgresSqlStatementKind::Drop { drop } = &facts.statements[2].facts else {
        panic!()
    };
    assert!(drop.table.is_some());
    let PostgresSqlStatementKind::Drop { drop } = &facts.statements[3].facts else {
        panic!()
    };
    assert_eq!(drop.signatures, ["app.touch(UUID)"]);
    assert!(matches!(
        facts.statements[5].facts,
        PostgresSqlStatementKind::CreateView { .. }
    ));
}
use super::{facts, fixture};

#[test]
fn syntax_failure_preserves_neighboring_statements_and_source_identity() {
    let facts = facts("partial.sql");
    assert_eq!(
        facts
            .statements
            .iter()
            .map(|statement| statement.ordinal)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(facts.diagnostics.len(), 1);
    assert_eq!(facts.statements[1].span.start.line, 3);
    assert_eq!(facts.diagnostics[0].span.as_ref().unwrap().start.line, 2);
    let source = fixture("partial.sql");
    for statement in &facts.statements {
        assert_eq!(
            &source[statement.span.start.offset..statement.span.end.offset],
            statement.sql
        );
    }
}

#[test]
fn lexical_failure_and_empty_sources_return_structured_diagnostics() {
    let facts = facts("lexical.sql");
    assert_eq!(facts.statements.len(), 1);
    assert!(!facts.diagnostics.is_empty());
    let sources = [
        PostgresSqlSource {
            sql: fixture("empty.sql"),
            file_name: Some("empty.sql".into()),
        },
        PostgresSqlSource {
            sql: fixture("partial.sql"),
            file_name: None,
        },
    ];
    let batch = parse_postgres_sources(&sources);
    assert!(batch[0].statements.is_empty());
    assert_eq!(batch[1].statements.len(), 2);
}
