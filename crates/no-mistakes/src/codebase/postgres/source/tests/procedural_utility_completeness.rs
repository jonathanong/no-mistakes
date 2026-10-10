use super::super::PostgresSqlStatementKind;
use super::facts;

#[test]
fn classified_utilities_do_not_make_procedural_blocks_incomplete() {
    // These utility statements intentionally parse as `Other`; their classified occurrence
    // spans must keep the legacy completeness projection from rejecting the enclosing block.
    let facts = facts("procedural-utility-completeness.sql");
    let blocks = facts
        .statements
        .iter()
        .filter_map(|statement| match &statement.facts {
            PostgresSqlStatementKind::DoBlock { block } => Some(block),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 14);
    for (index, block) in blocks.iter().enumerate() {
        // Parsed DML retains the existing source-fact completeness semantics; execution
        // safety is represented separately by its DML occurrence kind.
        assert_eq!(block.complete, index < 5, "block {index}: {block:?}");
    }
    assert!(blocks[..5].iter().all(|block| block.diagnostics.is_empty()));
    assert!(
        blocks[0].occurrences[0].kind
            == super::super::PostgresSqlProceduralOccurrenceKind::ControlFlow
    );
    assert!(blocks[0].occurrences[0]
        .occurrences
        .iter()
        .any(|occurrence| {
            occurrence.kind == super::super::PostgresSqlProceduralOccurrenceKind::Utility
        }));
    assert!(blocks[1].occurrences.iter().any(|occurrence| {
        occurrence.kind == super::super::PostgresSqlProceduralOccurrenceKind::Utility
    }));
    assert_eq!(
        blocks[2].occurrences[0].kind,
        super::super::PostgresSqlProceduralOccurrenceKind::Utility
    );
    assert!(blocks[3].occurrences[0]
        .occurrences
        .iter()
        .any(|occurrence| {
            occurrence.kind == super::super::PostgresSqlProceduralOccurrenceKind::Utility
        }));
    assert!(blocks[4].occurrences[0].occurrences.iter().any(
        |occurrence| occurrence.kind == super::super::PostgresSqlProceduralOccurrenceKind::Dml
    ));
    assert!(blocks[5].occurrences.iter().any(|occurrence| {
        occurrence.kind == super::super::PostgresSqlProceduralOccurrenceKind::DynamicExecute
    }));
    for block in &blocks[6..] {
        assert!(block.occurrences.iter().any(|occurrence| {
            occurrence.kind == super::super::PostgresSqlProceduralOccurrenceKind::Unknown
        }));
    }
}
