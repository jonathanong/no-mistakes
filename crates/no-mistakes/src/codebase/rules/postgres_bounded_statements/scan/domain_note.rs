//! Audit an accepted unique-key domain product without failing the check.
use super::super::evaluate::DomainAccept;
use super::super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{SqlObligationVerdict, SqlVerdictStatus};

pub(super) fn record(
    facts: &dyn SqlFactContext,
    opts: &CompiledOptions,
    file: &str,
    source: Option<&str>,
    accepts: &[DomainAccept],
) {
    for accept in accepts {
        let line = accept.line.max(1);
        let start = source
            .unwrap_or_default()
            .split_inclusive('\n')
            .take(line.saturating_sub(1))
            .map(str::len)
            .sum::<usize>() as u32;
        facts.record_sql_advisory(SqlObligationVerdict {
            profile: opts.embedded.clone(),
            rule: RULE_ID.into(),
            application: None,
            subcheck: "domain-product".into(),
            file: file.into(),
            span: (start, start),
            status: SqlVerdictStatus::Violation,
            advisory: true,
            evidence: None,
            known_prefix: None,
            helper_lines: Vec::new(),
            residual_reasons: vec![format!(
                "accepted {} by unique-key domain product {}, within maxDomainRows {}",
                accept.table, accept.product, accept.cap
            )],
        });
    }
}
