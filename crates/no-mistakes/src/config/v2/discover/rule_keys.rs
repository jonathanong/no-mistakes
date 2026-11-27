//! Reject unknown rule IDs and rule-entry keys before serde silently drops them.
use super::super::schema::RuleDef;
use crate::codebase::rules::rule_ids::configurable_rule_ids;
use crate::config::source_locations::{ConfigLocations, ConfigPath};
use anyhow::Result;
use no_mistakes_rule_kernel::rule_ids::closest_rule_id;
use serde_yaml::{Mapping, Value};
use std::cell::OnceCell;
use std::path::Path;

pub(super) fn validate(source: &str, path: &Path) -> Result<()> {
    // Syntax errors remain the responsibility of the ordinary config parser.
    let Ok(Value::Mapping(top)) = crate::config::parse_config::<Value>(source, path) else {
        return Ok(());
    };
    let validation = Validation::new(source, path);
    let origin = ConfigPath::default();
    validation.rules(top.get("rules"), &origin.clone().key("rules"), true)?;
    validation.rules(
        top.get("repositoryRules"),
        &origin.clone().key("repositoryRules"),
        false,
    )?;
    for group in ["projects", "overrides"] {
        if let Some(entries) = top.get(group) {
            validation.group_rules(entries, &origin.clone().key(group))?;
        }
    }
    Ok(())
}

pub(super) fn validate_fragment(source: &str, path: &Path, section: &str) -> Result<()> {
    if !matches!(section, "rules" | "projects") {
        return Ok(());
    }
    let Ok(value) = crate::config::parse_config::<Value>(source, path) else {
        return Ok(());
    };
    let validation = Validation::new(source, path);
    let origin = ConfigPath::default();
    if section == "rules" {
        validation.rules(Some(&value), &origin, true)
    } else {
        validation.group_rules(&value, &origin)
    }
}

struct Validation<'a> {
    source: &'a str,
    path: &'a Path,
    locations: OnceCell<Option<ConfigLocations>>,
}

impl<'a> Validation<'a> {
    fn new(source: &'a str, path: &'a Path) -> Self {
        Self {
            source,
            path,
            locations: OnceCell::new(),
        }
    }

    fn line(&self, origin: &ConfigPath) -> usize {
        self.locations
            .get_or_init(|| ConfigLocations::parse(self.source, self.path))
            .as_ref()
            .and_then(|locations| locations.line(origin))
            .unwrap_or(1)
    }

    fn rule_id(&self, id: &str, origin: &ConfigPath) -> Result<()> {
        if id.trim().is_empty() || configurable_rule_ids().any(|known| known == id) {
            return Ok(());
        }
        let suggestion = closest_rule_id(id, configurable_rule_ids())
            .map(|candidate| format!(" Did you mean `{candidate}`?"))
            .unwrap_or_default();
        anyhow::bail!(
            "{}:{}: unknown rule ID `{id}`.{suggestion}",
            crate::codebase::ts_resolver::normalize_path(self.path).display(),
            self.line(origin)
        );
    }

    fn rules(&self, value: Option<&Value>, origin: &ConfigPath, applications: bool) -> Result<()> {
        match value {
            Some(Value::Sequence(list)) => {
                for (index, value) in list.iter().enumerate() {
                    let origin = origin.clone().index(index);
                    match value {
                        Value::Mapping(entry) => {
                            if let Some(id) = entry.get("rule").and_then(Value::as_str) {
                                self.rule_id(id, &origin.clone().key("rule"))?;
                            }
                            if applications {
                                self.rule_entry(entry, &origin, index)?;
                            }
                        }
                        Value::String(id) if !applications => self.rule_id(id, &origin)?,
                        _ => {}
                    }
                }
            }
            Some(Value::Mapping(entries))
                if !applications || !entries.get("rule").is_some_and(Value::is_string) =>
            {
                for id in entries.keys().filter_map(Value::as_str) {
                    self.rule_id(id, &origin.clone().key(id))?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn group_rules(&self, value: &Value, origin: &ConfigPath) -> Result<()> {
        match value {
            Value::Mapping(groups) => {
                for (name, group) in groups {
                    if let Some(name) = name.as_str() {
                        self.rules(
                            group.get("rules"),
                            &origin.clone().key(name).key("rules"),
                            false,
                        )?;
                    }
                }
            }
            Value::Sequence(groups) => {
                for (index, group) in groups.iter().enumerate() {
                    self.rules(
                        group.get("rules"),
                        &origin.clone().index(index).key("rules"),
                        false,
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn rule_entry(&self, entry: &Mapping, origin: &ConfigPath, index: usize) -> Result<()> {
        self.validate_advisory_rule(entry, origin, index)?;
        for key in entry.keys() {
            let name = key
                .as_str()
                .map_or_else(|| format!("{key:?}"), str::to_string);
            if RuleDef::SUPPORTED_KEYS.contains(&name.as_str()) {
                continue;
            }
            let id = entry.get("rule").and_then(Value::as_str).unwrap_or("?");
            let location = self.line(&origin.clone().key(&name));
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
                self.path.display(),
                RuleDef::SUPPORTED_KEYS.join(", ")
            );
        }
        Ok(())
    }

    fn validate_advisory_rule(
        &self,
        entry: &Mapping,
        origin: &ConfigPath,
        index: usize,
    ) -> Result<()> {
        if entry.get("rule").and_then(Value::as_str) != Some("postgres-grouping-not-aggregation") {
            return Ok(());
        }
        if entry
            .get("severity")
            .is_some_and(|value| value.as_str() != Some("advisory"))
        {
            let line = self.line(&origin.clone().key("severity"));
            anyhow::bail!("{}:{line}: rules[{index}] postgres-grouping-not-aggregation is advisory only; omit severity or use severity: advisory", self.path.display());
        }
        if entry
            .get("options")
            .and_then(Value::as_mapping)
            .is_some_and(|options| options.contains_key("fail"))
        {
            let line = self.line(&origin.clone().key("options").key("fail"));
            anyhow::bail!("{}:{line}: rules[{index}] postgres-grouping-not-aggregation option fail is not supported; remove it because this rule cannot fail a run", self.path.display());
        }
        Ok(())
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
