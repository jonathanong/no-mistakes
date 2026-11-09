//! Configured coverage producers require an assertion reachable in the canonical graph.
use super::{analysis_policy::AnalysisDiagnostics, RuleFinding};
use crate::codebase::dependencies::graph::{
    CallRoot, DepGraph, EdgeKind, KnownCallTargetRef, NodeId, ResolvedCallSite, TsFactLookup,
};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use crate::fx::{FxHashMap, FxHashSet};
use anyhow::Result;
use std::path::Path;
mod builtin_tables;
mod builtins;
mod config;
mod external;
mod ownership;
mod source_failures;
mod traversal;
mod uncertainty;
use config::Options;
pub(crate) use ownership::ownership_edges;
pub const RULE_ID: &str = "coverage-producer-assertion-owner";

pub(crate) fn check(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
    facts: &dyn TsFactLookup,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<AnalysisDiagnostics> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut diagnostics = AnalysisDiagnostics::default();
    // Group existing occurrences once; each producer only visits its reachable owners.
    let mut sites: FxHashMap<NodeId, Vec<&ResolvedCallSite>> = FxHashMap::default();
    let replaced: FxHashSet<_> = graph
        .assertion_call_sites()
        .iter()
        .map(|site| (&site.file, site.caller_id, site.offset))
        .collect();
    for site in graph
        .resolved_call_sites()
        .iter()
        .filter(|site| {
            site.invocation != crate::codebase::dependencies::extract::InvocationKind::Callback
                || !replaced.contains(&(&site.file, site.caller_id, site.offset))
        })
        .chain(graph.assertion_call_sites())
    {
        sites.entry(source(site)).or_default().push(site);
    }
    let call_index = builtins::CallIndex::new(&sites, facts);
    for application in config.rule_applications(RULE_ID) {
        let options: Options = application.try_rule_options()?;
        options.validate()?;
        let policy = config.effective_rule_analysis(application)?.unanalyzable;
        let contracts = ownership::resolve(&root, &options, graph, facts)?;
        let filter = super::path_filter::RulePathFilter::new(&root, config, application)?;
        for producer in &options.producers {
            let node = producer.resolve(&root, graph, facts)?;
            if !filter.is_match(node.as_file().expect("callable file")) {
                continue;
            }
            let outcome = traversal::analyze(
                &node,
                &options,
                graph,
                facts,
                (&sites, &call_index),
                &contracts,
            );
            source_failures::require_sources(&root, &outcome)?;
            let reason = if outcome.uncertain() {
                outcome.reason(&root)
            } else if !outcome.assertion {
                "coverage producer has no reachable assertion owner; executing code alone does not validate behavior; call an assertion-bearing helper or configure a genuine ownership contract".to_string()
            } else {
                continue;
            };
            let reason = match application.message.as_deref() {
                Some(message) if outcome.uncertain() => format!("{message}; {reason}"),
                Some(message) => message.to_string(),
                None => reason,
            };
            let finding = RuleFinding {
                rule: RULE_ID.to_string(),
                file: relative_slash_path(&root, node.as_file().expect("callable file")),
                line: declaration_line(&node, sources),
                message: format!("{}: {}#{}", reason, producer.file, producer.symbol),
                import: None,
                target: Some(producer.symbol.clone()),
                causes: if outcome.uncertain() {
                    outcome.finding_causes(&root)
                } else {
                    Vec::new()
                },
            };
            if outcome.uncertain() {
                diagnostics.unanalyzable(policy, finding);
            } else {
                diagnostics.findings.push(finding);
            }
        }
    }
    Ok(diagnostics)
}
pub(super) fn source(site: &ResolvedCallSite) -> NodeId {
    match (site.caller.as_deref(), site.caller_id) {
        (None, _) => NodeId::file(&site.file),
        (Some(symbol), Some(id)) => NodeId::callable(&site.file, symbol, id),
        (Some(symbol), None) => NodeId::symbol(&site.file, symbol),
    }
}
pub(super) fn eligible(site: &ResolvedCallSite, facts: &dyn TsFactLookup) -> bool {
    // Canonical collection verifies class ownership for constructor membership;
    // its bridge executes on construction, unlike reference-only callbacks/members.
    let bridge = crate::codebase::dependencies::graph::is_constructor_bridge(
        site.invocation,
        &site.source_callee,
    );
    if !bridge
        && !matches!(
            site.invocation,
            crate::codebase::dependencies::extract::InvocationKind::Call
                | crate::codebase::dependencies::extract::InvocationKind::Construct
        )
    {
        return false;
    }
    facts.get_ts_facts(&site.file).is_some_and(|facts| {
        !failed(facts)
            && (bridge || {
                let ranges = &facts.unreachable_call_ranges;
                let end = ranges.partition_point(|(owner, start, _)| {
                    (*owner, *start) <= (site.caller_id, site.offset)
                });
                end == 0 || {
                    let (owner, _, until) = ranges[end - 1];
                    owner != site.caller_id || site.offset >= until
                }
            })
    })
}

pub(super) fn generator(node: &NodeId, facts: &dyn TsFactLookup) -> bool {
    matches!(node, NodeId::Symbol { file, callable_id: Some(id), .. }
        if facts.get_ts_facts(file).is_some_and(|file| file.value_flow.generator_callables.contains(id)))
}
#[cfg(test)]
mod tests;

fn declaration_line(node: &NodeId, sources: &crate::codebase::ts_source::SourceStore) -> usize {
    let offset = match node {
        NodeId::Symbol {
            callable_id: Some(id),
            ..
        } => id.0 as usize,
        _ => 0,
    };
    sources
        .read_path(node.as_file().expect("callable file"))
        .map(|source| {
            source
                .bytes()
                .take(offset)
                .filter(|byte| *byte == b'\n')
                .count()
                + 1
        })
        .unwrap_or(1)
}

pub(super) fn failed(facts: &crate::codebase::ts_source::facts::TsFileFacts) -> bool {
    facts.parse_error.is_some() || facts.operational_error.is_some() || facts.fatal_parse_error
}
pub(super) fn unavailable(file: &Path, facts: &dyn TsFactLookup) -> bool {
    facts.get_ts_facts(file).is_none_or(failed)
}
