use super::*;

pub(super) fn selected_site<'a>(
    selection: &'a EffectsSelection,
    site: &crate::codebase::dependencies::graph::ResolvedCallSite,
    target_nodes: &crate::fx::FxHashMap<
        crate::codebase::dependencies::graph::NodeId,
        &'a crate::config::v2::schema::EffectTargetConfig,
    >,
) -> Option<&'a crate::config::v2::schema::EffectTargetConfig> {
    if !site.alternatives.is_empty() {
        let mut targets = site
            .alternatives
            .iter()
            .map(|alternative| selected_site(selection, alternative, target_nodes));
        let first = targets.next().flatten()?;
        for target in targets {
            target?;
        }
        return Some(first);
    }
    let direct = match &site.target {
        crate::codebase::dependencies::graph::ResolvedCallTarget::ModuleExport {
            specifier,
            export_path,
            ..
        } => selection
            .targets
            .get(specifier)
            .and_then(|exports| exports.get(export_path)),
        _ => None,
    };
    let returned = site.returned_from.iter().find_map(|(module, export)| {
        selection
            .returned
            .get(module)
            .and_then(|exports| exports.get(export))
    });
    direct
        .or(returned)
        .or_else(|| selection.provenance.match_site(site))
        .or_else(|| {
            matches!(
                site.target,
                crate::codebase::dependencies::graph::ResolvedCallTarget::RepositoryFunction { .. }
            )
            .then(|| {
                site.target_node
                    .as_ref()
                    .and_then(|node| target_nodes.get(node).copied())
            })
            .flatten()
        })
}

// Explicit executor/provenance selectors keep #2071's all-branches policy;
// plain-name effects retain each independently known fallback identity.
pub(super) fn visit_choice(
    selection: &EffectsSelection,
    site: &crate::codebase::dependencies::graph::ResolvedCallSite,
    nodes: &crate::fx::FxHashMap<
        crate::codebase::dependencies::graph::NodeId,
        &crate::config::v2::schema::EffectTargetConfig,
    >,
    record: &mut impl FnMut(EffectCallAlternative),
) {
    if let Some(target) = selected_site(selection, site, nodes) {
        record(effect_alternative(target));
        return;
    }
    use crate::codebase::dependencies::graph::KnownCallTargetRef;
    for candidate in site.targets() {
        let name = match candidate.target {
            KnownCallTargetRef::Global { name } => name,
            KnownCallTargetRef::ModuleExport { export_path, .. } => export_path,
            _ => continue,
        };
        if let Some(category) = selection.names.get(name) {
            record(EffectCallAlternative {
                callee: name.to_owned(),
                category: category.clone(),
            });
        }
    }
}
