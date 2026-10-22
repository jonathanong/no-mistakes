//! Exact file traversal can use source call facts without target resolution.
use super::config::{Options, Root};
use super::{arguments, findings, roots, RULE_ID};
use crate::codebase::dependencies::extract::FunctionCall;
use crate::codebase::dependencies::graph::{NodeId, TsFactLookup};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::config::v2::schema::RuleDef;
use crate::fx::{FxHashMap, FxHashSet};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub(super) fn check(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    application: &RuleDef,
    index: usize,
    options: &Options,
    graph_files: &crate::codebase::check_facts::PathMembership<'_>,
    facts: &dyn TsFactLookup,
) -> Result<Vec<RuleFinding>> {
    let required = crate::codebase::ts_source::facts::TsFactPlan {
        function_calls: true,
        ..Default::default()
    };
    if !facts.covers_ts_fact_plan(required) {
        bail!("{RULE_ID}: prepared facts are missing source calls required by file traversal");
    }
    let files = file_roots(root, options, graph_files, facts)?;
    let filter =
        crate::codebase::rules::path_filter::RulePathFilter::new(root, config, application)?;
    let allowed = super::selection::invocations::allowed_invocations(&options.invocations);
    let label = application.name.as_deref().map_or_else(
        || format!("application #{index}"),
        |name| format!("{name}, application #{index}"),
    );
    let mut findings = Vec::new();
    for (path, roots) in files {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("forbidden_calls.path_filter_checks", 1);
        }
        if !filter.is_match(&path) {
            continue;
        }
        let file = facts
            .get_ts_facts(&path)
            .expect("validated source file facts");
        let mut calls = file.function_calls.iter().collect::<Vec<_>>();
        calls.sort_by(|a, b| {
            (&a.line, &a.offset, &a.caller, &a.callee)
                .cmp(&(&b.line, &b.offset, &b.caller, &b.callee))
        });
        for call in calls {
            if !crate::codebase::dependencies::graph::traversable_source_call(file, call)
                || !allowed.contains(&call.invocation)
                || !roots.contains(&source_node(&path, call))
            {
                continue;
            }
            let arguments = options
                .targets
                .iter()
                .any(|target| target.arguments().is_some())
                .then(|| arguments::for_call(file, call.offset))
                .flatten();
            if let Some(finding) = findings::finding_for_source_call(
                root,
                application,
                &label,
                options,
                &path,
                call,
                arguments,
            ) {
                findings.push((call.offset, finding));
            }
        }
    }
    Ok(findings::finish(findings))
}

pub(super) fn file_roots(
    root: &Path,
    options: &Options,
    graph_files: &crate::codebase::check_facts::PathMembership<'_>,
    facts: &dyn TsFactLookup,
) -> Result<Vec<(PathBuf, FxHashSet<NodeId>)>> {
    let patterns = options
        .exclude_roots
        .as_ref()
        .map(|v| v.values())
        .unwrap_or_default();
    let excludes = crate::codebase::rules::path_filter::GlobMatcher::new(
        &patterns,
        "forbidden-calls excludeRoots",
    )?;
    let mut files = Vec::new();
    for selector in &options.roots {
        let Root::File(selector) = selector else {
            unreachable!("source-only eligibility")
        };
        let path = roots::require_file(root, &selector.file, "file root")?;
        if roots::excludes_root(root, &path, &excludes) {
            continue;
        }
        if graph_files.contains(path.as_path())
            && crate::codebase::dependencies::extract::is_indexable(&path)
        {
            let file = facts.get_ts_facts(&path).ok_or_else(|| {
                anyhow::anyhow!(
                    "{RULE_ID}: prepared facts are missing for configured root `{}`",
                    selector.file
                )
            })?;
            if let Some(error) = file.parse_error.as_ref() {
                bail!(
                    "{RULE_ID}: configured root `{}` failed to parse: {error}",
                    crate::codebase::ts_source::relative_slash_path(root, &path)
                );
            }
        }
        files.push(path);
    }
    if files.is_empty() {
        bail!("{RULE_ID}: excludeRoots excluded all configured roots");
    }
    // Match roots::expand: validate every path first, then fail on the first
    // root with no callable source, before sorting/deduplicating the union.
    let mut validated = Vec::with_capacity(files.len());
    for path in files {
        if !crate::codebase::dependencies::extract::is_indexable(&path) {
            bail!("{RULE_ID}: a configured root resolves to no callable source");
        }
        let Some(file) = facts
            .get_ts_facts(&path)
            .filter(|_| graph_files.contains(path.as_path()))
        else {
            bail!("{RULE_ID}: a configured root resolves to no callable source");
        };
        let roots = callable_roots(&path, file);
        if roots.is_empty() {
            bail!("{RULE_ID}: a configured root resolves to no callable source");
        }
        validated.push((path, roots));
    }
    validated.sort_by(|left, right| left.0.cmp(&right.0));
    validated.dedup_by(|left, right| left.0 == right.0);
    Ok(validated)
}

fn callable_roots(path: &Path, file: &TsFileFacts) -> FxHashSet<NodeId> {
    let mut by_scope: FxHashMap<&str, Vec<_>> = FxHashMap::default();
    for (id, scope) in &file.callable_scope_ids {
        by_scope.entry(scope).or_default().push(*id);
    }
    let mut nodes = FxHashSet::default();
    for scope in &file.callable_scopes {
        match by_scope.get(scope.as_str()) {
            Some(ids) if !ids.is_empty() => {
                for id in ids {
                    nodes.insert(NodeId::callable(path, scope.clone(), *id));
                }
            }
            _ => {
                nodes.insert(NodeId::symbol(path, scope.as_str()));
            }
        }
    }
    if !file.unknown_calls.is_empty()
        || file
            .function_calls
            .iter()
            .any(|call| crate::codebase::dependencies::graph::traversable_source_call(file, call))
    {
        nodes.insert(NodeId::file(path));
    }
    nodes
}

fn source_node(path: &Path, call: &FunctionCall) -> NodeId {
    call.caller.as_deref().map_or_else(
        || NodeId::file(path),
        |scope| {
            call.caller_id.map_or_else(
                || NodeId::symbol(path, scope),
                |id| NodeId::callable(path, scope.to_string(), id),
            )
        },
    )
}
