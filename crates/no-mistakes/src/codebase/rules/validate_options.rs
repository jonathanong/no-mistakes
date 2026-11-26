use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::source_locations::ConfigPath;
use crate::config::v2::schema::RuleDef;
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

pub(crate) fn finish(mut errors: Vec<String>) -> Result<()> {
    if errors.is_empty() {
        return Ok(());
    }
    errors.sort();
    Err(anyhow::anyhow!(errors.join("\n")))
}

pub(crate) fn indexed_rule_applications<'a>(
    config: &'a NoMistakesConfig,
    rule_id: &'a str,
) -> impl Iterator<Item = (usize, &'a RuleDef)> {
    config.rules.iter().enumerate().filter(move |(_, rule)| {
        rule.enabled && rule.rule == rule_id && config.rule_has_effective_target(rule)
    })
}

pub(crate) fn push_application_errors(
    root: &Path,
    config: &NoMistakesConfig,
    rule_index: usize,
    fallback: Option<&Path>,
    app_errors: BTreeSet<String>,
    out: &mut Vec<String>,
) {
    let key = ConfigPath::default().key("rules").index(rule_index);
    let source = config
        .config_provenance
        .origin(&key)
        .map(|(path, _)| path)
        .or(fallback);
    let prefix = display_config_path(root, source);
    for message in app_errors {
        match &prefix {
            Some(rel) => out.push(format!("{rel}: {message}")),
            None => out.push(message),
        }
    }
}

fn display_config_path(root: &Path, config_path: Option<&Path>) -> Option<String> {
    let path = config_path?;
    let normalized_root = normalize_path(root);
    let normalized_path = normalize_path(path);
    if normalized_path.starts_with(&normalized_root) {
        let rel = relative_slash_path(&normalized_root, &normalized_path);
        return (!rel.is_empty()).then_some(rel);
    }
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(rel) = relative_between(&normalize_path(&cwd), &normalized_path) {
            return Some(rel);
        }
    }
    let rel = relative_slash_path(&normalized_root, &normalized_path);
    (!rel.is_empty()).then_some(rel)
}

fn relative_between(base: &Path, target: &Path) -> Option<String> {
    let base_parts: Vec<_> = base.components().collect();
    let target_parts: Vec<_> = target.components().collect();
    let common = base_parts
        .iter()
        .zip(&target_parts)
        .take_while(|(a, b)| a == b)
        .count();
    if common == 0 {
        return None;
    }
    let mut parts: Vec<String> = vec!["..".into(); base_parts.len() - common];
    parts.extend(
        target_parts[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    (!parts.is_empty()).then(|| parts.join("/"))
}

#[doc(hidden)]
pub fn validate_rule_options(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<()> {
    let mut errors = Vec::new();
    super::github_actions_action_input_policy::collect_config_errors(
        root,
        config_path,
        config,
        &mut errors,
    );
    super::github_actions_cache_key_policy::collect_config_errors(
        root,
        config_path,
        config,
        &mut errors,
    );
    super::github_actions_no_required_pr_path_filters::collect_config_errors(
        root,
        config_path,
        config,
        &mut errors,
    );
    super::vitest_default_excludes::collect_config_errors(root, config_path, config, &mut errors);
    finish(errors)
}
