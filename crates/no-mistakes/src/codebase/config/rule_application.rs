use serde::Deserialize;

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
    pub fn rule_options<T: for<'de> Deserialize<'de> + Default>(&self) -> T {
        serde_yaml::from_value(self.options.clone()).unwrap_or_default()
    }
}
