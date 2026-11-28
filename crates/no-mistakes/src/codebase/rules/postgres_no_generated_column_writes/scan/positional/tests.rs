use super::{
    call_anchors, emit_unproven, line_span, positional_finding, CallAnchor, PositionalUncertainty,
    WriteScan,
};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{
    EmbeddedSqlCall, EmbeddedSqlFileFacts, EmbeddedSqlOptions, SqlStatementFileFacts,
};
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
    emit(
        &mut findings,
        "writes.sql",
        &[(1, "votes".to_string(), 0)],
        None,
        &[],
        &PositionalUncertainty::Report,
    );
    assert_eq!(findings.len(), 1);
}

#[test]
fn advise_without_source_records_an_empty_span() {
    let facts = CheckFactMap::default();
    let profile = EmbeddedSqlOptions::default();
    emit(
        &mut PostgresFindings::default(),
        "writes.sql",
        &[(1, "votes".to_string(), 0)],
        None,
        &[],
        &PositionalUncertainty::Advise(&facts, &profile),
    );
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 1);
    assert_eq!(verdicts[0].span, (0, 0));
    assert_eq!(verdicts[0].subcheck, "positional-column-order");
    assert!(verdicts[0].residual_reasons[0].contains("`votes`"));
}

#[test]
fn advise_keeps_the_executor_call_span() {
    let facts = CheckFactMap::default();
    let profile = EmbeddedSqlOptions::default();
    let anchors = [Some(CallAnchor {
        span: (12, 40),
        helper_lines: vec![7],
    })];
    emit(
        &mut PostgresFindings::default(),
        "write.ts",
        &[(4, "votes".to_string(), 0)],
        Some("return write(`INSERT INTO votes VALUES ($1)`)\n"),
        &anchors,
        &PositionalUncertainty::Advise(&facts, &profile),
    );
    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts[0].span, (12, 40));
    assert_eq!(verdicts[0].helper_lines, vec![7]);
}

#[test]
fn call_anchors_follow_statement_calls_and_stop_at_the_prepared_count() {
    let mut embedded = embedded_file(vec![
        call_with_text("INSERT INTO votes VALUES ($1, $2)"),
        EmbeddedSqlCall::default(),
        call_with_text("INSERT INTO votes VALUES ($1)"),
    ]);
    embedded.call_spans = vec![(10, 20), (21, 22), (30, 48)];
    embedded.imported_call_lines.insert((10, 20), vec![4]);
    let first = call_anchors(&embedded, 1);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].as_ref().unwrap().span, (10, 20));
    assert_eq!(first[0].as_ref().unwrap().helper_lines, vec![4]);
    let padded = call_anchors(&embedded, 3);
    assert_eq!(padded[0].as_ref().unwrap().span, (10, 20));
    assert_eq!(padded[1].as_ref().unwrap().span, (30, 48));
    assert!(padded[1].as_ref().unwrap().helper_lines.is_empty());
    assert!(padded[2].is_none());
}

fn emit(
    findings: &mut PostgresFindings,
    file: &str,
    sites: &[(usize, String, usize)],
    source: Option<&str>,
    anchors: &[Option<CallAnchor>],
    uncertainty: &PositionalUncertainty<'_>,
) {
    emit_unproven(
        findings,
        &mut VariantFindingDedup::default(),
        &SqlStatementFileFacts::default(),
        file,
        sites,
        &WriteScan {
            source,
            anchors,
            uncertainty,
        },
        0,
    );
}

fn call_with_text(sql: &str) -> EmbeddedSqlCall {
    EmbeddedSqlCall {
        sql_text: Some(sql.to_string()),
        ..Default::default()
    }
}

fn embedded_file(calls: Vec<EmbeddedSqlCall>) -> EmbeddedSqlFileFacts {
    EmbeddedSqlFileFacts {
        lock_helper_summaries: Default::default(),
        imported_lock_helpers: Default::default(),
        path: "write.ts".into(),
        executor_bindings: Vec::new(),
        calls,
        call_spans: Vec::new(),
        exported_helpers: Default::default(),
        exported_constants: Default::default(),
        exported_programs: Default::default(),
        pending_imported: Vec::new(),
        imported_call_lines: Default::default(),
        source_comment_spans: Vec::new(),
        fragments: Vec::new(),
        fragment_variants: Vec::new(),
        fragment_sites: Vec::new(),
        call_fragments: Default::default(),
        param_append_fragments: Vec::new(),
        matched_factory_names: Vec::new(),
        matched_type_names: Vec::new(),
        pending_relative: Default::default(),
        undeclared_executor_types: Vec::new(),
        unproven_member_query_spans: Vec::new(),
        postgres_rule_ids: Vec::new(),
        record_unproven_spans: true,
    }
}
