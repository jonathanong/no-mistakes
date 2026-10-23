mod diagnostics;
use crate::config::v2::{schema::RuleDef, AnalysisFallback, Effort, NoMistakesConfig};
use anyhow::Result;
pub use diagnostics::AnalysisDiagnostics;
use std::path::{Path, PathBuf};

/// A low-tier implementation declares every rule whose enforcement it assumes.
#[derive(Clone, Copy)]
pub struct RuleEffortSupport {
    pub rule: &'static str,
    pub partners: &'static [&'static str],
    /// Certifies semantic/profile/runner coverage before file coverage is tested.
    /// Unknown or unsupported selectors must return false.
    pub partner_scope: fn(&RuleDef, &RuleDef) -> bool,
}

// These positive syntax proofs need no companion rule: every other shape is
// explicitly unanalyzable under the caller's policy, rather than inferred clear.
const SUPPORT: &[RuleEffortSupport] = &[
    RuleEffortSupport {
        rule: "postgres-require-query-annotation",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "test-no-unmocked-dynamic-imports",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "playwright-unique-test-ids",
        partners: &[],
        partner_scope: |_, _| false,
    },
];

pub fn prepare_rule_analysis(
    root: &Path,
    config: &mut NoMistakesConfig,
    visible: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<()> {
    prepare_rule_analysis_with_support(root, config, visible, sources, SUPPORT)
}

/// The same preparation path permits a small fixture-backed test adopter.
#[doc(hidden)]
pub fn prepare_rule_analysis_with_support(
    root: &Path,
    config: &mut NoMistakesConfig,
    visible: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    support: &[RuleEffortSupport],
) -> Result<()> {
    config.materialize_analysis_defaults()?;
    let mut policies = Vec::new();
    let mut warnings = Vec::new();
    let mut fallbacks = Vec::new();
    for rule in &config.rules {
        if !rule.enabled {
            policies.push(None);
            continue;
        }
        let mut policy = config.requested_rule_analysis(rule)?;
        if policy.effort == Effort::Low {
            if let Some(adopter) = support.iter().find(|entry| entry.rule == rule.rule) {
                crate::config::v2::validate_low_alias(rule)?;
                let files = if adopter.partners.is_empty() {
                    Vec::new()
                } else {
                    application_files(root, config, rule, visible)?
                };
                for partner in adopter.partners {
                    let applications = config.rule_applications(partner);
                    let mut covered = crate::fx::fx_set();
                    for application in &applications {
                        if (adopter.partner_scope)(rule, application) {
                            covered.extend(application_files(root, config, application, visible)?);
                        }
                    }
                    let uncovered = files.iter().any(|file| {
                        !covered.contains(file)
                            || sources.read_path(file).map_or(true, |source| {
                                crate::codebase::ts_source::has_disable_file_comment(
                                    &source, partner,
                                ) || !crate::codebase::ts_source::disabled_statement_lines(
                                    &source, partner,
                                )
                                .is_empty()
                            })
                    });
                    if applications.is_empty() || uncovered {
                        warnings.push(format!(
                            "{}: effort low requires enabled partner `{partner}` covering this application's scope; falling back to max",
                            rule.name.as_deref().unwrap_or(&rule.rule)
                        ));
                        fallbacks.push(AnalysisFallback {
                            rule: rule.rule.clone(),
                            partner: (*partner).to_owned(),
                            requested: Effort::Low,
                            effective: Effort::Max,
                        });
                        policy.effort = Effort::Max;
                    }
                }
            } else {
                warnings.push(format!(
                    "{}: requested effort low; effective effort max because this rule has no low implementation",
                    rule.name.as_deref().unwrap_or(&rule.rule)
                ));
                policy.effort = Effort::Max;
            }
        }
        if policy.effort == Effort::Low
            && rule.rule == "postgres-require-query-annotation"
            && rule.options.get("trustedSqlTags").is_some()
        {
            warnings.push(format!(
                "{}: trustedSqlTags is deprecated and ignored at effort low; remove it: the literal leading /* name */ determines annotation for every tag",
                rule.name.as_deref().unwrap_or(&rule.rule)
            ));
        }
        policies.push(Some(policy));
    }
    for (rule, policy) in config.rules.iter_mut().zip(policies) {
        rule.resolved_analysis = policy;
    }
    config.analysis_warnings = warnings;
    config.analysis_fallbacks = fallbacks;
    Ok(())
}

fn application_files(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    visible: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let (mut files, roots) = super::selected_rule_files_with_roots(root, config, rule, visible)?;
    // Rule-local include/exclude narrows the application beyond project targeting.
    if let Some(include) = patterns(rule, "include")? {
        if !include.is_empty() {
            files = super::matching_files(root, &include, &files, &roots)?;
        }
    }
    if let Some(exclude) = patterns(rule, "exclude")? {
        let excluded: crate::fx::FxHashSet<_> =
            super::matching_files(root, &exclude, &files, &roots)?
                .into_iter()
                .collect();
        files.retain(|file| !excluded.contains(file));
    }
    Ok(files)
}

fn patterns(rule: &RuleDef, name: &str) -> Result<Option<Vec<String>>> {
    rule.options
        .get(name)
        .map(|value| {
            serde_yaml::from_value(value.clone())
                .map_err(|error| anyhow::anyhow!("{}: invalid options.{name}: {error}", rule.rule))
        })
        .transpose()
}

#[cfg(test)]
#[path = "analysis_policy/tag_warning_tests.rs"]
mod tag_warning_tests;
#[cfg(test)]
mod tests;
