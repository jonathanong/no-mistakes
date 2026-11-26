use super::options::Target;
use crate::codebase::dependencies::graph::{DepGraph, ResolvedCallSite};
use crate::codebase::rules::no_single_enqueue_in_loop::modules::{
    matches_resolved, ResolvedModules,
};
use crate::codebase::ts_source::facts::TsFileFacts;

pub(super) fn target_matches(
    graph: &DepGraph,
    target: &Target,
    site: &ResolvedCallSite,
    resolved: &ResolvedModules,
) -> bool {
    graph.matches_module_export(&target.module, &target.export, site)
        || resolved
            .files(&target.module)
            .is_some_and(|files| matches_resolved(files, &target.export, site))
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
