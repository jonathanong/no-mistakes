use super::{plan, Gate, DUPLICATE, MISSING_BOUND, MISSING_JUSTIFICATION, UNMATCHED};

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
