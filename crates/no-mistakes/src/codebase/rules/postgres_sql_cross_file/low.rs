//! Conservative syntax tier shared by SQL identifier policy adopters.
use super::{compile, finding, Options};
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::postgres::{SqlObligationVerdict, SqlVerdictStatus};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

mod recovered;
use recovered::classify;

const REWRITE: &str = "pass a literal SQL string or template directly to the imported executor, or through one local const; keep relation names literal and pass external data as bind values";

pub(crate) fn check(
    root: &Path,
    config: &NoMistakesConfig,
    application: &RuleDef,
    index: usize,
    facts: &CheckFactMap,
    rule_id: &str,
) -> Result<Vec<RuleFinding>> {
    let options: Options = application.try_rule_options()?;
    let options = compile(rule_id, &options)?;
    let filter = super::super::path_filter::RulePathFilter::new(root, config, application)?;
    let policy = config.effective_rule_analysis(application)?;
    let context = facts.for_sql_application(index, application.name.as_deref());
    let local_index = config
        .rule_applications(rule_id)
        .iter()
        .position(|rule| std::ptr::eq(*rule, application))
        .unwrap_or(index);
    let application_key =
        crate::codebase::postgres::sql_identifier_syntax::helper_recovery::ApplicationKey::new(
            &options.embedded,
            rule_id,
            local_index,
        );
    let observer = crate::diagnostics::current();
    let batches = facts
        .files()
        .par_iter()
        .map(|path| -> Result<Vec<RuleFinding>> {
            let mut findings = Vec::new();
            let rel = relative_slash_path(root, path);
            if !options.includes(&rel) || !filter.is_match(path) {
                return Ok(findings);
            }
            let Some(file) = facts.ts.get(path) else {
                anyhow::bail!("{rule_id}: prepared facts are missing for {rel}");
            };
            let calls = file.sql_identifier_syntax.calls_for(&application_key);
            let calls = if let Some(calls) = calls {
                calls
            } else if file.parse_error.is_some() || file.ts.parse_error.is_some() {
                // Parser diagnostics are authoritative; a malformed unrelated file
                // supplies no proof that a configured SQL executor exists.
                return Ok(findings);
            } else {
                anyhow::bail!(
                    "{rule_id}: prepared low-effort SQL syntax projection is missing for {rel}"
                );
            };
            for call in calls {
                if let Some(observer) = &observer {
                    observer.increment("postgres.sql_identifier.low.calls", 1);
                }
                let (sources, unknown) =
                    classify(&call.input, rule_id, file, &application_key, call.span);
                for source in &sources {
                    findings.push(finding(
                        root,
                        rule_id,
                        path,
                        call.line,
                        super::super::postgres_no_unbounded_sql_identifier::diagnostic_message(
                            source,
                        ),
                        Some(source.clone()),
                        Some(source.clone()),
                    ));
                }
                if unknown && policy.unanalyzable == Unanalyzable::Report {
                    findings.push(finding(
                        root,
                        rule_id,
                        path,
                        call.line,
                        format!(
                            "SQL identifier provenance is unanalyzable at effort low; {REWRITE}"
                        ),
                        None,
                        None,
                    ));
                }
                if let Some(context) = &context {
                    context.record_sql_verdict(SqlObligationVerdict {
                        profile: options.embedded.clone(),
                        rule: rule_id.into(),
                        application: None,
                        subcheck: "identifier-provenance".into(),
                        file: rel.clone(),
                        span: call.span,
                        status: if !sources.is_empty() {
                            SqlVerdictStatus::Violation
                        } else if unknown {
                            SqlVerdictStatus::Abstain
                        } else {
                            SqlVerdictStatus::ProvenClear
                        },
                        advisory: unknown && policy.unanalyzable == Unanalyzable::Advise,
                        evidence: None,
                        known_prefix: None,
                        helper_lines: Vec::new(),
                        residual_reasons: if unknown {
                            vec![format!(
                                "SQL identifier provenance is unanalyzable at effort low; {REWRITE}"
                            )]
                        } else {
                            Vec::new()
                        },
                    });
                }
            }
            Ok(findings)
        })
        .collect::<Vec<_>>();
    let mut findings = Vec::new();
    for batch in batches {
        findings.extend(batch?);
    }
    super::super::sort_findings(&mut findings);
    Ok(findings)
}
#[cfg(test)]
mod tests;
