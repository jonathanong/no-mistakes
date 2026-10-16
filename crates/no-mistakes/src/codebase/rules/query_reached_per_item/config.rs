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
    returned: crate::fx::FxHashMap<&'a str, crate::fx::FxHashSet<&'a str>>,
    nodes: crate::fx::FxHashSet<crate::codebase::dependencies::graph::NodeId>,
    targets: crate::fx::FxHashMap<&'a str, crate::fx::FxHashSet<&'a str>>,
    provenance: crate::effect_provenance::EffectProvenance,
}

impl<'a> Sinks<'a> {
    pub(super) fn new(
        config: &'a EffectKindConfig,
        graph: &crate::codebase::dependencies::graph::DepGraph,
    ) -> Self {
        let mut targets: crate::fx::FxHashMap<_, crate::fx::FxHashSet<_>> =
            crate::fx::FxHashMap::default();
        for target in &config.targets {
            targets
                .entry(target.module.as_str())
                .or_default()
                .insert(target.export.as_str());
        }
        let nodes = config
            .targets
            .iter()
            .flat_map(|target| graph.module_export_nodes(&target.module, &target.export))
            .collect();
        let mut returned: crate::fx::FxHashMap<_, crate::fx::FxHashSet<_>> =
            crate::fx::FxHashMap::default();
        for target in &config.returned_executors {
            returned
                .entry(target.module.as_str())
                .or_default()
                .insert(target.export.as_str());
        }
        Self {
            provenance: crate::effect_provenance::EffectProvenance::new(config, &[]),
            returned,
            config,
            targets,
            nodes,
        }
    }

    pub(super) fn match_site(&self, site: &ResolvedCallSite) -> Option<bool> {
        if !site.alternatives.is_empty() {
            return site
                .alternatives
                .iter()
                .try_fold(false, |transaction, alternative| {
                    self.match_site(alternative)
                        .map(|matched| transaction || matched)
                });
        }
        if self.provenance.match_site(site).is_some() {
            return Some(true);
        }
        let returned = site.returned_from.iter().any(|(module, export)| {
            self.returned
                .get(module.as_str())
                .is_some_and(|exports| exports.contains(export.as_str()))
        });
        sink(self.config, &site.source_callee)
            .filter(|transaction| {
                !*transaction
                    || site.source_callee.contains('.')
                    || !site.returned_from.is_empty()
                    || !matches!(site.target, ResolvedCallTarget::RepositoryFunction { .. })
            })
            .map(|transaction| transaction || returned)
            .or_else(|| {
                if returned {
                    return Some(true);
                }
                if matches!(site.target, ResolvedCallTarget::RepositoryFunction { .. })
                    && site
                        .target_node
                        .as_ref()
                        .is_some_and(|node| self.nodes.contains(node))
                {
                    return Some(false);
                }
                let ResolvedCallTarget::ModuleExport {
                    specifier,
                    export_path,
                    ..
                } = &site.target
                else {
                    return None;
                };
                if export_path.contains('.') {
                    if let Some(transaction) = sink(self.config, export_path) {
                        return Some(transaction);
                    }
                }
                self.targets
                    .get(specifier.as_str())
                    .is_some_and(|exports| exports.contains(export_path.as_str()))
                    .then_some(false)
            })
    }
}

pub(super) fn sink(config: &EffectKindConfig, name: &str) -> Option<bool> {
    if config
        .transaction_functions
        .iter()
        .any(|value| value == name)
    {
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
        crate::effect_provenance::validate(effect, kind)?;
        anyhow::ensure!(
            effect
                .targets
                .iter()
                .chain(&effect.returned_executors)
                .all(|target| {
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
        effect_selectors: config
            .effects
            .values()
            .any(|effect| !effect.targets.is_empty()),
        ..Default::default()
    })
}
