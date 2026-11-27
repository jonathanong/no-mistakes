use anyhow::Result;

mod option_location;
pub use option_location::{ConfigPath, OptionLocation, Segment};

pub fn format_invalid_option(
    rule: &str,
    option: &str,
    value: impl std::fmt::Debug,
    allowed: impl std::fmt::Display,
) -> String {
    format!("Rule `{rule}` has invalid value {value:?} for `options.{option}`. Expected {allowed}.")
}

pub fn finish(mut errors: Vec<String>) -> Result<()> {
    if errors.is_empty() {
        return Ok(());
    }
    errors.sort();
    Err(anyhow::anyhow!(errors.join("\n")))
}

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoOptions {}

pub struct ValidationContext<'a, Config> {
    pub root: &'a std::path::Path,
    pub config: &'a Config,
}

pub type RuleValidator<Rule, Config> = (
    &'static str,
    fn(&Rule, &ValidationContext<'_, Config>) -> Result<()>,
);

pub fn semantic_option_path(message: &str) -> ConfigPath {
    let path = ConfigPath::default().key("options");
    let Some(option) = message
        .split_once("options.")
        .or_else(|| message.split_once(" option "))
        .map(|(_, option)| option)
    else {
        return path;
    };
    option
        .split(|c: char| !(c.is_alphanumeric() || matches!(c, '_' | '.' | '-')))
        .take(1)
        .filter(|key| !key.is_empty())
        .flat_map(|key| key.split('.'))
        .fold(path, ConfigPath::key)
}
