use super::options::Target;
use crate::codebase::dependencies::graph::{DepGraph, KnownCallTargetRef, ResolvedCallSite};
use crate::codebase::rules::no_single_enqueue_in_loop::modules::{
    matches_resolved_candidate, ResolvedModules,
};
use crate::codebase::ts_source::facts::TsFileFacts;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TargetFit {
    All,
    None,
    Mixed,
}

pub(super) fn target_fit(
    graph: &DepGraph,
    target: &Target,
    site: &ResolvedCallSite,
    resolved: &ResolvedModules,
) -> TargetFit {
    if site.has_unknown_target() || site.targets().is_empty() {
        return TargetFit::None;
    }
    let files = resolved.files(&target.module);
    let mut matched = false;
    let mut missed = false;
    for candidate in site.targets() {
        let hit = graph.candidate_matches_module_export(&target.module, &target.export, candidate)
            || files.is_some_and(|files| {
                matches_resolved_candidate(files, &target.export, candidate.target)
            });
        if hit {
            matched = true;
        } else {
            missed = true;
        }
    }
    match (matched, missed) {
        (true, false) => TargetFit::All,
        (true, true) => TargetFit::Mixed,
        (false, _) => TargetFit::None,
    }
}

pub(super) fn has_local_value_export_target(site: &ResolvedCallSite) -> bool {
    site.targets().any(|candidate| {
        matches!(
            candidate.target,
            KnownCallTargetRef::ModuleExport {
                specifier,
                repository_target: None,
                ..
            } if specifier.starts_with(['.', '/', '#'])
        )
    })
}

pub(super) fn candidate_targets<'a>(
    targets: &'a [Target],
    site: &ResolvedCallSite,
    ts_facts: &TsFileFacts,
) -> Vec<&'a Target> {
    let terminal = callee_terminal(&site.source_callee);
    let imported = imported_terminal(ts_facts, &site.source_callee);
    targets
        .iter()
        .filter(|target| {
            let name = Some(target.terminal_name());
            terminal == name || imported == name
        })
        .collect()
}

fn imported_terminal<'a>(ts_facts: &'a TsFileFacts, callee: &str) -> Option<&'a str> {
    ts_facts
        .imported_bindings
        .iter()
        .find(|binding| !binding.is_type_only && binding.local == callee)
        .and_then(|binding| callee_terminal(&binding.imported))
}

fn callee_terminal(callee: &str) -> Option<&str> {
    let tail = callee.rsplit('.').next().unwrap_or(callee).trim();
    (!tail.is_empty()).then_some(tail)
}
