use anyhow::Result;
use serde::{de::IntoDeserializer, Deserialize};

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct RuleApplicationConfig {
    #[serde(default)]
    pub rule: String,
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub repository: bool,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub options: serde_yaml::Value,
}

impl RuleApplicationConfig {
    /// Deserializes configured options, preserving the historical fallback-to-default
    /// behavior for library callers.
    pub fn rule_options<T: for<'de> Deserialize<'de> + Default>(&self) -> T {
        self.try_rule_options().unwrap_or_default()
    }

    /// Deserializes configured options with an actionable diagnostic.
    ///
    /// Check entrypoints use this fallible variant so a typo never silently
    /// disables a configured codebase rule. The original [`Self::rule_options`]
    /// API intentionally remains infallible for existing programmatic callers.
    pub fn try_rule_options<T: for<'de> Deserialize<'de> + Default>(&self) -> Result<T> {
        if matches!(&self.options, serde_yaml::Value::Null)
            || matches!(&self.options, serde_yaml::Value::Mapping(options) if options.is_empty())
        {
            return Ok(T::default());
        }
        let deserializer = self.options.clone().into_deserializer();
        serde_path_to_error::deserialize(deserializer).map_err(|error| {
            let path = error.path().to_string();
            let location = if path.is_empty() || path == "." {
                "options".to_string()
            } else {
                format!("options.{path}")
            };
            anyhow::anyhow!(
                "invalid options for rule `{}` at {location}: {}",
                self.rule,
                error.inner()
            )
        })
    }
}
