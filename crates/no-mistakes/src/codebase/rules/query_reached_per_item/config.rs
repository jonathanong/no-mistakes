use super::RULE_ID;
use crate::codebase::dependencies::graph::{GraphBuildPlan, ResolvedCallSite, ResolvedCallTarget};
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

/// Index selectors once per family; every occurrence uses the prepared binding.
pub(super) struct Sinks<'a> {
    config: &'a EffectKindConfig,
    targets: crate::fx::FxHashMap<&'a str, crate::fx::FxHashSet<&'a str>>,
}

impl<'a> Sinks<'a> {
    pub(super) fn new(config: &'a EffectKindConfig) -> Self {
        let mut targets: crate::fx::FxHashMap<_, crate::fx::FxHashSet<_>> =
            crate::fx::FxHashMap::default();
        for target in &config.targets {
            targets
                .entry(target.module.as_str())
                .or_default()
                .insert(target.export.as_str());
        }
        Self { config, targets }
    }

    pub(super) fn match_site(&self, site: &ResolvedCallSite) -> Option<bool> {
        // Keep legacy spelling/transaction precedence when both selectors match.
        sink(self.config, &site.source_callee).or_else(|| {
            let ResolvedCallTarget::ModuleExport {
                specifier,
                export_path,
                ..
            } = &site.target
            else {
                return None;
            };
            self.targets
                .get(specifier.as_str())
                .is_some_and(|exports| exports.contains(export_path.as_str()))
                .then_some(false)
        })
    }
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
        let Some(effect) = config.effects.get(kind) else {
            bail!("{RULE_ID}: unknown effects kind `{kind}`");
        };
        anyhow::ensure!(
            effect.targets.iter().all(|target| {
                !target.module.trim().is_empty() && !target.export.trim().is_empty()
            }),
            "effects kind `{kind}` targets require non-empty module and export"
        );
    }
    Ok(())
}

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    config.rule_configured(RULE_ID).then(|| GraphBuildPlan {
        calls: true,
        ..Default::default()
    })
}
