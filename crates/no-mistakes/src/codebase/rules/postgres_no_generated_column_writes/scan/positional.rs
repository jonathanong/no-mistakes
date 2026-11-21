use super::super::RULE_ID;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::{
    EmbeddedSqlFileFacts, EmbeddedSqlOptions, SqlObligationVerdict, SqlStatementFileFacts,
    SqlVerdictStatus,
};
use crate::codebase::rules::{PostgresFindings, RuleFinding, VariantFindingDedup};

/// Executor call that owns one prepared statement, shared with policy verdicts.
#[derive(Clone)]
pub(super) struct CallAnchor {
    pub(super) span: (u32, u32),
    pub(super) helper_lines: Vec<u32>,
}

/// One anchor per prepared statement, walking calls the way policy verdicts do.
pub(super) fn call_anchors(
    embedded: &EmbeddedSqlFileFacts,
    statements: usize,
) -> Vec<Option<CallAnchor>> {
    let mut anchors = Vec::with_capacity(statements);
    for (call, span) in embedded.calls.iter().zip(&embedded.call_spans) {
        let helper_lines = embedded
            .imported_call_lines
            .get(span)
            .cloned()
            .unwrap_or_default();
        for version in call.statement_calls() {
            if version.sql_text.is_none() {
                continue;
            }
            if anchors.len() == statements {
                return anchors;
            }
            anchors.push(Some(CallAnchor {
                span: *span,
                helper_lines: helper_lines.clone(),
            }));
        }
    }
    anchors.resize(statements, None);
    anchors
}

/// How a columnless insert with unknown positions follows `unanalyzableSql`.
pub(super) enum PositionalUncertainty<'a> {
    Report,
    Advise(&'a dyn SqlFactContext, &'a EmbeddedSqlOptions),
    Ignore,
}

/// Source text, executor anchors, and the column-order policy for one file.
pub(super) struct WriteScan<'a, 'b> {
    pub(super) source: Option<&'a str>,
    pub(super) anchors: &'a [Option<CallAnchor>],
    pub(super) uncertainty: &'a PositionalUncertainty<'b>,
}

pub(super) fn emit_unproven(
    findings: &mut PostgresFindings,
    dedup: &mut VariantFindingDedup<'_>,
    statement: &SqlStatementFileFacts,
    file: &str,
    sites: &[(usize, String, usize)],
    scan: &WriteScan<'_, '_>,
    statement_index: usize,
) {
    let anchor = scan.anchors.get(statement_index).and_then(Option::as_ref);
    match scan.uncertainty {
        PositionalUncertainty::Ignore => {}
        PositionalUncertainty::Report => {
            for (line, table, write_index) in sites {
                dedup.push(
                    findings,
                    statement,
                    SqlFactSite::Write(*write_index),
                    scan.source,
                    positional_finding(file, *line, table),
                );
            }
        }
        PositionalUncertainty::Advise(facts, profile) => {
            for (line, table, _) in sites {
                facts.record_sql_advisory(advisory(
                    profile,
                    file,
                    scan.source.unwrap_or(""),
                    *line,
                    table,
                    anchor,
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
    anchor: Option<&CallAnchor>,
) -> SqlObligationVerdict {
    SqlObligationVerdict {
        profile: profile.clone(),
        rule: RULE_ID.to_string(),
        application: None,
        subcheck: "positional-column-order".to_string(),
        file: file.to_string(),
        span: anchor
            .map(|anchor| anchor.span)
            .unwrap_or_else(|| line_span(source, line)),
        status: SqlVerdictStatus::Abstain,
        advisory: true,
        evidence: None,
        known_prefix: None,
        helper_lines: anchor
            .map(|anchor| anchor.helper_lines.clone())
            .unwrap_or_default(),
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

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
