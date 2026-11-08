use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{SqlObligationVerdict, SqlVerdictStatus};
use serde::Deserialize;

#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Policy {
    #[default]
    Ignore,
    #[serde(alias = "require-bound")]
    Advise,
    Report,
}

pub(in crate::codebase) fn body_bounds_requested(options: &serde_yaml::Value) -> bool {
    policy(options).is_some_and(|policy| policy != Policy::Ignore)
}

pub(in crate::codebase) fn body_advice_requested(options: &serde_yaml::Value) -> bool {
    policy(options) == Some(Policy::Advise)
}

fn policy(options: &serde_yaml::Value) -> Option<Policy> {
    options
        .get("materializedBodies")
        .and_then(|value| serde_yaml::from_value(value.clone()).ok())
}

pub(super) fn message(body: &str, table: &str) -> String {
    format!("{body} can build an unbounded candidate pool from {table}; add a LIMIT inside the body or restrict each relation by a primary/unique key or a reviewed columnBounds key; an outer cap does not bound the body")
}

pub(super) fn advise(
    facts: &dyn SqlFactContext,
    opts: &CompiledOptions,
    file: &str,
    source: Option<&str>,
    line: usize,
    message: &str,
) {
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
        subcheck: "materialized-body".into(),
        file: file.into(),
        span: (start, start),
        status: SqlVerdictStatus::Violation,
        advisory: true,
        evidence: None,
        known_prefix: None,
        helper_lines: Vec::new(),
        residual_reasons: vec![message.into()],
    });
}

pub(super) fn finish(
    facts: &dyn SqlFactContext,
    opts: &CompiledOptions,
    root: &std::path::Path,
    sources: &crate::codebase::ts_source::SourceStore,
    mut findings: crate::codebase::rules::PostgresFindings,
    advice: crate::codebase::rules::PostgresFindings,
) -> crate::codebase::rules::PostgresFindings {
    let advice_sites = (!advice.0.is_empty()).then(|| {
        advice
            .0
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
    });
    findings.extend(advice);
    findings.sort();
    let mut findings = opts
        .allow
        .clone()
        .apply_located(&opts.schema_catalog_path, findings);
    if let Some(advice_sites) = advice_sites {
        findings.0.retain(|located| {
            if !advice_sites.contains(located) {
                return true;
            }
            let finding = &located.finding;
            let source = crate::codebase::rules::read_source(sources, &root.join(&finding.file));
            let prefix = format!("{}:{}: ", finding.file, finding.line);
            advise(
                facts,
                opts,
                &finding.file,
                source.as_deref(),
                finding.line,
                finding
                    .message
                    .strip_prefix(&prefix)
                    .unwrap_or(&finding.message),
            );
            false
        });
    }
    findings.sort();
    findings
}
