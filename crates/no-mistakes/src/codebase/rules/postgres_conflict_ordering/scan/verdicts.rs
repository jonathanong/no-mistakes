use super::{analysis, has_safe_directive, CompiledOptions, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{
    combine_sql_verdicts, sql_evidence_for_call, EmbeddedSqlCall, EmbeddedSqlFileFacts,
    EmbeddedSqlKind, SchemaCatalog, SqlBranchJudgment, SqlConflictInsertFact, SqlEvidenceKind,
    SqlObligationVerdict, SqlStatementFileFacts, SqlVerdictStatus,
};
use crate::fx::FxHashMap;
use std::borrow::Cow;

mod prefix;

pub(super) fn record(
    file: &str,
    source: &str,
    embedded: &EmbeddedSqlFileFacts,
    opts: &CompiledOptions,
    catalog: &SchemaCatalog,
    prepared: &FxHashMap<(usize, usize), &SqlStatementFileFacts>,
    facts: &dyn SqlFactContext,
) {
    if !facts.wants_sql_verdicts() {
        return;
    }
    for (call_index, original) in embedded.calls.iter().enumerate() {
        let exempt = has_safe_directive(
            source,
            original.line,
            original.sql_text.as_deref().unwrap_or_default(),
            &opts.safe_directive,
        );
        let mut statuses = Vec::new();
        let mut residual_reasons = Vec::new();
        let mut positive_survives_unknown = false;
        for (version_index, version) in original.statement_calls().enumerate() {
            let judgment = judge(
                file,
                source,
                &version,
                prepared.get(&(call_index, version_index)).copied(),
                opts,
                catalog,
            );
            positive_survives_unknown |= judgment.status == SqlVerdictStatus::Violation
                && (judgment.survives_unknown || !version.variants.is_empty());
            statuses.push(judgment.status);
            if let Some(reason) = judgment.reason {
                residual_reasons.push(reason.to_string());
            }
        }
        let exhaustive = exempt
            || !original.variants_truncated()
                && (original.kind != EmbeddedSqlKind::Dynamic || !original.variants.is_empty());
        if !exhaustive {
            residual_reasons.push("opaque-sql-tail".to_string());
        }
        residual_reasons.sort();
        residual_reasons.dedup();
        let status = if exempt {
            SqlVerdictStatus::ProvenClear
        } else {
            combine_sql_verdicts(&statuses, exhaustive, positive_survives_unknown)
        };
        if original.variants_truncated()
            && opts.advise_unanalyzable
            && !exempt
            && status == SqlVerdictStatus::Violation
        {
            facts.record_sql_advisory(SqlObligationVerdict {
                profile: opts.embedded.clone(),
                rule: RULE_ID.to_string(),
                application: None,
                subcheck: "canonical-conflict-order-residual".to_string(),
                file: file.to_string(),
                span: embedded.call_spans[call_index],
                status: SqlVerdictStatus::Abstain,
                advisory: true,
                evidence: None,
                known_prefix: original.sql_text.clone(),
                helper_lines: Vec::new(),
                residual_reasons: vec!["opaque-sql-tail".to_string()],
            });
        }
        facts.record_sql_verdict(SqlObligationVerdict {
            profile: opts.embedded.clone(),
            rule: RULE_ID.to_string(),
            application: None,
            subcheck: "canonical-conflict-order".to_string(),
            file: file.to_string(),
            span: embedded.call_spans[call_index],
            status,
            advisory: opts.advise_unanalyzable,
            evidence: if exempt {
                Some(SqlEvidenceKind::PolicyExemption)
            } else {
                sql_evidence_for_call(original, status)
            },
            known_prefix: original
                .is_unanalyzable()
                .then(|| original.sql_text.clone())
                .flatten(),
            helper_lines: embedded
                .imported_call_lines
                .get(&embedded.call_spans[call_index])
                .cloned()
                .unwrap_or_default(),
            residual_reasons,
        });
    }
}

fn judge(
    file: &str,
    source: &str,
    call: &EmbeddedSqlCall,
    prepared: Option<&SqlStatementFileFacts>,
    opts: &CompiledOptions,
    catalog: &SchemaCatalog,
) -> SqlBranchJudgment {
    let sql = call.sql_text.as_deref().unwrap_or_default();
    if has_safe_directive(source, call.line, sql, &opts.safe_directive) {
        return SqlBranchJudgment::clear();
    }
    if call.kind == EmbeddedSqlKind::Dynamic && call.variants.is_empty() {
        // Conflict-only preparation omits this call. Judge the known prefix here.
        return prefix::dynamic_prefix_judgment(call, catalog);
    }
    if !analysis::contains_insert_conflict(sql) {
        return if call.sql_text.is_some() {
            SqlBranchJudgment::clear()
        } else {
            SqlBranchJudgment::abstain("unresolved-sql-fragment")
        };
    }
    let Some(statement) = prepared else {
        return SqlBranchJudgment::abstain("missing-prepared-sql-facts");
    };
    let Some(locations) = statement.variant_locations.as_ref() else {
        return SqlBranchJudgment::abstain("missing-prepared-sql-facts");
    };
    let prepared_conflicts = super::gate::prepared_conflicts(locations, statement.parse_failed);
    if statement.parse_failed && prepared_conflicts.conflicts.is_empty() {
        return SqlBranchJudgment::abstain("invalid-sql");
    }
    let inserts: Cow<'_, [SqlConflictInsertFact]> = Cow::Borrowed(prepared_conflicts.conflicts);
    if inserts.is_empty() {
        return SqlBranchJudgment::abstain("unresolved-conflict-statement");
    }
    let findings = analysis::findings_for_inserts(file, call.line as usize, &inserts, catalog);
    let mut uncertainty = false;
    for (_, finding) in &findings {
        let target = finding.target.as_deref().unwrap_or_default();
        if matches!(
            target,
            "unresolved-arbiter" | "ambiguous-arbiter" | "unresolved-source-order"
        ) {
            uncertainty = true;
        }
    }
    // A later order finding keeps the branch dependent on an open tail.
    if let Some(survives) = prefix::targetless_survives(&findings) {
        return SqlBranchJudgment::violation(survives);
    }
    if statement.parse_failed && prepared_conflicts.unanalyzable {
        return SqlBranchJudgment::abstain("invalid-sql");
    }
    if uncertainty {
        SqlBranchJudgment::abstain("unresolved-conflict-catalog-or-source")
    } else {
        SqlBranchJudgment::clear()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod allocation_tests;

#[cfg(test)]
mod projection_tests;
