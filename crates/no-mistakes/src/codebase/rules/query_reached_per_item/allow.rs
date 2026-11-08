use super::{
    config::{Allowance, Options},
    CheckFactMap, ResolvedCallSite, TsFactLookup,
};
use crate::codebase::rules::{
    allow_audit::{AllowAudit, CheckedScope},
    path_filter::RulePathFilter,
    RuleFinding,
};
use crate::codebase::ts_source::SourceStore;
use std::path::Path;

pub(super) fn matches(allow: &Allowance, file: &str, site: &ResolvedCallSite, line: u32) -> bool {
    allow.file == file
        && allow
            .callee
            .as_ref()
            .is_none_or(|callee| callee == &site.source_callee)
        && allow.line.is_none_or(|allowed| allowed == line)
        && allow
            .function
            .as_deref()
            .is_none_or(|name| site.caller.as_deref() == Some(name))
}

pub(super) fn finish(
    root: &Path,
    facts: &CheckFactMap,
    sources: Option<&SourceStore>,
    filter: &RulePathFilter,
    options: &Options,
    audit: &AllowAudit,
) -> Vec<RuleFinding> {
    if facts.graph_file_universe().iter().any(|file| {
        crate::codebase::dependencies::extract::is_indexable(file)
            && facts.get_ts_facts(file).is_none_or(|facts| {
                facts.parse_error.is_some() || facts.operational_error.is_some()
            })
    }) {
        return Vec::new();
    }
    let graph_files =
        crate::codebase::check_facts::PathMembership::new(facts.graph_file_universe());
    let checked = facts
        .files()
        .iter()
        .filter(|file| {
            graph_files.contains(file)
                && filter.is_match(file)
                && facts.get_ts_facts(file).is_some_and(|facts| {
                    facts.parse_error.is_none() && facts.operational_error.is_none()
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    let scope = CheckedScope::new(root, sources, &checked);
    let complete = audit.audits_missing_files()
        && sources.is_some_and(|sources| {
            sources
                .inventory()
                .paths()
                .iter()
                .filter(|file| {
                    crate::codebase::dependencies::extract::is_indexable(file)
                        && filter.is_match(file)
                })
                .all(|file| {
                    scope.exact(&crate::codebase::ts_source::relative_slash_path(root, file))
                })
        });
    audit.finish_files(
        |index| scope.exact(&options.allow[index].file),
        |index| {
            if complete {
                options.allow[index].file.clone()
            } else {
                String::new()
            }
        },
        |index| {
            let entry = &options.allow[index];
            let mut selector = entry.file.clone();
            if let Some(line) = entry.line {
                selector.push_str(&format!(":{line}"));
            }
            if let Some(callee) = &entry.callee {
                selector.push_str(&format!("#{callee}"));
            }
            if let Some(function) = &entry.function {
                selector.push_str(&format!("@{function}"));
            }
            selector
        },
    )
}

pub(super) fn filter(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
    sources: Option<&SourceStore>,
) -> anyhow::Result<RulePathFilter> {
    let mut inferred = sources
        .filter(|sources| sources.inventory_is_complete())
        .map(|sources| {
            sources.inferred_roots_for_projects(
                root,
                rule.projects
                    .iter()
                    .filter_map(|name| config.projects.get(name)),
            )
        })
        .unwrap_or_default();
    RulePathFilter::new_with_inferred(root, config, rule, &mut inferred)
}
