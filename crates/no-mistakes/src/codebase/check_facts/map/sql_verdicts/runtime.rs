use super::{CheckFactMap, SqlApplicationFacts, SqlObligationVerdict};
use crate::codebase::postgres::{EmbeddedSqlFileFacts, EmbeddedSqlOptions, SqlStatementFileFacts};
use std::path::Path;

/// Borrow request-owned facts with runtime SQL selection local to an application.
#[doc(hidden)]
pub trait SqlFactContext: Sync {
    fn prepared(&self) -> &CheckFactMap;
    fn record_sql_verdict(&self, verdict: SqlObligationVerdict);
    #[doc(hidden)]
    fn is_low_sql(&self) -> bool {
        self.low_sql_application().is_some()
    }
    #[doc(hidden)]
    fn low_sql_application(&self) -> Option<(&str, usize)> {
        None
    }
    #[doc(hidden)]
    fn embedded_sql(
        &self,
        path: &Path,
        profile: &EmbeddedSqlOptions,
    ) -> anyhow::Result<&EmbeddedSqlFileFacts> {
        if let Some((rule, index)) = self.low_sql_application() {
            return crate::codebase::postgres::sql_identifier_syntax::statement_policy::projection(
                self.prepared(),
                path,
                profile,
                rule,
                index,
            )
            .map(|value| &value.embedded);
        }
        self.prepared().embedded_sql(path, profile)
    }
    #[doc(hidden)]
    fn postgres_statements(
        &self,
        path: &Path,
        profile: Option<&EmbeddedSqlOptions>,
    ) -> anyhow::Result<&[SqlStatementFileFacts]> {
        if let (Some(profile), Some((rule, index))) = (profile, self.low_sql_application()) {
            return crate::codebase::postgres::sql_identifier_syntax::statement_policy::projection(
                self.prepared(),
                path,
                profile,
                rule,
                index,
            )
            .map(|value| value.statements.as_slice());
        }
        self.prepared().postgres_statements(path, profile)
    }
    #[doc(hidden)]
    fn record_sql_advisory(&self, advisory: SqlObligationVerdict) {
        self.prepared().record_sql_advisory(advisory);
    }
    /// Explain the low tier's static rewrite while preserving maximum-effort messages.
    fn sql_unanalyzable_message(&self, message: &str) -> String {
        if self.is_low_sql() {
            format!("{message}; {LOW_SQL_REWRITE}")
        } else {
            message.to_string()
        }
    }
    fn wants_sql_verdicts(&self) -> bool {
        self.prepared().wants_sql_verdicts()
    }
}

impl SqlApplicationFacts<'_> {
    /// Select the bounded prepared SQL projection without copying request facts.
    pub fn with_low_sql(mut self, rule: &str) -> Self {
        self.low_rule = Some(rule.to_string());
        self
    }
}

pub(super) const LOW_SQL_REWRITE: &str = "SQL is unanalyzable at effort low; pass valid SQL as a literal string or interpolation-free template directly to the configured executor or through one local const; remove JavaScript escapes or use effort max for inferred SQL";
pub(super) fn explain_low(verdict: &mut SqlObligationVerdict, low: bool) {
    if low && !verdict.residual_reasons.is_empty() {
        verdict.residual_reasons.push(LOW_SQL_REWRITE.to_string());
        verdict.residual_reasons.sort();
        verdict.residual_reasons.dedup();
    }
}
