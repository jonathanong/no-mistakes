use super::{schema::RuleDef, NoMistakesConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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
        let unanalyzable = if let Some(shared) = option(rule, "unanalyzable")? {
            shared
        } else {
            legacy_policy(rule)?
                .or(self.unanalyzable)
                .or(self.unanalyzable_sql)
                .unwrap_or_default()
        };
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

    /// Propagate new shared defaults to legacy SQL parsers, without rewriting
    /// pre-existing alias payloads or inserting options for implicit defaults.
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
            if (shared.is_some() || (!has_alias && inherited.is_some()))
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

fn legacy_policy(rule: &RuleDef) -> Result<Option<Unanalyzable>> {
    if !sql_policy_rule(&rule.rule) {
        if legacy_sql_options(&rule.rule) {
            // Project scope historically copied this key to rules that ignored
            // it. Those opaque payloads must remain valid at maximum effort.
            return Ok(rule
                .options
                .get("unanalyzableSql")
                .and_then(|value| serde_yaml::from_value(value.clone()).ok()));
        }
        return option(rule, "unanalyzableSql");
    }
    let Some(value) = rule
        .options
        .get("unanalyzableSql")
        .and_then(serde_yaml::Value::as_str)
    else {
        return Ok(None);
    };
    let mode = if rule.rule == "postgres-require-query-annotation" {
        match value {
            "report" => Some(Unanalyzable::Report),
            "advise" => Some(Unanalyzable::Advise),
            "ignore" => Some(Unanalyzable::Ignore),
            _ => None,
        }
    } else {
        match value.trim().to_ascii_lowercase().as_str() {
            "fail" | "" => Some(Unanalyzable::Report),
            "advise" => Some(Unanalyzable::Advise),
            "ignore" => Some(Unanalyzable::Ignore),
            _ => None,
        }
    };
    Ok(mode)
}

pub(crate) fn validate_low_alias(rule: &RuleDef) -> Result<()> {
    if rule.options.get("unanalyzable").is_none()
        && rule.options.get("unanalyzableSql").is_some()
        && legacy_policy(rule)?.is_none()
    {
        anyhow::bail!("{}: invalid options.unanalyzableSql", rule.rule);
    }
    Ok(())
}

pub(super) fn legacy_sql_options(rule: &str) -> bool {
    rule.starts_with("postgres-")
}

pub(super) fn sql_policy_rule(rule: &str) -> bool {
    matches!(
        rule,
        "postgres-bounded-statements"
            | "postgres-conflict-ordering"
            | "postgres-explicit-columns"
            | "postgres-generated-column-predicates"
            | "postgres-idempotent-insert"
            | "postgres-lock-ordering"
            | "postgres-no-generated-column-writes"
            | "postgres-no-offset"
            | "postgres-require-query-annotation"
            | "postgres-required-predicates"
            | "postgres-sql-shape-policy"
            | "postgres-sql-statement-policy"
    )
}

#[cfg(test)]
mod tests;
