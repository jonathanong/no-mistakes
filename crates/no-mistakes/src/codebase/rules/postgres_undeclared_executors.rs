//! Missing executor configuration is an abstention, never proof of execution.
use super::{RuleFinding, RuleGlobFilter};
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::postgres::{EmbeddedSqlOptions, SqlObligationVerdict, SqlVerdictStatus};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    include: Vec<String>,
    exclude: Vec<String>,
    import_specifier: Option<String>,
    executor_names: Vec<String>,
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
}

/// Borrow the existing request fact map, with each application's normal scope.
pub(crate) fn check(
    root: &Path,
    config: &NoMistakesConfig,
    rule_id: &str,
    files: &[PathBuf],
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for (index, rule) in config.rule_applications(rule_id).into_iter().enumerate() {
        let policy = config.effective_rule_analysis(rule)?.unanalyzable;
        if policy == Unanalyzable::Ignore && !facts.wants_sql_verdicts() {
            continue;
        }
        let opts: Options = rule.try_rule_options()?;
        let Some(specifier) = opts.import_specifier.filter(|value| !value.is_empty()) else {
            // An unscoped/disabled profile cannot prove a configured-module type.
            continue;
        };
        let paths = RuleGlobFilter::new(rule_id, &opts.include, &opts.exclude)?;
        let profile = EmbeddedSqlOptions::configured(&specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags);
        let view = facts.for_sql_application(index, rule.name.as_deref());
        let context = view
            .as_ref()
            .map_or(facts as &dyn SqlFactContext, |view| view);
        for path in super::selected_rule_files(root, config, rule, files)? {
            let rel = relative_slash_path(root, &path);
            if !crate::codebase::dependencies::extract::is_indexable(&path) || !paths.includes(&rel)
            {
                continue;
            }
            // The owning rule has already applied its read/parse-error policy.
            // An unavailable file supplies no imported receiver identity.
            let Ok(file) = facts.embedded_sql(&path, &profile) else {
                continue;
            };
            for call in &file.undeclared_executor_types {
                let (subcheck, target, message) = if call.configured_type {
                    (
                        "unresolved-configured-executor-receiver",
                        "unresolved-configured-executor-receiver",
                        format!(
                            "configured executor receiver of type `{}` from `{}` cannot be resolved; use an explicitly typed parameter/helper (supported shape) so its SQL is checked",
                            call.type_name, call.module,
                        ),
                    )
                } else {
                    (
                        "undeclared-executor-type",
                        "undeclared-executor-type",
                        format!(
                            "undeclared executor type `{}` from `{}`; add `{}` to executorTypeNames and use an explicitly typed parameter/helper so its SQL is checked",
                            call.type_name, call.module, call.type_name,
                        ),
                    )
                };
                if context.wants_sql_verdicts() {
                    context.record_sql_verdict(SqlObligationVerdict {
                        profile: profile.clone(),
                        rule: rule_id.to_string(),
                        application: None,
                        subcheck: subcheck.into(),
                        file: rel.clone(),
                        span: call.span,
                        status: SqlVerdictStatus::Abstain,
                        advisory: policy == Unanalyzable::Advise,
                        evidence: None,
                        known_prefix: None,
                        helper_lines: Vec::new(),
                        residual_reasons: vec![message.clone()],
                    });
                }
                if policy == Unanalyzable::Report {
                    findings.push(RuleFinding {
                        rule: rule_id.into(),
                        file: rel.clone(),
                        line: call.line as usize,
                        message: format!("{rel}:{}: {message}", call.line),
                        import: Some(call.module.clone()),
                        target: Some(target.into()),
                    });
                }
            }
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[cfg(test)]
mod tests;
