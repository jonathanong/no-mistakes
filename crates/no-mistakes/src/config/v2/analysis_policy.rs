use super::{schema::RuleDef, NoMistakesConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
mod legacy;
use legacy::legacy_policy;
pub(crate) use legacy::validate_low_alias;
pub(super) use legacy::{legacy_sql_options, sql_policy_rule};
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
    clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    Low,
    #[default]
    Max,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Unanalyzable {
    #[default]
    Report,
    Advise,
    Ignore,
}

/// Resolved before fact demand. Non-adopters retain maximum analysis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EffectiveRuleAnalysis {
    pub effort: Effort,
    pub unanalyzable: Unanalyzable,
}

impl NoMistakesConfig {
    pub fn requested_rule_analysis(&self, rule: &RuleDef) -> Result<EffectiveRuleAnalysis> {
        let effort = self
            .effort_override
            .or(option(rule, "effort")?)
            .or(self.effort)
            .unwrap_or_default();
        if rule.options.get("unanalyzable").is_some()
            && (!legacy_sql_options(&rule.rule) || sql_policy_rule(&rule.rule))
        {
            validate_low_alias(rule)?;
        }
        let mut unanalyzable = if let Some(shared) = option(rule, "unanalyzable")? {
            shared
        } else {
            legacy_policy(rule)?
                .or(self.unanalyzable)
                .or(self.unanalyzable_sql)
                .unwrap_or_default()
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
            if !self.rules[index].enabled {
                continue;
            }
            let rule = &self.rules[index];
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
                let value = match policy.unanalyzable {
                    Unanalyzable::Report if rule.rule != "postgres-require-query-annotation" => {
                        "fail"
                    }
                    Unanalyzable::Report => "report",
                    Unanalyzable::Advise => "advise",
                    Unanalyzable::Ignore => "ignore",
                };
                if self.rules[index].options.is_null() {
                    self.rules[index].options = serde_yaml::Value::Mapping(Default::default());
                }
                self.rules[index].options["unanalyzableSql"] = value.into();
            }
        }
        Ok(())
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
