use super::super::PostgresSqlSpan;
use super::procedural_occurrences::{block, kinds};

fn slice<'a>(sql: &'a str, span: &PostgresSqlSpan) -> &'a str {
    &sql[span.start.offset..span.end.offset]
}

#[test]
fn quoted_loop_closing_label_is_one_control_flow_occurrence() {
    // NULL may stay an unknown child. The quoted closing label must not be another occurrence.
    let (sql, parsed) = block(r#"DO $$ BEGIN <<"Retry">> LOOP NULL; END LOOP "Retry"; END $$;"#);
    assert_eq!(parsed.occurrences.len(), 1);
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Unknown\"]"]);
    assert!(slice(&sql, &parsed.occurrences[0].span).contains(r#"END LOOP "Retry""#));
    let nested = &parsed.occurrences[0].occurrences;
    assert_eq!(nested.len(), 1);
    assert!(slice(&sql, &nested[0].span).contains("NULL"));
    assert!(!slice(&sql, &nested[0].span).contains("Retry"));
    assert!(!parsed.complete);
    assert!(parsed.statements.is_empty());
}

#[test]
fn unquoted_and_unlabeled_loop_closing_labels_stay_one_control_flow() {
    let (sql, unquoted) = block("DO $$ BEGIN <<retry>> LOOP NULL; END LOOP retry; END $$;");
    assert_eq!(unquoted.occurrences.len(), 1);
    assert_eq!(kinds(&unquoted.occurrences), ["ControlFlow[\"Unknown\"]"]);
    assert!(slice(&sql, &unquoted.occurrences[0].span).contains("END LOOP retry"));

    let (_, unlabeled) = block("DO $$ BEGIN LOOP NULL; END LOOP; END $$;");
    assert_eq!(unlabeled.occurrences.len(), 1);
    assert_eq!(kinds(&unlabeled.occurrences), ["ControlFlow[\"Unknown\"]"]);
}

#[test]
fn quoted_word_in_statement_position_is_not_that_keyword() {
    let (_, quoted) = block(r#"DO $$ BEGIN "INSERT" INTO t VALUES (1); END $$;"#);
    assert_eq!(quoted.occurrences.len(), 1);
    assert_eq!(kinds(&quoted.occurrences), ["Unknown"]);
    assert!(!format!("{:?}", quoted.occurrences).contains("Dml"));
}
