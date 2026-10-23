use super::super::catalog_check::{
    locks_interpolated_relation, locks_single_row, orders_by_catalog_key,
};
use super::super::directive::{contains_for_update, has_safe_directive};
use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{
    combine_sql_verdicts, sql_evidence_for_call, EmbeddedSqlCall, EmbeddedSqlFileFacts,
    EmbeddedSqlKind, LockingSelectMetadata, SchemaCatalog, SqlBranchJudgment, SqlEvidenceKind,
    SqlObligationVerdict, SqlStatementFileFacts, SqlVerdictStatus,
};
use std::borrow::Cow;

pub(super) fn record(
    file: &str,
    source: &str,
    embedded: &EmbeddedSqlFileFacts,
    opts: &CompiledOptions,
    catalog: Option<&SchemaCatalog>,
    statements: Option<&[SqlStatementFileFacts]>,
    facts: &dyn SqlFactContext,
) {
    let prepared = statements
        .map(crate::codebase::rules::index_sql_variants)
        .unwrap_or_default();
    let checks = if catalog.is_some() {
        &["row-lock-order", "catalog-key-order"][..]
    } else {
        &["row-lock-order"][..]
    };
    for (call_index, original) in embedded.calls.iter().enumerate() {
        let exempt = has_safe_directive(
            source,
            original.line,
            original.sql_text.as_deref().unwrap_or_default(),
            &opts.safe_directive,
        );
        let versions = original
            .statement_calls()
            .enumerate()
            .map(|(index, version)| {
                let sql = version.sql_text.as_deref().unwrap_or_default();
                let metadata = if version.kind == EmbeddedSqlKind::Dynamic
                    || has_safe_directive(source, version.line, sql, &opts.safe_directive)
                    || !contains_for_update(sql)
                {
                    Err("metadata-not-required")
                } else {
                    metadata(&version, prepared.get(&(call_index, index)).copied())
                };
                checks
                    .iter()
                    .map(|check| judge(check, source, &version, opts, catalog, &metadata))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        for (check_index, check) in checks.iter().enumerate() {
            let mut reasons = Vec::new();
            let statuses = versions
                .iter()
                .map(|version| {
                    let judgment = &version[check_index];
                    if let Some(reason) = judgment.reason {
                        reasons.push(reason.to_string());
                    }
                    judgment.status
                })
                .collect::<Vec<_>>();
            let exhaustive = exempt
                || !original.variants_truncated()
                    && (original.kind != EmbeddedSqlKind::Dynamic || !original.variants.is_empty());
            if !exhaustive {
                reasons.push("opaque-sql-tail".to_string());
            }
            reasons.sort();
            reasons.dedup();
            let status = if exempt {
                SqlVerdictStatus::ProvenClear
            } else {
                combine_sql_verdicts(&statuses, exhaustive, false)
            };
            facts.record_sql_verdict(SqlObligationVerdict {
                profile: opts.embedded.clone(),
                rule: RULE_ID.to_string(),
                application: None,
                subcheck: (*check).to_string(),
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
                residual_reasons: reasons,
            });
        }
    }
}

#[cfg(test)]
mod tests;

fn metadata<'a>(
    call: &'a EmbeddedSqlCall,
    prepared: Option<&'a SqlStatementFileFacts>,
) -> Result<Cow<'a, [LockingSelectMetadata]>, &'static str> {
    let Some(sql) = call.sql_text.as_deref() else {
        return Err("unresolved-sql-fragment");
    };
    if let Some(statement) = prepared {
        if statement.parse_failed {
            return Err("invalid-sql");
        }
        if let Some(locations) = &statement.variant_locations {
            return Ok(Cow::Borrowed(&locations.locking));
        }
    }
    let _ = sql;
    Err("missing-prepared-sql-facts")
}

fn judge(
    check: &str,
    source: &str,
    call: &EmbeddedSqlCall,
    opts: &CompiledOptions,
    catalog: Option<&SchemaCatalog>,
    metadata: &Result<Cow<'_, [LockingSelectMetadata]>, &'static str>,
) -> SqlBranchJudgment {
    let sql = call.sql_text.as_deref().unwrap_or_default();
    if has_safe_directive(source, call.line, sql, &opts.safe_directive) {
        return SqlBranchJudgment::clear();
    }
    if call.kind == EmbeddedSqlKind::Dynamic {
        return SqlBranchJudgment::abstain("opaque-sql-tail");
    }
    if !contains_for_update(sql) {
        return if call.sql_text.is_some() {
            SqlBranchJudgment::clear()
        } else {
            SqlBranchJudgment::abstain("unresolved-sql-fragment")
        };
    }
    let locks = match metadata {
        Ok(locks) => locks,
        Err(reason) => return SqlBranchJudgment::abstain(reason),
    };
    if locks.is_empty() {
        return SqlBranchJudgment::abstain("unresolved-locking-select");
    }
    let mut unknown = false;
    for lock in locks.iter().filter(|lock| {
        lock.has_multi_row_predicate
            && !lock.skips_locked_rows
            && !catalog.is_some_and(|catalog| locks_single_row(lock, catalog))
    }) {
        let known = catalog.is_some_and(|catalog| {
            lock.tables.as_deref().is_some_and(|tables| {
                !tables.is_empty() && tables.iter().all(|table| catalog.relation(table).is_some())
            })
        });
        if catalog.is_some() && !known {
            unknown = true;
            continue;
        }
        if check == "row-lock-order" {
            if !lock.has_order_by {
                return SqlBranchJudgment::violation(false);
            }
            continue;
        }
        let Some(catalog) = catalog else {
            return SqlBranchJudgment::abstain("missing-relation-catalog");
        };
        if locks_interpolated_relation(lock) {
            unknown = true;
        } else if !orders_by_catalog_key(lock, catalog) {
            return SqlBranchJudgment::violation(false);
        }
    }
    if unknown {
        SqlBranchJudgment::abstain("unknown-locked-relation")
    } else {
        SqlBranchJudgment::clear()
    }
}
