use super::display_config_path;
use crate::config::source_locations::ConfigPath;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use std::collections::BTreeSet;
use std::path::Path;

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
    let prefix = source.map(|file| display_config_path(root, file));
    for message in app_errors {
        match &prefix {
            Some(rel) => out.push(format!("{rel}: {message}")),
            None => out.push(message),
        }
    }
}

pub(in crate::codebase::rules) fn collect_config_errors(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    rule_id: &str,
    collect: impl Fn(&Path, &NoMistakesConfig, &RuleDef) -> anyhow::Result<BTreeSet<String>>,
    errors: &mut Vec<String>,
) {
    for (index, rule) in indexed_rule_applications(config, rule_id) {
        let app_errors = collect(root, config, rule).unwrap_or_else(|error| {
            let mut errors = BTreeSet::from([format!("{error:#}")]);
            if let Err(error) =
                crate::codebase::rules::path_filter::RulePathFilter::new(root, config, rule)
            {
                errors.insert(format!("{error:#}"));
            }
            errors
        });
        push_application_errors(root, config, index, config_path, app_errors, errors);
    }
}
