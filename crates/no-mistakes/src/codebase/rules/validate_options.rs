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
mod no_options;
mod postgres;
mod postgres_compiled;
mod selectors;
mod standard;
mod workflow;

mod postgres_sql;

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NoOptions {}

pub(super) struct ValidationContext<'a> {
    root: &'a Path,
    config: &'a NoMistakesConfig,
}

type Validate = fn(&RuleDef, &ValidationContext<'_>) -> Result<()>;
type RuleValidator = (&'static str, Validate);

fn decode<T: for<'de> serde::Deserialize<'de> + Default>(
    rule: &RuleDef,
    _context: &ValidationContext<'_>,
) -> Result<()> {
    rule.strict_rule_options::<T>().map(drop)
}

const VALIDATORS: &[&[RuleValidator]] = &[
    postgres::VALIDATORS,
    postgres_compiled::VALIDATORS,
    postgres_sql::VALIDATORS,
    compiled::VALIDATORS,
    compiled_misc::VALIDATORS,
    selectors::VALIDATORS,
    workflow::VALIDATORS,
    standard::VALIDATORS,
    no_options::VALIDATORS,
];

pub(crate) fn validate_rule_options(
    root: &Path,
    config_path: &Path,
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
            let (path, line) = source.unwrap_or_else(|| (config_path.to_path_buf(), 1));
            let display = display_config_path(root, &path);
            format!("{display}:{line}: {message}")
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
            let validate = VALIDATORS
                .iter()
                .flat_map(|group| group.iter())
                .find_map(|(id, validate)| (*id == rule.rule).then_some(*validate))
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "internal error: rule `{}` has no registered option validator",
                        rule.rule
                    )
                })?;
            validate(rule, &ValidationContext { root, config })?;
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
        .take(1)
        .filter(|key| !key.is_empty())
        .flat_map(|key| key.split('.'))
        .fold(path, ConfigPath::key)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
