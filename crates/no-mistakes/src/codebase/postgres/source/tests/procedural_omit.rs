use super::super::procedural_omit::{non_sql_marks, sql_occurrence};
use super::super::{
    PostgresSqlPosition, PostgresSqlProceduralOccurrence, PostgresSqlProceduralOccurrenceKind,
    PostgresSqlSpan, PostgresSqlStatementKind,
};
use super::procedural_occurrences::block;

fn kinds(occurrences: &[PostgresSqlProceduralOccurrence]) -> Vec<String> {
    occurrences
        .iter()
        .map(|occurrence| {
            let nested = kinds(&occurrence.occurrences);
            if nested.is_empty() {
                format!("{:?}", occurrence.kind)
            } else {
                format!("{:?}{nested:?}", occurrence.kind)
            }
        })
        .collect()
}

#[test]
fn raise_between_creates_keeps_both_statements() {
    let (sql, parsed) = block(
        "DO $$ BEGIN CREATE TABLE a(id int); RAISE NOTICE 'x'; CREATE TABLE b(id int); END $$;",
    );
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        kinds(&parsed.occurrences),
        ["Utility", "ControlFlow", "Utility"]
    );
    let raise =
        &sql[parsed.occurrences[1].span.start.offset..parsed.occurrences[1].span.end.offset];
    assert!(raise.contains("RAISE NOTICE 'x'"));
    assert!(!raise.contains("CREATE"));
    assert_eq!(parsed.statements.len(), 2);
    assert!(parsed.statements[0].sql.contains("CREATE TABLE a"));
    assert!(parsed.statements[1].sql.contains("CREATE TABLE b"));
    assert!(matches!(
        parsed.statements[0].facts,
        PostgresSqlStatementKind::CreateTable { .. }
    ));
    assert!(matches!(
        parsed.statements[1].facts,
        PostgresSqlStatementKind::CreateTable { .. }
    ));
}

#[test]
fn escaped_raise_between_creates_uses_original_offsets() {
    let (_, parsed) = block(
        "DO 'BEGIN CREATE TABLE a(id int); RAISE NOTICE ''x''; CREATE TABLE b(id int); END';",
    );
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        kinds(&parsed.occurrences),
        ["Utility", "ControlFlow", "Utility"]
    );
    assert_eq!(parsed.statements.len(), 2);
    assert!(parsed.statements[0].sql.contains("CREATE TABLE a"));
    assert!(parsed.statements[1].sql.contains("CREATE TABLE b"));
}

#[test]
fn create_beside_a_loop_stays_incomplete_with_the_static_dml_diagnostic() {
    let (_, parsed) = block(
        "DO $$ BEGIN CREATE TABLE a(id int); FOR i IN 1..2 LOOP INSERT INTO a VALUES (i); END LOOP; END $$;",
    );
    assert!(!parsed.complete);
    assert_eq!(parsed.statements.len(), 1);
    assert!(parsed.statements[0].sql.contains("CREATE TABLE a"));
    assert!(parsed
        .statements
        .iter()
        .all(|statement| !statement.sql.contains("INSERT")));
    assert_eq!(
        kinds(&parsed.occurrences),
        ["Utility", "ControlFlow[\"Dml\"]"]
    );
    assert!(parsed.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Static DML is a source occurrence, not an executed statement")
    }));
}

fn at(offset: usize) -> PostgresSqlPosition {
    PostgresSqlPosition {
        offset,
        line: 1,
        column: offset + 1,
    }
}

fn occurrence(
    kind: PostgresSqlProceduralOccurrenceKind,
    start: usize,
    end: usize,
    occurrences: Vec<PostgresSqlProceduralOccurrence>,
) -> PostgresSqlProceduralOccurrence {
    PostgresSqlProceduralOccurrence {
        kind,
        span: PostgresSqlSpan {
            start: at(start),
            end: at(end),
        },
        occurrences,
    }
}

