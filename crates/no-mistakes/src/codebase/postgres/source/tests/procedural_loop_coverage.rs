use super::super::{
    PostgresSqlProceduralOccurrence, PostgresSqlProceduralOccurrenceKind, PostgresSqlStatementKind,
};
use super::{facts, fixture};

fn utilities<'a>(
    occurrences: &'a [PostgresSqlProceduralOccurrence],
    result: &mut Vec<&'a PostgresSqlProceduralOccurrence>,
) {
    for occurrence in occurrences {
        if occurrence.kind == PostgresSqlProceduralOccurrenceKind::Utility {
            result.push(occurrence);
        }
        utilities(&occurrence.occurrences, result);
    }
}

#[test]
fn procedural_loop_utilities_require_reachable_typed_statements() {
    let sql = fixture("procedural-loop-coverage.sql");
    let facts = facts("procedural-loop-coverage.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    assert_eq!(facts.statements.len(), 21);
    for (index, statement) in facts.statements.iter().enumerate() {
        let PostgresSqlStatementKind::DoBlock { block } = &statement.facts else {
            panic!()
        };
        assert_eq!(
            block.complete,
            matches!(index, 16 | 19 | 20),
            "block {index}: {block:?}"
        );
        if block.complete {
            assert!(block.diagnostics.is_empty(), "block {index}: {block:?}");
        } else {
            assert!(!block.diagnostics.is_empty(), "block {index}: {block:?}");
        }
        let mut found = Vec::new();
        utilities(&block.occurrences, &mut found);
        for occurrence in found {
            let span = &occurrence.span;
            assert!(span.start.offset < span.end.offset);
            let text = &sql[span.start.offset..span.end.offset];
            assert!(
                text.starts_with("CREATE")
                    || text.starts_with("EXECUTE")
                    || text.starts_with("LOCK")
                    || text.starts_with("DO"),
                "{text}"
            );
        }
        if matches!(index, 0..=3 | 5..=7 | 11 | 15 | 17) {
            let notes = block
                .diagnostics
                .iter()
                .filter(|note| note.message.contains("no typed statement projection"))
                .collect::<Vec<_>>();
            assert_eq!(
                notes.len(),
                if index == 5 { 2 } else { 1 },
                "block {index}: {block:?}"
            );
            for note in notes {
                let span = note.span.as_ref().unwrap();
                assert!(sql[span.start.offset..span.end.offset].ends_with(';') || index == 15);
            }
        }
    }
    let PostgresSqlStatementKind::DoBlock { block } = &facts.statements[6].facts else {
        panic!()
    };
    assert_eq!(block.statements.len(), 2);
    assert!(block.statements[0]
        .sql
        .starts_with("CREATE TABLE before_loop"));
    assert!(block.statements[1]
        .sql
        .starts_with("CREATE TABLE after_loop"));
}

#[test]
fn typed_if_branches_keep_complete_ordered_global_spans() {
    let sql = fixture("procedural-loop-coverage.sql");
    let facts = facts("procedural-loop-coverage.sql");
    let PostgresSqlStatementKind::DoBlock { block } = &facts.statements[16].facts else {
        panic!()
    };
    assert!(block.complete);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    assert_eq!(branches.len(), 3);
    for (branch, name) in branches
        .iter()
        .zip(["branch_first", "branch_second", "branch_third"])
    {
        let statement = &branch.statements[0];
        assert!(matches!(
            statement.facts,
            PostgresSqlStatementKind::CreateTable { .. }
        ));
        assert!(statement.sql.contains(name));
        assert_eq!(
            &sql[statement.span.start.offset..statement.span.end.offset],
            statement.sql
        );
        assert!(branch.span.start.offset <= statement.span.start.offset);
        assert!(branch.span.end.offset >= statement.span.end.offset);
    }
    assert!(branches[0].condition.is_some());
    assert!(branches[1].condition.is_some());
    assert!(branches[2].condition.is_none());
    assert!(branches
        .windows(2)
        .all(|pair| pair[0].span.end.offset <= pair[1].span.start.offset));
}
