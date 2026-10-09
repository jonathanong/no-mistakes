use super::RULE_ID;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::config::v2::{schema::EffectKindConfig, NoMistakesConfig};
use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Options {
    pub(super) effects: Vec<String>,
    pub(super) allow: Vec<Allowance>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Allowance {
    pub(super) file: String,
    pub(super) callee: Option<String>,
    pub(super) line: Option<u32>,
}

pub(super) fn sink(config: &EffectKindConfig, name: &str) -> Option<bool> {
    if config.transaction_functions.iter().any(|value| {
        value == name
            || name
                .rsplit_once('.')
                .is_some_and(|(_, terminal)| value == terminal)
    }) {
        Some(true)
    } else if config
        .functions
        .iter()
        .chain(config.categories.values().flatten())
        .any(|value| {
            value == name
                || name
                    .rsplit_once('.')
                    .is_some_and(|(_, terminal)| value == terminal)
        })
    {
        Some(false)
    } else {
        None
    }
}
pub(super) fn exempt(config: &EffectKindConfig, name: &str) -> bool {
    config.batch_functions.iter().any(|value| {
        value == name
            || name
                .rsplit_once('.')
                .is_some_and(|(_, terminal)| value == terminal)
    })
}

pub(super) fn validate(options: &Options, config: &NoMistakesConfig) -> Result<()> {
    if options.effects.is_empty() {
        bail!("{RULE_ID}: options.effects must select at least one configured effect kind");
    }
    if options
        .allow
        .iter()
        .any(|allow| allow.file.is_empty() || allow.line == Some(0))
    {
        bail!("{RULE_ID}: allow entries require a non-empty file and positive line when supplied");
    }
    for kind in &options.effects {
        if !config.effects.contains_key(kind) {
            bail!("{RULE_ID}: unknown effects kind `{kind}`");
        }
    }
    Ok(())
}

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    config.rule_configured(RULE_ID).then(|| GraphBuildPlan {
        calls: true,
        ..Default::default()
    })
}
