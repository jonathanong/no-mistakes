use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

pub(crate) fn format_invalid_option(
    rule: &str,
    option: &str,
    value: impl std::fmt::Debug,
    allowed: impl std::fmt::Display,
) -> String {
    format!("Rule `{rule}` has invalid value {value:?} for `options.{option}`. Expected {allowed}.")
}

pub(crate) fn finish(mut errors: Vec<String>) -> Result<()> {
    if errors.is_empty() {
        return Ok(());
    }
    errors.sort();
    Err(anyhow::anyhow!(errors.join("\n")))
}

fn display_config_path(root: &Path, config_path: Option<&Path>) -> Option<String> {
    let path = config_path?;
    let normalized_root = normalize_path(root);
    let normalized_path = normalize_path(path);
    let rel = relative_slash_path(&normalized_root, &normalized_path);
    (!rel.is_empty()).then_some(rel)
}

#[doc(hidden)]
pub fn validate_rule_options(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<()> {
    let mut errors = Vec::new();
    super::github_actions_action_input_policy::collect_config_errors(root, config, &mut errors);
    super::github_actions_cache_key_policy::collect_config_errors(root, config, &mut errors);
    super::vitest_default_excludes::collect_config_errors(root, config, &mut errors);
    if let Some(rel) = display_config_path(root, config_path) {
        for message in &mut errors {
            *message = format!("{rel}: {message}");
        }
    }
    finish(errors)
}
