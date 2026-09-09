use super::super::RuleFinding;
use super::config::{self, Invocation, Options, Traversal};
use super::findings::finding_for_site;
use super::roots;
use super::RULE_ID;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{CallTraversal, DepGraph, NodeId, ResolvedCallSite};
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

pub(crate) fn graph_plan(
    config: &NoMistakesConfig,
) -> Option<crate::codebase::dependencies::graph::GraphBuildPlan> {
    config
        .rule_configured(RULE_ID)
        .then(|| crate::codebase::dependencies::graph::GraphBuildPlan {
            calls: true,
            ..Default::default()
        })
}
pub(crate) fn check_with_graph(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
    catalog: Option<&super::super::PreparedVitestProjectCatalog>,
    graph_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut findings = Vec::new();
    for (index, application) in config.rule_applications(RULE_ID).into_iter().enumerate() {
        let options: Options = application.try_rule_options()?;
        findings.extend(check_application(ApplicationCheck {
            root: &root,
            config,
            application,
            index: index + 1,
            options: &options,
            graph,
            catalog,
            graph_files,
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
    catalog: Option<&'a super::super::PreparedVitestProjectCatalog>,
    graph_files: &'a [PathBuf],
}
fn check_application(input: ApplicationCheck<'_>) -> Result<Vec<RuleFinding>> {
    let ApplicationCheck {
        root,
        config,
        application,
        index,
        options,
        graph,
        catalog,
        graph_files,
    } = input;
    config::validate(options)?;
    let roots = roots::expand(root, options, graph, catalog, graph_files)?;
    let path_filter = super::super::path_filter::RulePathFilter::new(root, config, application)?;
    let source_nodes = reachable_source_nodes(graph, &roots, options);
    let invocations = allowed_invocations(&options.invocations);
    let label = application.name.as_deref().map_or_else(
        || format!("application #{index}"),
        |name| format!("{name}, application #{index}"),
    );
    let findings = graph
        .resolved_call_sites()
        .iter()
        .filter(|site| invocations.contains(&site.invocation))
        .filter(|site| path_filter.is_match(&site.file))
        .filter(|site| source_nodes.contains(&site_source_node(site)))
        .filter_map(|site| {
            finding_for_site(root, application, &label, options, site)
                .map(|finding| (site.offset, finding))
        })
        .collect::<Vec<_>>();
    let mut totals = BTreeMap::new();
    for (_, finding) in &findings {
        *totals.entry(finding.clone()).or_insert(0usize) += 1;
    }
    let mut seen = BTreeMap::new();
    let mut findings = findings
        .into_iter()
        .map(|(_, mut finding)| {
            if totals.get(&finding).copied().unwrap_or_default() > 1 {
                let occurrence = seen.entry(finding.clone()).or_insert(0usize);
                *occurrence += 1;
                finding.message = format!("{} (occurrence {})", finding.message, occurrence);
            }
            finding
        })
        .collect::<Vec<_>>();
    findings.sort_by(|left, right| {
        (&left.file, left.line, &left.message, &left.target).cmp(&(
            &right.file,
            right.line,
            &right.message,
            &right.target,
        ))
    });
    Ok(findings)
}
fn reachable_source_nodes(
    graph: &DepGraph,
    roots: &[NodeId],
    options: &Options,
) -> HashSet<NodeId> {
    let traversal = match options.traversal {
        Traversal::Direct => CallTraversal::Direct,
        Traversal::File => CallTraversal::File,
        Traversal::Transitive => CallTraversal::Transitive,
    };
    let cap = if traversal == CallTraversal::Direct {
        1
    } else {
        options.max_depth.unwrap_or(usize::MAX)
    };
    let mut nodes = roots.iter().cloned().collect::<HashSet<_>>();
    for trace in graph.call_traces(roots, traversal, options.max_depth) {
        if trace.nodes.len().saturating_sub(1) < cap {
            nodes.insert(trace.target);
        }
    }
    nodes
}
fn allowed_invocations(configured: &[Invocation]) -> Vec<InvocationKind> {
    let values = configured
        .iter()
        .map(|kind| match kind {
            Invocation::Call => InvocationKind::Call,
            Invocation::Construct => InvocationKind::Construct,
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        vec![InvocationKind::Call]
    } else {
        values
    }
}
fn site_source_node(site: &ResolvedCallSite) -> NodeId {
    site.caller.as_deref().map_or_else(
        || NodeId::file(&site.file),
        |symbol| {
            site.caller_id.map_or_else(
                || NodeId::symbol(&site.file, symbol),
                |id| NodeId::callable(&site.file, symbol.to_string(), id),
            )
        },
    )
}
