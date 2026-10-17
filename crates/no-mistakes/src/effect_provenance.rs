use crate::codebase::dependencies::graph::ResolvedCallSite;
use crate::config::v2::schema::{EffectKindConfig, EffectTargetConfig};
use crate::fx::FxHashMap;

/// Index configured provenance once; consumers project the canonical call sites.
pub(crate) struct EffectProvenance {
    typed: FxHashMap<(String, String), EffectTargetConfig>,
    members: FxHashMap<(String, String, String), EffectTargetConfig>,
}

impl EffectProvenance {
    pub(crate) fn new(config: &EffectKindConfig, categories: &[String]) -> Self {
        let selected = |category: &Option<String>| {
            categories.is_empty()
                || category
                    .as_ref()
                    .is_some_and(|category| categories.contains(category))
        };
        Self {
            typed: config
                .typed_executors
                .iter()
                .filter(|target| selected(&target.category))
                .map(|target| {
                    (
                        (target.module.clone(), target.export.clone()),
                        target.clone(),
                    )
                })
                .collect(),
            members: config
                .returned_members
                .iter()
                .filter(|target| selected(&target.category))
                .map(|target| {
                    (
                        (
                            target.module.clone(),
                            target.export.clone(),
                            target.member.clone(),
                        ),
                        EffectTargetConfig {
                            module: target.module.clone(),
                            export: format!("{}().{}", target.export, target.member),
                            category: target.category.clone(),
                        },
                    )
                })
                .collect(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.typed.is_empty() && self.members.is_empty()
    }

    pub(crate) fn match_site(&self, site: &ResolvedCallSite) -> Option<&EffectTargetConfig> {
        site.typed_from
            .iter()
            .find_map(|origin| self.typed.get(origin))
            .or_else(|| {
                site.returned_member_from
                    .iter()
                    .find_map(|origin| self.members.get(origin))
            })
    }

    pub(crate) fn matching_sites<'a>(
        &'a self,
        site: &'a ResolvedCallSite,
    ) -> impl Iterator<Item = &'a EffectTargetConfig> + 'a {
        site.typed_from
            .iter()
            .filter_map(|origin| self.typed.get(origin))
            .chain(
                site.returned_member_from
                    .iter()
                    .filter_map(|origin| self.members.get(origin)),
            )
    }
}

pub(crate) fn validate(config: &EffectKindConfig, kind: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        config
            .typed_executors
            .iter()
            .all(|target| !target.module.trim().is_empty() && !target.export.trim().is_empty()),
        "effects kind `{kind}` typedExecutors require non-empty module and export"
    );
    anyhow::ensure!(
        config
            .returned_members
            .iter()
            .all(|target| !target.module.trim().is_empty()
                && !target.export.trim().is_empty()
                && !target.member.trim().is_empty()),
        "effects kind `{kind}` returnedMembers require non-empty module, export, and member"
    );
    Ok(())
}
