use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn insert_sources_and_conflicts_retain_original_boundaries() {
    let sql = fixture("insert.sql");
    let result = facts("insert.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 13);
    let inserts = result
        .statements
        .iter()
        .map(|statement| {
            assert_eq!(
                statement.sql,
                sql[statement.span.start.offset..statement.span.end.offset]
            );
            let PostgresSqlStatementKind::Insert { insert } = &statement.facts else {
                panic!("INSERT facts expected")
            };
            insert
        })
        .collect::<Vec<_>>();
    assert!(inserts[0].columns_omitted);
    let PostgresSqlInsertSource::Values { rows, .. } = &inserts[0].source else {
        panic!("VALUES expected")
    };
    assert_eq!(rows[0][1].sql, "'ON CONFLICT; VALUES'");
    assert_eq!(
        inserts[1].table.as_ref().unwrap().parts[1].identity,
        "Accounts"
    );
    assert_eq!(inserts[1].alias.as_ref().unwrap().identity, "a");
    assert_eq!(inserts[1].columns[0].parts[0].identity, "ID");
    assert!(matches!(
        inserts[2].source,
        PostgresSqlInsertSource::DefaultValues
    ));
    assert!(matches!(
        inserts[3].source,
        PostgresSqlInsertSource::Select { .. }
    ));
    assert!(matches!(
        inserts[1].on_conflict.as_ref().unwrap().target,
        PostgresSqlConflictTarget::Omitted
    ));
    assert!(matches!(
        inserts[4].on_conflict.as_ref().unwrap().target,
        PostgresSqlConflictTarget::Columns { .. }
    ));
    assert!(matches!(
        inserts[5].on_conflict.as_ref().unwrap().target,
        PostgresSqlConflictTarget::Constraint { .. }
    ));
    let conflict = inserts[6].on_conflict.as_ref().unwrap();
    assert_eq!(conflict.predicate.as_ref().unwrap().sql, "id > 0");
    let predicate_span = conflict.predicate.as_ref().unwrap().span.as_ref().unwrap();
    assert_eq!(
        &sql[predicate_span.start.offset..predicate_span.end.offset],
        "id > 0"
    );
    let PostgresSqlConflictAction::DoUpdate {
        assignments,
        predicate,
    } = &conflict.action
    else {
        panic!("UPDATE expected")
    };
    assert_eq!(predicate.as_ref().unwrap().sql, "a.id > 0");
    assert_eq!(
        assignments[0].provenance,
        PostgresSqlInsertProvenance::ExcludedColumn
    );
    assert_eq!(
        assignments[1].provenance,
        PostgresSqlInsertProvenance::TargetColumn
    );
    assert!(inserts[6].complete);
    let PostgresSqlConflictAction::DoUpdate { assignments, .. } =
        &inserts[7].on_conflict.as_ref().unwrap().action
    else {
        panic!("UPDATE expected")
    };
    assert_eq!(
        assignments
            .iter()
            .map(|assignment| assignment.provenance)
            .collect::<Vec<_>>(),
        vec![
            PostgresSqlInsertProvenance::TargetColumn,
            PostgresSqlInsertProvenance::Literal,
            PostgresSqlInsertProvenance::Placeholder,
            PostgresSqlInsertProvenance::Unresolved,
            PostgresSqlInsertProvenance::Unresolved,
            PostgresSqlInsertProvenance::ExcludedColumn
        ]
    );
    assert!(!inserts[7].complete);
    assert_eq!(inserts[7].diagnostics.len(), 1);
    assert!(inserts[8].on_conflict.as_ref().unwrap().predicate.is_some());
    assert!(inserts[10]
        .on_conflict
        .as_ref()
        .unwrap()
        .predicate
        .is_some());
    assert!(!inserts[11].complete);
    assert!(!inserts[12].complete);
    assert!(!inserts[9].complete); // Tuple RHS provenance is deliberately unresolved.
}

#[test]
fn unsupported_conflict_predicates_fail_closed_and_preserve_procedural_grammar() {
    let result = facts("insert-partial.sql");
    assert_eq!(result.diagnostics.len(), 3, "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 6);
    assert_eq!(
        result
            .statements
            .iter()
            .map(|statement| statement.ordinal)
            .collect::<Vec<_>>(),
        vec![0, 2, 4, 6, 7, 8]
    );
    for statement in &result.statements[4..] {
        let PostgresSqlStatementKind::DoBlock { block } = &statement.facts else {
            panic!("DO facts expected")
        };
        assert!(block.complete, "{:?}", block.diagnostics);
    }
}

#[test]
fn malformed_conflict_clauses_report_diagnostics_and_keep_the_next_insert() {
    let result = facts("insert-errors.sql");
    assert_eq!(result.diagnostics.len(), 10, "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 1);
    assert_eq!(result.statements[0].ordinal, 10);
    let PostgresSqlStatementKind::Insert { insert } = &result.statements[0].facts else {
        panic!("INSERT expected")
    };
    assert!(insert.complete);
}

#[test]
fn unsupported_parser_shapes_never_imply_complete_insert_facts() {
    use sqlparser::{
        dialect::{ClickHouseDialect, Dialect, GenericDialect},
        parser::Parser,
    };
    for (fixture_name, dialect) in [
        ("insert-projection.sql", &GenericDialect {} as &dyn Dialect),
        (
            "insert-projection-function.sql",
            &ClickHouseDialect {} as &dyn Dialect,
        ),
    ] {
        let sql = fixture(fixture_name);
        let locations = super::super::locations::Locations::new(&sql);
        let statements = Parser::parse_sql(dialect, &sql).unwrap();
        for statement in statements {
            let sqlparser::ast::Statement::Insert(insert) = statement else {
                panic!("INSERT expected")
            };
            let facts = super::super::insert::project(&insert, None, &locations);
            assert!(!facts.complete);
            assert_eq!(facts.diagnostics.len(), 1);
        }
    }
}

#[test]
fn truncated_insert_conflict_has_a_bounded_diagnostic() {
    let result = facts("insert-truncated.sql");
    assert!(result.statements.is_empty());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].span.as_ref().unwrap().end.offset,
        fixture("insert-truncated.sql").len()
    );
}
