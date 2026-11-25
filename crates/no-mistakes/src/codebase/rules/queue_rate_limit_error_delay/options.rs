use super::RULE_ID;
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

const MAX_DELAY_ARGUMENT: usize = 32;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Options {
    pub(super) targets: Vec<TargetInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct TargetInput {
    pub(super) module: String,
    pub(super) export: String,
    #[serde(default)]
    pub(super) delay_argument: usize,
    #[serde(default)]
    pub(super) delay_property: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Target {
    pub(super) module: String,
    pub(super) export: String,
    pub(super) delay_argument: usize,
    pub(super) delay_property: Option<String>,
}

impl Target {
    pub(super) fn label(&self) -> String {
        format!("{}#{}", self.module, self.export)
    }

    pub(super) fn terminal_name(&self) -> &str {
        self.export
            .rsplit('.')
            .next()
            .unwrap_or(self.export.as_str())
    }
}

pub(super) fn validate(options: &Options) -> Result<Vec<Target>> {
    if options.targets.is_empty() {
        bail!("{RULE_ID}: options.targets must contain at least one target");
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(options.targets.len());
    for (index, target) in options.targets.iter().enumerate() {
        let module = target.module.trim();
        let export = target.export.trim();
        if module.is_empty() || export.is_empty() {
            bail!("{RULE_ID}: options.targets[{index}] requires non-empty `module` and `export`");
        }
        if target.delay_argument >= MAX_DELAY_ARGUMENT {
            bail!(
                "{RULE_ID}: options.targets[{index}].delayArgument must be less than {MAX_DELAY_ARGUMENT}"
            );
        }
        let delay_property = match &target.delay_property {
            Some(prop) => {
                let trimmed = prop.trim();
                if trimmed.is_empty() {
                    bail!(
                        "{RULE_ID}: options.targets[{index}].delayProperty must be non-empty when set"
                    );
                }
                Some(trimmed.to_string())
            }
            None => None,
        };
        let key = (module.to_string(), export.to_string());
        if !seen.insert(key) {
            bail!("{RULE_ID}: options.targets lists `{module}` export `{export}` more than once");
        }
        out.push(Target {
            module: module.to_string(),
            export: export.to_string(),
            delay_argument: target.delay_argument,
            delay_property,
        });
    }
    Ok(out)
}

pub(crate) fn collect_application_errors(
    _root: &Path,
    _config: &NoMistakesConfig,
    rule: &RuleDef,
) -> Result<BTreeSet<String>> {
    let options = rule.strict_rule_options::<Options>()?;
    match validate(&options) {
        Ok(_) => Ok(BTreeSet::new()),
        Err(error) => Ok(BTreeSet::from([format!("{error:#}")])),
    }
}
