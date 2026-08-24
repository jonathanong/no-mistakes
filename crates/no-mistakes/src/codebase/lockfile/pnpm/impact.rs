use crate::codebase::lockfile::pnpm::parse_importers_for_impact;
use std::collections::{BTreeMap, BTreeSet};

mod graph;
mod importer_changes;
use graph::{affected_locators, dependency_locator};
use importer_changes::importer_change_names;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PnpmLocator {
    name: String,
    version: String,
    peer_context: String,
}

/// Direct package names whose importers can be affected by changed resolved
/// packages. This keeps lockfile planning on the shared pnpm parser rather
/// than trying to infer ownership from a second YAML reader.
pub(crate) fn impact_names(
    old: &str,
    new: &str,
    _changed: impl IntoIterator<Item = String>,
) -> Vec<String> {
    let mut names: BTreeSet<_> = affected_locators(old, new)
        .into_iter()
        .map(|locator| locator.name)
        .collect();
    names.extend(importer_change_names(old, new, _changed));
    names.into_iter().collect()
}

/// Importer paths which directly resolve each affected package name. This is
/// internal planning metadata; the public parser keeps its original shape.
pub(crate) fn impact_importer_paths(
    old: &str,
    new: &str,
    _names: &[String],
) -> BTreeMap<String, Vec<String>> {
    let exact = affected_locators(old, new);
    let mut paths: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for importer in parse_importers_for_impact(new)
        .into_iter()
        .chain(parse_importers_for_impact(old))
    {
        for dependency in importer
            .dependencies
            .into_iter()
            .chain(importer.dev_dependencies)
            .chain(importer.optional_dependencies)
            .chain(importer.peer_dependencies)
        {
            let Some(locator) = dependency_locator(&dependency) else {
                continue;
            };
            if exact.contains(&locator) {
                paths
                    .entry(locator.name)
                    .or_default()
                    .insert(importer.path.clone());
            }
        }
    }
    let importer_names: BTreeSet<_> = importer_change_names(old, new, std::iter::empty())
        .into_iter()
        .collect();
    for importer in parse_importers_for_impact(new)
        .into_iter()
        .chain(parse_importers_for_impact(old))
    {
        for dependency in importer
            .dependencies
            .into_iter()
            .chain(importer.dev_dependencies)
            .chain(importer.optional_dependencies)
            .chain(importer.peer_dependencies)
        {
            for name in [dependency.resolution_name, Some(dependency.alias)]
                .into_iter()
                .flatten()
            {
                if importer_names.contains(&name) {
                    paths.entry(name).or_default().insert(importer.path.clone());
                }
            }
        }
    }
    paths
        .into_iter()
        .map(|(name, paths)| (name, paths.into_iter().collect()))
        .collect()
}
