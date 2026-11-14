use super::{emit_unproven, line_span, positional_finding, PositionalUncertainty};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{EmbeddedSqlOptions, SqlStatementFileFacts};
use crate::codebase::rules::{PostgresFindings, VariantFindingDedup};

#[test]
fn line_span_reads_a_middle_line_and_the_unterminated_last_line() {
    assert_eq!(line_span("one\ntwo\n", 1), (0, 4));
    assert_eq!(line_span("one\ntwo", 2), (4, 7));
}

#[test]
fn report_names_the_table_and_asks_for_a_column_list() {
    let finding = positional_finding("writes.sql", 1, "votes");
    assert_eq!(finding.target.as_deref(), Some("unanalyzable"));
    assert!(finding.message.contains("columnless insert into `votes`"));
    assert!(finding.message.contains("name the columns explicitly"));
    let mut findings = PostgresFindings::default();
    emit_unproven(
        &mut findings,
        &mut VariantFindingDedup::default(),
        &SqlStatementFileFacts::default(),
        None,
        "writes.sql",
        &[(1, "votes".to_string(), 0)],
        &PositionalUncertainty::Report,
    );
    assert_eq!(findings.len(), 1);
}

#[test]
fn advise_without_source_records_an_empty_span() {
    let facts = CheckFactMap::default();
    let profile = EmbeddedSqlOptions::default();
    emit_unproven(
        &mut PostgresFindings::default(),
        &mut VariantFindingDedup::default(),
        &SqlStatementFileFacts::default(),
        None,
        "writes.sql",
        &[(1, "votes".to_string(), 0)],
        &PositionalUncertainty::Advise(&facts, &profile),
    );
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 1);
    assert_eq!(verdicts[0].span, (0, 0));
    assert_eq!(verdicts[0].subcheck, "positional-column-order");
    assert!(verdicts[0].residual_reasons[0].contains("`votes`"));
}
