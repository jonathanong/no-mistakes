use crate::codebase::lockfile::pnpm::{
    parse_importers_for_impact, PnpmImpactImporter, PnpmImporterDependency,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn importer_change_names(
    old: &str,
    new: &str,
    changed: impl IntoIterator<Item = String>,
) -> Vec<String> {
    let mut impacted: BTreeSet<String> = changed.into_iter().collect();
    let old_importers = parse_importers_for_impact(old);
    let new_importers = parse_importers_for_impact(new);
    let old_by_path: BTreeMap<_, _> = old_importers
        .iter()
        .map(|importer| (&importer.path, importer))
        .collect();
    let new_by_path: BTreeMap<_, _> = new_importers
        .iter()
        .map(|importer| (&importer.path, importer))
        .collect();
    let paths: BTreeSet<_> = old_importers
        .iter()
        .chain(&new_importers)
        .map(|importer| &importer.path)
        .collect();
    for path in paths {
        let old = old_by_path.get(path).copied();
        let new = new_by_path.get(path).copied();
        if old != new {
            add_changed_importer_dependency_names(&mut impacted, old, new);
        }
    }
    for importer in new_importers.into_iter().chain(old_importers) {
        for dependency in dependencies(importer) {
            if dependency
                .resolution_name
                .as_ref()
                .is_some_and(|name| impacted.contains(name))
                || impacted.contains(&dependency.alias)
            {
                impacted.insert(dependency.alias);
            }
        }
    }
    impacted.into_iter().collect()
}

fn dependencies(importer: PnpmImpactImporter) -> impl Iterator<Item = PnpmImporterDependency> {
    importer
        .dependencies
        .into_iter()
        .chain(importer.dev_dependencies)
        .chain(importer.optional_dependencies)
        .chain(importer.peer_dependencies)
}

fn add_changed_importer_dependency_names(
    impacted: &mut BTreeSet<String>,
    old: Option<&PnpmImpactImporter>,
    new: Option<&PnpmImpactImporter>,
) {
    for (old, new) in [
        (
            old.map_or(&[] as &[PnpmImporterDependency], |importer| {
                importer.dependencies.as_slice()
            }),
            new.map_or(&[] as &[PnpmImporterDependency], |importer| {
                importer.dependencies.as_slice()
            }),
        ),
        (
            old.map_or(&[] as &[PnpmImporterDependency], |importer| {
                importer.dev_dependencies.as_slice()
            }),
            new.map_or(&[] as &[PnpmImporterDependency], |importer| {
                importer.dev_dependencies.as_slice()
            }),
        ),
        (
            old.map_or(&[] as &[PnpmImporterDependency], |importer| {
                importer.optional_dependencies.as_slice()
            }),
            new.map_or(&[] as &[PnpmImporterDependency], |importer| {
                importer.optional_dependencies.as_slice()
            }),
        ),
        (
            old.map_or(&[] as &[PnpmImporterDependency], |importer| {
                importer.peer_dependencies.as_slice()
            }),
            new.map_or(&[] as &[PnpmImporterDependency], |importer| {
                importer.peer_dependencies.as_slice()
            }),
        ),
    ] {
        add_changed_dependency_group(impacted, old, new);
    }
}

fn add_changed_dependency_group(
    impacted: &mut BTreeSet<String>,
    old: &[PnpmImporterDependency],
    new: &[PnpmImporterDependency],
) {
    let old_by_alias: BTreeMap<_, _> = old.iter().map(|dep| (&dep.alias, dep)).collect();
    let new_by_alias: BTreeMap<_, _> = new.iter().map(|dep| (&dep.alias, dep)).collect();
    let aliases: BTreeSet<_> = old_by_alias
        .keys()
        .chain(new_by_alias.keys())
        .copied()
        .collect();
    for alias in aliases {
        for dependency in old_by_alias
            .get(alias)
            .copied()
            .into_iter()
            .chain(new_by_alias.get(alias).copied())
        {
            if old_by_alias.get(alias) != new_by_alias.get(alias) {
                impacted.insert(
                    dependency
                        .resolution_name
                        .clone()
                        .unwrap_or_else(|| dependency.alias.clone()),
                );
                impacted.insert(dependency.alias.clone());
            }
        }
    }
}
