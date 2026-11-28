use super::{schema::RuleDef, NoMistakesConfig};
use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
mod legacy;
use legacy::legacy_policy;
pub(super) use legacy::validate_inventory_policy;
pub(crate) use legacy::validate_low_alias;
pub(super) use legacy::{legacy_sql_options, project_sql_policy, sql_policy_rule};
mod fallback;
pub use fallback::AnalysisFallback;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deserialize,
    Serialize,
    ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    Low,
    #[default]
    Max,
}

pub use no_mistakes_rule_kernel::file_diagnostics::Unanalyzable;

/// Resolved before fact demand. Non-adopters retain maximum analysis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EffectiveRuleAnalysis {
    pub effort: Effort,
    pub unanalyzable: Unanalyzable,
}

impl RuleDef {
    pub(super) fn configured_effort(&self) -> Result<Option<Effort>> {
        self.options
            .get("effort")
            .filter(|value| !value.is_null() && value.as_str() != Some(""))
            .map(|value| {
                Effort::deserialize(value).with_context(|| {
                    let allowed = Effort::value_variants()
                        .iter()
                        .filter_map(Effort::to_possible_value)
                        .map(|value| value.get_name().to_owned())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "{}: invalid options.effort (expected one of: {allowed})",
                        self.rule
                    )
                })
            })
            .transpose()
    }
}

impl NoMistakesConfig {
    pub fn requested_rule_analysis(&self, rule: &RuleDef) -> Result<EffectiveRuleAnalysis> {
        validate_inventory_policy(rule)?;
        let effort = self
            .effort_override
            .or(rule.configured_effort()?)
            .or(self.effort)
            .unwrap_or_default();
        if rule.options.get("unanalyzable").is_some()
            && (!legacy_sql_options(&rule.rule) || sql_policy_rule(&rule.rule))
        {
            validate_low_alias(rule)?;
        }
        let mut unanalyzable = if rule.rule == "postgres-lock-inventory" {
            Unanalyzable::Report
        } else if let Some(shared) = option(rule, "unanalyzable")? {
            shared
        } else if matches!(
            rule.rule.as_str(),
            "github-actions-env-policy" | "github-actions-no-insta-snapshot-updates"
        ) {
            self.unanalyzable.unwrap_or(Unanalyzable::Advise)
        } else {
            legacy_policy(rule)?
                .or(self.unanalyzable)
                .or(self.unanalyzable_sql)
                .unwrap_or_else(|| default_unanalyzable(&rule.rule))
        };
        if rule.rule == "postgres-grouping-not-aggregation" && unanalyzable == Unanalyzable::Report
        {
            unanalyzable = Unanalyzable::Advise;
        }
        Ok(EffectiveRuleAnalysis {
            effort,
            unanalyzable,
        })
    }

    /// Read the post-validation policy used by both planning and rule execution.
    pub fn effective_rule_analysis(&self, rule: &RuleDef) -> Result<EffectiveRuleAnalysis> {
        if let Some(resolved) = rule.resolved_analysis {
            return Ok(resolved);
        }
        let mut policy = self.requested_rule_analysis(rule)?;
        policy.effort = Effort::Max;
        Ok(policy)
    }

    /// Propagate shared defaults and normalize advisory-only SQL policy aliases
    /// without inserting options for implicit defaults.
    pub fn materialize_analysis_defaults(&mut self) -> Result<()> {
        for index in 0..self.rules.len() {
            let rule = &self.rules[index];
            if !rule.enabled {
                rule.configured_effort()?;
                continue;
            }
            // Preserve the rule parser's original invalid-type diagnostic.
            if !rule.options.is_null() && !rule.options.is_mapping() {
                continue;
            }
            let policy = self.requested_rule_analysis(rule)?;
            let shared: Option<Unanalyzable> = option(rule, "unanalyzable")?;
            let has_alias = rule.options.get("unanalyzableSql").is_some();
            let inherited = self.unanalyzable.or(self.unanalyzable_sql);
            let grouping_advisory_alias = rule.rule == "postgres-grouping-not-aggregation"
                && rule
                    .options
                    .get("unanalyzableSql")
                    .and_then(serde_yaml::Value::as_str)
                    .is_some_and(|value| {
                        ["fail", "report"]
                            .iter()
                            .any(|mode| value.trim().eq_ignore_ascii_case(mode))
                    });
            if (shared.is_some() || (!has_alias && inherited.is_some()) || grouping_advisory_alias)
                && sql_policy_rule(&rule.rule)
            {
                let value = canonical_sql_policy(&rule.rule, policy.unanalyzable);
                if self.rules[index].options.is_null() {
                    self.rules[index].options = serde_yaml::Value::Mapping(Default::default());
                }
                self.rules[index].options["unanalyzableSql"] = value.into();
            }
        }
        Ok(())
    }
}

pub(super) fn canonical_sql_policy(rule: &str, policy: Unanalyzable) -> &'static str {
    match policy {
        Unanalyzable::Report if rule == "postgres-grouping-not-aggregation" => "advise",
        Unanalyzable::Report if rule == "postgres-require-query-annotation" => "report",
        Unanalyzable::Report => "fail",
        Unanalyzable::Advise => "advise",
        Unanalyzable::Ignore => "ignore",
    }
}

fn default_unanalyzable(rule: &str) -> Unanalyzable {
    if rule == crate::codebase::rules::NO_CROSS_PACKAGE_RELATIVE_IMPORTS {
        Unanalyzable::Advise
    } else {
        Unanalyzable::Report
    }
}

fn option<T: serde::de::DeserializeOwned>(rule: &RuleDef, key: &str) -> Result<Option<T>> {
    rule.options
        .get(key)
        .map(|value| {
            serde_yaml::from_value(value.clone())
                .with_context(|| format!("{}: invalid options.{key}", rule.rule))
        })
        .transpose()
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
