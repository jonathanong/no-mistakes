use crate::config::source_locations::ConfigPath;
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use locations::display_config_path;
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

pub(crate) fn finish(mut errors: Vec<String>) -> Result<()> {
    if errors.is_empty() {
        return Ok(());
    }
    errors.sort();
    Err(anyhow::anyhow!(errors.join("\n")))
}

mod applications;
pub(super) use applications::{
    collect_config_errors, indexed_rule_applications, push_application_errors,
};

mod compiled;
mod compiled_misc;
mod locations;
mod postgres;
mod postgres_compiled;
mod selectors;
mod standard;
mod workflow;

pub(crate) fn validate_rule_options(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    location: &dyn Fn(&ConfigPath) -> Option<(std::path::PathBuf, usize)>,
) -> Result<()> {
    let mut errors = Vec::new();
    for (index, rule) in config
        .rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| rule.enabled)
    {
        let app = ConfigPath::default().key("rules").index(index);
        let stamp = |option: &ConfigPath, message: String| {
            let mut key = app.clone();
            for segment in option.as_slice() {
                key = match segment {
                    crate::config::source_locations::Segment::Key(key_name) => key.key(key_name),
                    crate::config::source_locations::Segment::Index(index) => key.index(*index),
                };
            }
            let mut source = location(&key);
            while source.is_none() && key != app {
                key.pop();
                source = location(&key);
            }
            if let Some((path, line)) = source {
                let display = display_config_path(root, Some(&path))
                    .unwrap_or_else(|| path.display().to_string());
                format!("{display}:{line}: {message}")
            } else if let Some(prefix) = display_config_path(root, config_path) {
                format!("{prefix}:1: {message}")
            } else {
                message
            }
        };
        match config
            .requested_rule_analysis(rule)
            .and_then(|_| application_errors(root, config, rule))
        {
            Ok(messages) => errors.extend(
                messages
                    .into_iter()
                    .map(|message| stamp(&semantic_option_path(&message), message)),
            ),
            Err(error) => {
                let message = format!("{error:#}");
                let option = error
                    .downcast_ref::<crate::config::v2::schema::OptionLocation>()
                    .map(|error| error.path.clone())
                    .unwrap_or_else(|| semantic_option_path(&message));
                let message = if message.contains(&rule.rule) {
                    message
                } else {
                    format!("invalid options for rule `{}`: {message}", rule.rule)
                };
                errors.push(stamp(&option, message));
            }
        }
    }
    finish(errors)
}

fn application_errors(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
) -> Result<BTreeSet<String>> {
    match rule.rule.as_str() {
        super::GITHUB_ACTIONS_ACTION_INPUT_POLICY => {
            super::github_actions_action_input_policy::collect_application_errors(
                root, config, rule,
            )
        }
        super::GITHUB_ACTIONS_CACHE_KEY_POLICY => {
            super::github_actions_cache_key_policy::collect_application_errors(root, config, rule)
        }
        super::VITEST_DEFAULT_EXCLUDES => {
            super::vitest_default_excludes::collect_application_errors(root, config, rule)
        }
        _ => {
            if !postgres::postgres(rule)?
                && !postgres_compiled::postgres_compiled(rule)?
                && !compiled::compiled(rule)?
                && !compiled_misc::compiled_misc(rule, config, root)?
                && !selectors::validate(rule, root)?
                && !workflow::workflow(rule)?
            {
                standard::standard(rule)?;
            }
            crate::codebase::postgres::validate_embedded_sql_rule_options(rule)?;
            if rule.options.get("reportUnmatchedExecutorNames").is_some() {
                rule.try_rule_options::<super::postgres_unmatched_executors::Options>()?;
            }
            Ok(BTreeSet::new())
        }
    }
}

fn semantic_option_path(message: &str) -> ConfigPath {
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
        .next()
        .filter(|key| !key.is_empty())
        .map_or_else(
            || path.clone(),
            |key| key.split('.').fold(path.clone(), ConfigPath::key),
        )
}