fn overlaps_slice(
    start: usize,
    end: usize,
    occurrences: &[PostgresSqlProceduralOccurrence],
) -> bool {
    occurrences.iter().any(|occurrence| {
        !matches!(
            occurrence.kind,
            PostgresSqlProceduralOccurrenceKind::Utility | PostgresSqlProceduralOccurrenceKind::Dml
        ) && start < occurrence.span.end.offset
            && occurrence.span.start.offset < end
    })
}

#[test]
fn sql_kinds_stay_utility_or_dml() {
    use PostgresSqlProceduralOccurrenceKind::{ControlFlow, Dml, DynamicExecute, Unknown, Utility};
    assert!(sql_occurrence(Utility));
    assert!(sql_occurrence(Dml));
    assert!(!sql_occurrence(ControlFlow));
    assert!(!sql_occurrence(DynamicExecute));
    assert!(!sql_occurrence(Unknown));
}

#[test]
fn many_tokens_against_many_spans_do_not_nest_loops() {
    use PostgresSqlProceduralOccurrenceKind::{ControlFlow, Dml, DynamicExecute, Unknown, Utility};
    const N: usize = 64;
    const ORIGIN: usize = 8;
    const STRIDE: usize = 4;
    const LEN: usize = N * STRIDE;
    let mut occurrences = vec![
        occurrence(ControlFlow, 0, 4, Vec::new()),
        occurrence(Unknown, ORIGIN + LEN + 2, ORIGIN + LEN + 6, Vec::new()),
    ];
    for index in 0..N {
        let base = ORIGIN + index * STRIDE;
        let sql_kind = if index % 2 == 0 { Utility } else { Dml };
        let nested_kind = [ControlFlow, DynamicExecute, Unknown][index % 3];
        // Nested non-SQL covers the parent. The parent kind decides omission.
        occurrences.push(occurrence(
            sql_kind,
            base,
            base + 2,
            vec![occurrence(nested_kind, base, base + 2, Vec::new())],
        ));
        let non_sql = [ControlFlow, DynamicExecute, Unknown][index % 3];
        occurrences.push(occurrence(non_sql, base + 2, base + 4, Vec::new()));
    }
    // Overlaps the first control-flow span so the union, not a toggle, is marked.
    occurrences.push(occurrence(ControlFlow, ORIGIN + 2, ORIGIN + 3, Vec::new()));
    assert!(occurrences
        .iter()
        .any(|occurrence| !occurrence.occurrences.is_empty()));

    let marks = non_sql_marks(ORIGIN, LEN, &occurrences);
    assert_eq!(
        marks.span_passes(),
        occurrences.len(),
        "one pass over the slice, not its nested children or each token"
    );
    assert_eq!(marks.body_passes(), LEN, "one pass over the body");
    assert!(!marks.overlaps(ORIGIN, ORIGIN + 2), "utility bytes stay");
    assert!(
        marks.overlaps(ORIGIN + 2, ORIGIN + 4),
        "control-flow bytes drop"
    );
    assert!(
        !marks.overlaps(ORIGIN + 2, ORIGIN + 2),
        "a point on the span start is outside the half-open predicate"
    );

    let mut tokens = vec![(4, 9), (ORIGIN + LEN - 1, ORIGIN + LEN + 5)];
    for index in 0..N {
        let base = ORIGIN + index * STRIDE;
        tokens.push((base, base + 2));
        tokens.push((base + 2, base + 4));
        tokens.push((base + 1, base + 3));
        tokens.push((base + 2, base + 2));
    }
    let probed = marks.token_checks();
    for (start, end) in &tokens {
        assert_eq!(
            marks.overlaps(*start, *end),
            overlaps_slice(*start, *end, &occurrences),
            "token {start}..{end}"
        );
    }
    assert_eq!(
        marks.token_checks() - probed,
        tokens.len(),
        "one constant-time check per token"
    );
    let nested = occurrences
        .iter()
        .map(|occurrence| occurrence.occurrences.len())
        .sum::<usize>();
    assert!(nested > 0);
    assert!(occurrences.len() + LEN + tokens.len() < occurrences.len() * tokens.len());
    assert!(
        marks.span_passes() + marks.body_passes() + marks.token_checks()
            < occurrences.len() * tokens.len()
    );
}
