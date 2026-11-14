use super::super::RULE_ID;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::{
    EmbeddedSqlOptions, SqlObligationVerdict, SqlStatementFileFacts, SqlVerdictStatus,
};
use crate::codebase::rules::{PostgresFindings, RuleFinding, VariantFindingDedup};

/// How a columnless insert with unknown positions follows `unanalyzableSql`.
pub(super) enum PositionalUncertainty<'a> {
    Report,
    Advise(&'a dyn SqlFactContext, &'a EmbeddedSqlOptions),
    Ignore,
}

pub(super) fn emit_unproven(
    findings: &mut PostgresFindings,
    dedup: &mut VariantFindingDedup<'_>,
    statement: &SqlStatementFileFacts,
    source: Option<&str>,
    file: &str,
    sites: &[(usize, String, usize)],
    uncertainty: &PositionalUncertainty<'_>,
) {
    match uncertainty {
        PositionalUncertainty::Ignore => {}
        PositionalUncertainty::Report => {
            for (line, table, write_index) in sites {
                dedup.push(
                    findings,
                    statement,
                    SqlFactSite::Write(*write_index),
                    source,
                    positional_finding(file, *line, table),
                );
            }
        }
        PositionalUncertainty::Advise(facts, profile) => {
            for (line, table, _) in sites {
                facts.record_sql_advisory(advisory(
                    profile,
                    file,
                    source.unwrap_or(""),
                    *line,
                    table,
                ));
            }
        }
    }
}

fn positional_finding(file: &str, line: usize, table: &str) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!(
            "{file}:{line}: columnless insert into `{table}` cannot be checked because its \
column order is unknown; name the columns explicitly so a generated or \
trigger-maintained column can be ruled out, or set unanalyzableSql: ignore"
        ),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}

fn advisory(
    profile: &EmbeddedSqlOptions,
    file: &str,
    source: &str,
    line: usize,
    table: &str,
) -> SqlObligationVerdict {
    SqlObligationVerdict {
        profile: profile.clone(),
        rule: RULE_ID.to_string(),
        application: None,
        subcheck: "positional-column-order".to_string(),
        file: file.to_string(),
        span: line_span(source, line),
        status: SqlVerdictStatus::Abstain,
        advisory: true,
        evidence: None,
        known_prefix: None,
        helper_lines: Vec::new(),
        residual_reasons: vec![format!(
            "an explicit column list is required for `{table}` because its column order is unknown"
        )],
    }
}

fn line_span(source: &str, line: usize) -> (u32, u32) {
    let target = line.max(1);
    let mut start = 0usize;
    let mut current = 1usize;
    for (index, byte) in source.bytes().enumerate() {
        if byte != b'\n' {
            continue;
        }
        if current == target {
            return (start as u32, (index + 1) as u32);
        }
        current += 1;
        start = index + 1;
    }
    (start as u32, source.len() as u32)
}

#[cfg(test)]
mod tests {
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
}
