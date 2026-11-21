use super::modules::ResolvedModules;
use super::options::Targets;
use crate::codebase::dependencies::extract::{CallableId, InvocationKind};
use crate::codebase::dependencies::graph::{DepGraph, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::{NoMistakesConfig, RuleDef};
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

type PositionKey = (PathBuf, u32, Option<CallableId>, bool);

pub(super) fn scan(
    root: &Path,
    config: &NoMistakesConfig,
    application: &RuleDef,
    targets: &Targets,
    graph: &DepGraph,
    facts: &crate::codebase::check_facts::CheckFactMap,
    resolved: &ResolvedModules,
) -> Result<Vec<RuleFinding>> {
    let selected = selected_files(root, config, application, facts)?;
    let positions = per_item_positions(facts);
    let mut findings = Vec::new();
    for site in graph.resolved_call_sites() {
        let file = crate::codebase::ts_resolver::normalize_path(&site.file);
        if !selected.contains(&file) {
            continue;
        }
        let Some((line, construct)) = positions.get(&(
            file.clone(),
            site.offset,
            site.caller_id,
            site.invocation == InvocationKind::Callback,
        )) else {
            continue;
        };
        if !reported_construct(construct) {
            continue;
        }
        if bulk_skips(graph, targets, site, resolved) {
            continue;
        }
        let Some((module, export)) = matching_target(graph, &targets.single, site, resolved) else {
            continue;
        };
        findings.push(finding(
            root,
            application,
            &file,
            *line,
            construct,
            site,
            (module, export),
        ));
    }
    Ok(findings)
}

fn selected_files(
    root: &Path,
    config: &NoMistakesConfig,
    application: &RuleDef,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<HashSet<PathBuf>> {
    let files =
        super::super::selected_rule_files(root, config, application, facts.graph_file_universe())?;
    Ok(files
        .into_iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
        .collect())
}

fn per_item_positions(
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> BTreeMap<PositionKey, (u32, String)> {
    let mut positions = BTreeMap::new();
    for file in facts.graph_file_universe() {
        let Some(ts) = facts.get_ts_facts(file) else {
            continue;
        };
        let file = crate::codebase::ts_resolver::normalize_path(file);
        for call in &ts.per_item_calls {
            positions.insert(
                (file.clone(), call.offset, call.caller_id, call.is_callback),
                (call.line, call.construct.clone()),
            );
        }
    }
    positions
}

fn reported_construct(construct: &str) -> bool {
    matches!(
        construct,
        "for"
            | "for-of"
            | "for-in"
            | "while"
            | "do"
            | "map callback"
            | "forEach callback"
            | "flatMap callback"
    )
}

fn bulk_skips(
    graph: &DepGraph,
    targets: &Targets,
    site: &ResolvedCallSite,
    resolved: &ResolvedModules,
) -> bool {
    targets.bulk.iter().any(|(module, export)| {
        target_matches(graph, module, export, site, resolved)
            && !same_resolved_export(module, export, &targets.single, resolved)
    })
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

fn matching_target<'a>(
    graph: &DepGraph,
    targets: &'a [(String, String)],
    site: &ResolvedCallSite,
    resolved: &ResolvedModules,
) -> Option<(&'a str, &'a str)> {
    targets.iter().find_map(|(module, export)| {
        target_matches(graph, module, export, site, resolved)
            .then_some((module.as_str(), export.as_str()))
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

fn finding(
    root: &Path,
    application: &RuleDef,
    file: &Path,
    line: u32,
    construct: &str,
    site: &ResolvedCallSite,
    target: (&str, &str),
) -> RuleFinding {
    let (module, export) = target;
    let body = format!(
        "single enqueue `{}` (`{module}#{export}`) inside {construct}; call the configured bulk API once with the collection",
        site.source_callee
    );
    let message = match application.message.as_deref() {
        Some(prefix) => format!("{prefix}: {body}"),
        None => body,
    };
    RuleFinding {
        causes: Vec::new(),
        rule: super::RULE_ID.to_string(),
        file: relative_slash_path(root, file),
        line: line as usize,
        message,
        import: Some(site.source_callee.clone()),
        target: Some(format!("{module}#{export}")),
    }
}
