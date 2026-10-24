use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{SqlObligationVerdict, SqlVerdictStatus};

pub(super) struct RawSqlAdvice {
    starts: Vec<usize>,
    length: usize,
}
impl RawSqlAdvice {
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
        opts: &CompiledOptions,
        file: &str,
        line: usize,
        reason: &str,
    ) {
        let index = line.saturating_sub(1).min(self.starts.len() - 1);
        let start = self.starts[index];
        let end = self.starts.get(index + 1).copied().unwrap_or(self.length);
        facts.record_sql_advisory(SqlObligationVerdict {
            profile: opts.embedded.clone(),
            rule: RULE_ID.into(),
            application: None,
            subcheck: "uuid-predicates".into(),
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
