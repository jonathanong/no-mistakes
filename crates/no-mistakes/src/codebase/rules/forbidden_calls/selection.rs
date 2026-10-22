use super::super::RuleFinding;
use super::config::{self, Options, Traversal};
use super::findings::finding_for_site_with_arguments;
use super::roots;
use super::RULE_ID;
use crate::codebase::dependencies::graph::{CallTraversal, DepGraph, NodeId, ResolvedCallSite};
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use crate::fx::FxHashSet;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) mod invocations;
mod plan;
use invocations::{allowed_invocations, site_source_node};
pub(crate) use plan::graph_plan;
pub(crate) fn check_with_graph(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
    vitest: Option<&super::super::PreparedVitestProjectCatalog>,
    playwright: Option<&super::super::PreparedPlaywrightProjectCatalog>,
    graph_files: &[PathBuf],
    facts: Option<&dyn crate::codebase::dependencies::graph::TsFactLookup>,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut findings = Vec::new();
    let source_files = plan::source_files(config, graph_files);
    for (index, application) in config.rule_applications(RULE_ID).into_iter().enumerate() {
        let options: Options = application.try_rule_options()?;
        findings.extend(check_application(ApplicationCheck {
            root: &root,
            config,
            application,
            index: index + 1,
            options: &options,
            graph,
            vitest,
            playwright,
            graph_files,
            facts,
            source_files: source_files.as_ref(),
        })?);
    }
    Ok(findings)
}
struct ApplicationCheck<'a> {
    root: &'a Path,
    config: &'a NoMistakesConfig,
    application: &'a RuleDef,
    index: usize,
    options: &'a Options,
    graph: &'a DepGraph,
    vitest: Option<&'a super::super::PreparedVitestProjectCatalog>,
    playwright: Option<&'a super::super::PreparedPlaywrightProjectCatalog>,
    graph_files: &'a [PathBuf],
    facts: Option<&'a dyn crate::codebase::dependencies::graph::TsFactLookup>,
    source_files: Option<&'a crate::codebase::check_facts::PathMembership<'a>>,
}
fn check_application(input: ApplicationCheck<'_>) -> Result<Vec<RuleFinding>> {
    let ApplicationCheck {
        root,
        config,
        application,
        index,
        options,
        graph,
        vitest,
        playwright,
        graph_files,
        facts,
        source_files,
    } = input;
    config::validate(options)?;
    if let Some(source_files) = source_files {
        let source_facts = facts.ok_or_else(|| {
            anyhow::anyhow!(
                "{RULE_ID}: prepared facts are missing source calls required by file traversal"
            )
        })?;
        return super::source_only::check(
            root,
            config,
            application,
            index,
            options,
            source_files,
            source_facts,
        );
    }
    let target_roots = super::super::target_roots(root, config, application);
    let roots = roots::expand(roots::ExpandRequest {
        root,
        options,
        graph,
        vitest,
        playwright,
        graph_files,
        target_roots: &target_roots,
    })?;
    let path_filter = super::super::path_filter::RulePathFilter::new(root, config, application)?;
    let source_nodes = reachable_source_nodes(graph, &roots, options);
    let invocations = allowed_invocations(&options.invocations);
    let label = application.name.as_deref().map_or_else(
        || format!("application #{index}"),
        |name| format!("{name}, application #{index}"),
    );
    let argument_facts = options
        .targets
        .iter()
        .any(|target| target.arguments().is_some())
        .then_some(facts)
        .flatten();
    let findings = candidate_sites(graph, &roots, options, &path_filter)
        .into_iter()
        .filter(|site| invocations.contains(&site.invocation))
        .filter(|site| {
            matches!(options.traversal, Traversal::File) || matches_path(&path_filter, &site.file)
        })
        .filter(|site| source_nodes.contains(&site_source_node(site)))
        .filter_map(|site| {
            finding_for_site_with_arguments(
                root,
                application,
                &label,
                options,
                site,
                argument_facts
                    .and_then(|facts| facts.get_ts_facts(&site.file))
                    .and_then(|facts| super::arguments::for_call(facts, site.offset)),
            )
            .map(|finding| (site.offset, finding))
        })
        .collect::<Vec<_>>();
    Ok(super::findings::finish(findings))
}
fn candidate_sites<'a>(
    graph: &'a DepGraph,
    roots: &[NodeId],
    options: &Options,
    path_filter: &super::super::path_filter::RulePathFilter,
) -> Vec<&'a ResolvedCallSite> {
    if matches!(options.traversal, Traversal::File) {
        root_files(roots)
            .into_iter()
            .filter(|file| matches_path(path_filter, file))
            .flat_map(|file| graph.call_sites_in_file(&file))
            .collect()
    } else {
        graph.resolved_call_sites().iter().collect()
    }
}

fn matches_path(filter: &super::super::path_filter::RulePathFilter, file: &Path) -> bool {
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("forbidden_calls.path_filter_checks", 1);
    }
    filter.is_match(file)
}

fn root_files(roots: &[NodeId]) -> Vec<PathBuf> {
    let mut files = roots
        .iter()
        .filter_map(|node| node.as_file().map(Path::to_path_buf))
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    files
}

fn reachable_source_nodes(
    graph: &DepGraph,
    roots: &[NodeId],
    options: &Options,
) -> FxHashSet<NodeId> {
    let mut nodes: FxHashSet<NodeId> = roots.iter().cloned().collect();
    if matches!(options.traversal, Traversal::File) {
        return nodes;
    }
    let traversal = match options.traversal {
        Traversal::Direct => CallTraversal::Direct,
        Traversal::Transitive => CallTraversal::Transitive,
        Traversal::File => unreachable!("file traversal returns the expanded roots"),
    };
    let cap = if traversal == CallTraversal::Direct {
        1
    } else {
        options.max_depth.unwrap_or(usize::MAX)
    };
    for trace in graph.call_traces(roots, traversal, options.max_depth) {
        if trace.nodes.len().saturating_sub(1) < cap {
            nodes.insert(trace.target);
        }
    }
    nodes
}
