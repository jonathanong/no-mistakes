//! Reject unknown rule-entry keys before serde silently drops them.
use super::super::schema::RuleDef;
use crate::config::source_locations::{ConfigLocations, ConfigPath};
use anyhow::Result;
use serde_yaml::Value;
use std::path::Path;

pub(super) fn validate(source: &str, path: &Path) -> Result<()> {
    // Syntax errors remain the responsibility of the ordinary config parser.
    let Ok(Value::Mapping(top)) = crate::config::parse_config::<Value>(source, path) else {
        return Ok(());
    };
    let Some(Value::Sequence(rules)) = top.get("rules") else {
        return Ok(());
    };
    validate_rules(source, path, rules, false)
}

pub(super) fn validate_fragment(source: &str, path: &Path, section: &str) -> Result<()> {
    if section != "rules" {
        return Ok(());
    }
    let Ok(Value::Sequence(rules)) = crate::config::parse_config::<Value>(source, path) else {
        return Ok(());
    };
    validate_rules(source, path, &rules, true)
}

fn validate_rules(source: &str, path: &Path, rules: &[Value], fragment: bool) -> Result<()> {
    for (index, rule) in rules.iter().enumerate() {
        let Value::Mapping(entry) = rule else {
            continue;
        };
        validate_advisory_rule(entry, path, index)?;
        for key in entry.keys() {
            let name = key
                .as_str()
                .map_or_else(|| format!("{key:?}"), str::to_string);
            if RuleDef::SUPPORTED_KEYS.contains(&name.as_str()) {
                continue;
            }
            let id = entry.get("rule").and_then(Value::as_str).unwrap_or("?");
            let location = ConfigLocations::parse(source, path)
                .and_then(|locations| {
                    let origin = if fragment {
                        ConfigPath::default()
                    } else {
                        ConfigPath::default().key("rules")
                    };
                    locations.line(&origin.index(index).key(&name))
                })
                .unwrap_or(1);
            let hint = if name == "<<" {
                " YAML merge keys (`<<`) are not supported on rule entries; \
                 write the shared keys out on each entry instead."
            } else {
                ""
            };
            anyhow::bail!(
                "{}:{location}: rules[{index}] (rule `{id}`) has unknown key `{name}`. \
                 Supported keys: {}. An unknown key would be silently ignored, \
                 so fix the spelling or remove it.{hint}",
                path.display(),
                RuleDef::SUPPORTED_KEYS.join(", ")
            );
        }
    }
    Ok(())
}

fn validate_advisory_rule(entry: &serde_yaml::Mapping, path: &Path, index: usize) -> Result<()> {
    if entry.get("rule").and_then(Value::as_str) != Some("postgres-grouping-not-aggregation") {
        return Ok(());
    }
    if entry
        .get("severity")
        .is_some_and(|value| value.as_str() != Some("advisory"))
    {
        anyhow::bail!("{}: rules[{index}] postgres-grouping-not-aggregation is advisory only; omit severity or use severity: advisory", path.display());
    }
    if entry
        .get("options")
        .and_then(Value::as_mapping)
        .is_some_and(|options| options.contains_key("fail"))
    {
        anyhow::bail!("{}: rules[{index}] postgres-grouping-not-aggregation option fail is not supported; remove it because this rule cannot fail a run", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
