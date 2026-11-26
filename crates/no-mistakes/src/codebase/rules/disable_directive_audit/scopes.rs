use super::known_rules::known_rule;
use crate::config::v2::schema::{NoMistakesConfig, RuleDef, RuleScope};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) fn comment_scopes<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
    audit_hygiene: bool,
) -> Result<BTreeMap<PathBuf, BTreeSet<&'a str>>> {
    if config.rules.is_empty() {
        let scope = RuleDef {
            scope: config.projects.is_empty().then_some(RuleScope::Repository),
            projects: config.projects.keys().cloned().collect(),
            ..RuleDef::default()
        };
        return Ok(
            super::super::selected_rule_files(root, config, &scope, files)?
                .into_iter()
                .map(|file| (file, BTreeSet::new()))
                .collect(),
        );
    }

    let mut scopes: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for rule in &config.rules {
        let mut selected_rule = rule.clone();
        selected_rule.enabled = true;
        let selected = super::super::selected_rule_files(root, config, &selected_rule, files)?;
        let enabled = audit_hygiene
            && rule.enabled
            && known_rule(&rule.rule)
            && config.rule_configured(&rule.rule);
        for file in selected {
            let rules = scopes.entry(file).or_default();
            if enabled {
                rules.insert(rule.rule.as_str());
            }
        }
    }
    Ok(scopes)
}
