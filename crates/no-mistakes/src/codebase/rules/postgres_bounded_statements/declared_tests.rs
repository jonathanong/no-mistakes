use super::{
    plan, Gate, AMBIGUOUS, DUPLICATE, MISSING_BOUND, MISSING_JUSTIFICATION, NON_POSITIVE, UNMATCHED,
};

fn lines(source: &str, anchors: &[usize]) -> (Vec<usize>, Vec<usize>) {
    let planned = plan(Some(source), anchors);
    let mut gates: Vec<usize> = planned.gates.keys().copied().collect();
    gates.sort_unstable();
    let unmatched = planned.unmatched().iter().map(|(line, _)| *line).collect();
    (gates, unmatched)
}

#[test]
fn binds_the_statement_line_or_the_line_above_it() {
    let source = "\
-- no-mistakes-bound postgres-bounded-statements rows<=100: page of ids
SELECT id FROM candidates WHERE id = ANY($1::uuid[]);
SELECT id FROM orders WHERE id = ANY($1::uuid[]); -- no-mistakes-bound postgres-bounded-statements rows<=50: page of ids
";
    let (gates, unmatched) = lines(source, &[2, 3]);
    assert_eq!(gates, vec![2, 3]);
    assert!(unmatched.is_empty());
    match plan(Some(source), &[2, 3]).gate(2) {
        Some(Gate::Valid { rows, line, .. }) => {
            assert_eq!((*line, *rows), (1, 100));
        }
        other => panic!("expected the previous-line bound, got {other:?}"),
    }
}

#[test]
fn reports_unmatched_invalid_and_duplicate_declarations() {
    let source = "\
-- no-mistakes-bound postgres-bounded-statements rows<=10: not on a statement
-- no-mistakes-bound postgres-bounded-statements rows<=100
-- no-mistakes-bound postgres-bounded-statements: missing limit
SELECT id FROM candidates; -- no-mistakes-bound postgres-bounded-statements rows<=4: trailing
";
    let planned = plan(Some(source), &[4]);
    assert_eq!(
        planned.unmatched(),
        &[(1, UNMATCHED), (2, MISSING_JUSTIFICATION)]
    );
    match planned.gate(4) {
        Some(Gate::Blocked { findings }) => {
            assert_eq!(findings.as_slice(), &[(3, MISSING_BOUND), (4, DUPLICATE)]);
        }
        other => panic!("expected two declarations on one statement, got {other:?}"),
    }
}

#[test]
fn skips_sources_without_a_declaration() {
    assert!(plan(None, &[1]).is_empty());
    assert!(plan(Some("SELECT id FROM candidates;"), &[1]).is_empty());
}

#[test]
fn a_shared_anchor_is_ambiguous() {
    let source = "\
-- no-mistakes-bound postgres-bounded-statements rows<=40: two statements
SELECT id FROM orders; SELECT id FROM orders;
";
    let planned = plan(Some(source), &[2, 2]);
    match planned.gate(2) {
        Some(Gate::Invalid { line, message }) => {
            assert_eq!((*line, *message), (1, AMBIGUOUS));
        }
        other => panic!("expected an ambiguous declaration, got {other:?}"),
    }
    assert!(planned.unmatched().is_empty());
}

#[test]
fn a_trailing_comment_does_not_cover_the_next_line() {
    let source = "\
SELECT id FROM orders; -- no-mistakes-bound postgres-bounded-statements rows<=40: only this statement
SELECT id FROM orders;
";
    let planned = plan(Some(source), &[1, 2]);
    assert!(matches!(
        planned.gate(1),
        Some(Gate::Valid {
            line: 1,
            rows: 40,
            ..
        })
    ));
    assert!(planned.gate(2).is_none());
    assert!(planned.unmatched().is_empty());
}

#[test]
fn a_zero_row_declaration_is_not_a_bound() {
    let source = "\
-- no-mistakes-bound postgres-bounded-statements rows<=0: empty page
SELECT id FROM orders;
";
    match plan(Some(source), &[2]).gate(2) {
        Some(Gate::Invalid { line, message }) => {
            assert_eq!((*line, *message), (1, NON_POSITIVE));
        }
        other => panic!("expected rows<=0 to be rejected, got {other:?}"),
    }
}
