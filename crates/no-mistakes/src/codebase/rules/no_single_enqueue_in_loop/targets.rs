use super::modules::ResolvedModules;
use super::options::Targets;
use crate::codebase::dependencies::graph::{DepGraph, KnownCallTargetRef, ResolvedCallSite};
use std::collections::BTreeSet;

pub(super) fn matching_target(
    graph: &DepGraph,
    targets: &Targets,
    site: &ResolvedCallSite,
    resolved: &ResolvedModules,
) -> Option<String> {
    if targets.bulk.iter().any(|(module, export)| {
        target_matches(graph, module, export, site, resolved)
            && !same_resolved_export(module, export, &targets.single, resolved)
    }) {
        return None;
    }
    targets
        .single
        .iter()
        .find(|(module, export)| target_matches(graph, module, export, site, resolved))
        .map(|(module, export)| format!("{module}#{export}"))
        .or_else(|| matching_pattern(graph, targets, site, resolved))
}

fn matching_pattern(
    graph: &DepGraph,
    targets: &Targets,
    site: &ResolvedCallSite,
    resolved: &ResolvedModules,
) -> Option<String> {
    let single = targets.single_export_pattern.as_ref()?;
    if site.has_unknown_target() {
        return None;
    }
    let mut matches = site.targets().map(|candidate| {
        let KnownCallTargetRef::ModuleExport {
            specifier,
            export_path,
            ..
        } = candidate.target
        else {
            return None;
        };
        (single.is_match(export_path)
            && targets
                .bulk_export_pattern
                .as_ref()
                .is_none_or(|bulk| !bulk.is_match(export_path))
            && targets
                .module_globs
                .as_ref()
                .is_none_or(|globs| globs.is_match(specifier))
            && !targets.bulk.iter().any(|(module, export)| {
                graph.candidate_matches_module_export(module, export, candidate)
                    || resolved.files(module).is_some_and(|files| {
                        super::modules::matches_resolved_candidate(files, export, candidate.target)
                    })
            }))
        .then_some((specifier, export_path))
    });
    let first = matches.next()??;
    if matches.len() == 0 {
        return Some(format!("{}#{}", first.0, first.1));
    }
    let mut identities = BTreeSet::from([first]);
    for identity in matches {
        identities.insert(identity?);
    }
    Some(
        identities
            .into_iter()
            .map(|(module, export)| format!("{module}#{export}"))
            .collect::<Vec<_>>()
            .join(", "),
    )
}

fn same_resolved_export(
    module: &str,
    export: &str,
    single: &[(String, String)],
    resolved: &ResolvedModules,
) -> bool {
    let Some(bulk_files) = resolved.files(module) else {
        return false;
    };
    single.iter().any(|(single_module, single_export)| {
        single_export == export
            && resolved
                .files(single_module)
                .is_some_and(|files| files.iter().any(|file| bulk_files.contains(file)))
    })
}

fn target_matches(
    graph: &DepGraph,
    module: &str,
    export: &str,
    site: &ResolvedCallSite,
    resolved: &ResolvedModules,
) -> bool {
    graph.matches_module_export(module, export, site)
        || resolved
            .files(module)
            .is_some_and(|files| super::modules::matches_resolved(files, export, site))
}
