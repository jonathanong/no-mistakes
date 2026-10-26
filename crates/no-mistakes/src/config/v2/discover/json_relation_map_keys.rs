//! Validate literal keys before generic config values erase duplicate map entries.
use anyhow::Result;
use jsonc_parser::ast::Value;
use serde::de::{IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

const RULE_ID: &str = "postgres-json-relation-ids";

pub(super) fn validate(source: &str, path: &Path) -> Result<()> {
    let duplicate = match path.extension().and_then(|extension| extension.to_str()) {
        Some("yaml" | "yml") => yaml_duplicate(source),
        Some("json" | "jsonc") => json_duplicate(
            source,
            path.extension()
                .is_some_and(|extension| extension == "jsonc"),
        ),
        _ => None,
    };
    if let Some(key) = duplicate {
        anyhow::bail!("{RULE_ID} option keyColumnMap: duplicate key {key}");
    }
    Ok(())
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Config {
    rules: Vec<Rule>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Rule {
    rule: String,
    options: Option<OptionsInput>,
}

// Buffering an untagged mapping retains repeated entries; arbitrary options on
// unrelated rules must not prevent inspecting a later configured application.
#[derive(Deserialize)]
#[serde(untagged)]
enum OptionsInput {
    Mapping(Options),
    Other(IgnoredAny),
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    key_column_map: Keys,
}

#[derive(Default)]
struct Keys(Option<String>);

impl<'de> Deserialize<'de> for Keys {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct KeyVisitor;
        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = Keys;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a key-to-column mapping")
            }

            fn visit_map<M: MapAccess<'de>>(
                self,
                mut map: M,
            ) -> std::result::Result<Keys, M::Error> {
                let mut keys = BTreeSet::new();
                let mut duplicate = None;
                while let Some((key, _)) = map.next_entry::<String, IgnoredAny>()? {
                    if !keys.insert(key.clone()) && duplicate.is_none() {
                        duplicate = Some(key);
                    }
                }
                Ok(Keys(duplicate))
            }
        }
        deserializer.deserialize_map(KeyVisitor)
    }
}

fn yaml_duplicate(source: &str) -> Option<String> {
    // Syntax/type errors remain the responsibility of the ordinary config parser.
    serde_yaml::from_str::<Config>(source)
        .ok()?
        .rules
        .into_iter()
        .filter(|rule| rule.rule == RULE_ID)
        .find_map(|rule| match rule.options? {
            OptionsInput::Mapping(options) => options.key_column_map.0,
            OptionsInput::Other(_) => None,
        })
}

fn property<'a, 'source>(value: &'a Value<'source>, key: &str) -> Option<&'a Value<'source>> {
    value
        .as_object()?
        .properties
        .iter()
        .rev()
        .find(|property| property.name.as_str() == key)
        .map(|property| &property.value)
}

fn json_duplicate(source: &str, comments: bool) -> Option<String> {
    let mut options = crate::config::jsonc_parse_options();
    options.allow_comments = comments;
    options.allow_trailing_commas = comments;
    let value = jsonc_parser::parse_to_ast(source, &Default::default(), &options)
        .ok()?
        .value?;
    let rules = property(&value, "rules")?.as_array()?;
    for rule in &rules.elements {
        let selected = property(rule, "rule")
            .and_then(Value::as_string_lit)
            .is_some_and(|rule| rule.value == RULE_ID);
        if !selected {
            continue;
        }
        let Some(mapping) = property(rule, "options")
            .and_then(|options| property(options, "keyColumnMap"))
            .and_then(Value::as_object)
        else {
            continue;
        };
        let mut keys = BTreeSet::new();
        for entry in &mapping.properties {
            let key = entry.name.as_str();
            if !keys.insert(key) {
                return Some(key.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests;
