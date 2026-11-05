use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{
    EmbeddedSqlOptions, SqlObligationVerdict, SqlStatementFileFacts, SqlVerdictStatus,
};

pub(super) fn record_parse_failures(
    facts: &dyn SqlFactContext,
    rule: &str,
    profile: &EmbeddedSqlOptions,
    file: &str,
    source: &str,
    statements: &SqlStatementFileFacts,
) {
    if !statements.parse_failed
        || statements.path.extension().and_then(|ext| ext.to_str()) != Some("sql")
    {
        return;
    }
    let advice = NativeSqlAdvice::new(source);
    for (_, line) in statements.parse_failure_sites() {
        advice.record(
            facts,
            rule,
            profile,
            file,
            line,
            "SQL could not be analyzed",
        );
    }
}

pub(super) struct NativeSqlAdvice {
    starts: Vec<usize>,
    length: usize,
}

impl NativeSqlAdvice {
    pub(super) fn new(source: &str) -> Self {
        Self {
            starts: std::iter::once(0)
                .chain(source.match_indices('\n').map(|(offset, _)| offset + 1))
                .collect(),
            length: source.len(),
        }
    }

    pub(super) fn record(
        &self,
        facts: &dyn SqlFactContext,
        rule: &str,
        profile: &EmbeddedSqlOptions,
        file: &str,
        line: usize,
        reason: &str,
    ) {
        let index = line.saturating_sub(1).min(self.starts.len() - 1);
        let start = self.starts[index];
        let end = self.starts.get(index + 1).copied().unwrap_or(self.length);
        facts.record_sql_advisory(SqlObligationVerdict {
            profile: profile.clone(),
            rule: rule.into(),
            application: None,
            subcheck: "native-sql-parse".into(),
            file: file.into(),
            span: (start as u32, end as u32),
            status: SqlVerdictStatus::Abstain,
            advisory: true,
            evidence: None,
            known_prefix: None,
            helper_lines: Vec::new(),
            residual_reasons: vec![reason.into()],
        });
    }
}
