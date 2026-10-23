//! Composition diagnostics do not turn invalid SQL into annotation failures.
use super::RULE_ID;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{
    EmbeddedSqlFileFacts, EmbeddedSqlOptions, SqlObligationVerdict, SqlStatementFileFacts,
    SqlVerdictStatus,
};
use crate::fx::{FxHashMap, FxHashSet};

pub(super) fn diagnostics(
    file: &str,
    embedded: &EmbeddedSqlFileFacts,
    profile: &EmbeddedSqlOptions,
    statements: &FxHashMap<(usize, usize), &SqlStatementFileFacts>,
    facts: &dyn SqlFactContext,
    observer: Option<&crate::diagnostics::InvocationObserver>,
) -> FxHashSet<(usize, usize)> {
    let mut invalid = FxHashSet::default();
    for (call_index, (call, span)) in embedded.calls.iter().zip(&embedded.call_spans).enumerate() {
        let mut skipped = 0;
        let append_composed = call
            .variants
            .iter()
            .any(|variant| !variant.append_sites.is_empty());
        for variant_index in 0..call.variants.len() {
            if append_composed
                && statements
                    .get(&(call_index, variant_index))
                    .is_some_and(|statement| statement.parse_failed)
            {
                invalid.insert((call_index, variant_index));
                skipped += 1;
            }
        }
        if skipped == 0 && !call.variants_truncated() {
            continue;
        }
        if let Some(observer) = observer {
            observer.increment("postgres.sql.variants_skipped_invalid", skipped);
        }
        let mut reasons = Vec::new();
        if skipped != 0 {
            reasons.push(format!("{skipped} invalid composed SQL variant(s) skipped; validate generated SQL in consumer tests"));
        }
        if call.variants_truncated() {
            reasons.push("SQL append variants truncated at the per-site exploration bound (cap or loop iterations); explored variants were still checked".to_string());
        }
        facts.record_sql_advisory(SqlObligationVerdict {
            profile: profile.clone(),
            rule: RULE_ID.to_string(),
            application: None,
            subcheck: "sql-composition".to_string(),
            file: file.to_string(),
            span: *span,
            status: SqlVerdictStatus::Abstain,
            advisory: true,
            evidence: None,
            known_prefix: None,
            helper_lines: Vec::new(),
            residual_reasons: reasons,
        });
    }
    invalid
}
