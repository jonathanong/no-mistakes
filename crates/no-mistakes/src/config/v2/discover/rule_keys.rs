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
    for (index, rule) in rules.iter().enumerate() {
        let Value::Mapping(entry) = rule else {
            continue;
        };
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
                    locations.line(&ConfigPath::default().key("rules").index(index).key(&name))
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

#[cfg(test)]
mod tests;
