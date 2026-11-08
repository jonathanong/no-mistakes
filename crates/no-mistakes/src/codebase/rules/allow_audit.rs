use super::config_diagnostic::ConfigAnchor;
use super::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

mod exceptions;
mod missing;
mod scope;
pub(crate) use exceptions::ExceptionAudit;
pub(crate) use missing::{findings as missing_file_findings, is_literal_file, MissingFiles};
pub(crate) use scope::CheckedScope;

pub(crate) const RULE_ID: &str = "unused-allow-entry";

pub(crate) struct AllowUsage(Vec<AtomicBool>);

impl AllowUsage {
    pub(crate) fn new(count: usize) -> Self {
        Self((0..count).map(|_| AtomicBool::new(false)).collect())
    }

    pub(crate) fn suppress(&self, matches: impl Iterator<Item = bool>) -> bool {
        let mut suppressed = false;
        for (used, matched) in self.0.iter().zip(matches) {
            if matched {
                used.store(true, Ordering::Relaxed);
                suppressed = true;
            }
        }
        suppressed
    }

    pub(crate) fn unused(&self) -> impl Iterator<Item = usize> + '_ {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(index, used)| (!used.load(Ordering::Relaxed)).then_some(index))
    }
}

pub(crate) struct AllowAudit {
    pub(crate) usage: AllowUsage,
    anchor: ConfigAnchor,
    rule: String,
    option: String,
    missing: Option<MissingFiles>,
    mapping: bool,
}

impl AllowAudit {
    pub(crate) fn new(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
        sources: Option<&SourceStore>,
        option: &str,
        count: usize,
    ) -> Self {
        Self {
            usage: AllowUsage::new(count),
            missing: None,
            mapping: rule
                .options
                .get(option)
                .is_some_and(serde_yaml::Value::is_mapping),
            anchor: if count == 0 {
                ConfigAnchor::fallback()
            } else {
                ConfigAnchor::resolve(root, config, sources, rule)
            },
            rule: rule.rule.clone(),
            option: option.to_string(),
        }
    }

    pub(crate) fn new_files(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
        sources: Option<&SourceStore>,
        option: &str,
        count: usize,
    ) -> Self {
        let mut audit = Self::new(root, config, rule, sources, option, count);
        if count > 0 {
            audit.missing = MissingFiles::new(root, config, rule, sources);
        }
        audit
    }

    pub(crate) fn audits_missing_files(&self) -> bool {
        self.missing.is_some()
    }

    pub(crate) fn finish_files(
        &self,
        eligible: impl Fn(usize) -> bool,
        file: impl Fn(usize) -> String,
        selector: impl Fn(usize) -> String,
    ) -> Vec<RuleFinding> {
        self.finish(
            |index| {
                eligible(index)
                    || self
                        .missing
                        .as_ref()
                        .is_some_and(|scope| scope.contains(&file(index)))
            },
            selector,
        )
    }

    pub(crate) fn finish(
        &self,
        eligible: impl Fn(usize) -> bool,
        selector: impl Fn(usize) -> String,
    ) -> Vec<RuleFinding> {
        self.usage
            .unused()
            .filter(|index| eligible(*index))
            .map(|index| {
                let selector = selector(index);
                let line = if self.mapping {
                    self.anchor.option_key_line(&self.option, &selector)
                } else {
                    self.anchor.option_item_line(&self.option, index)
                };
                stale_finding(&self.anchor, &self.rule, &self.option, selector, line)
            })
            .collect()
    }
}

pub(crate) fn stale_finding(
    anchor: &ConfigAnchor,
    rule: &str,
    option: &str,
    selector: String,
    line: usize,
) -> RuleFinding {
    anchor.emit(
        RULE_ID,
        line,
        format!("stale {rule} {option} entry `{selector}` matched no finding in this invocation; remove the entry so exceptions cannot hide stale rule intent"),
        Some(rule.to_string()),
        Some(selector),
    )
}

#[cfg(test)]
mod tests;
