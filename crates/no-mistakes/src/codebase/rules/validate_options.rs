use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn format_invalid_option(
    rule: &str,
    option: &str,
    value: impl std::fmt::Debug,
    allowed: impl std::fmt::Display,
) -> String {
    format!("Rule `{rule}` has invalid value {value:?} for `options.{option}`. Expected {allowed}.")
}

pub(crate) fn finish(errors: BTreeSet<String>) -> Result<()> {
    if errors.is_empty() {
        return Ok(());
    }
    Err(anyhow::anyhow!(errors
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n")))
}

#[doc(hidden)]
pub fn validate_rule_options(root: &Path, config: &NoMistakesConfig) -> Result<()> {
    let mut errors = BTreeSet::new();
    super::github_actions_action_input_policy::collect_config_errors(root, config, &mut errors);
    super::github_actions_cache_key_policy::collect_config_errors(root, config, &mut errors);
    super::vitest_default_excludes::collect_config_errors(root, config, &mut errors);
    finish(errors)
}
