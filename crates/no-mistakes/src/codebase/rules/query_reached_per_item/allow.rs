use super::{config::Options, CheckFactMap, TsFactLookup};
use crate::codebase::rules::{
    allow_audit::{AllowAudit, CheckedScope},
    path_filter::RulePathFilter,
    RuleFinding,
};
use crate::codebase::ts_source::SourceStore;
use std::path::Path;

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
    let checked = facts
        .files()
        .iter()
        .filter(|file| {
            filter.is_match(file)
                && facts.get_ts_facts(file).is_some_and(|facts| {
                    facts.parse_error.is_none() && facts.operational_error.is_none()
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    let scope = CheckedScope::new(root, sources, &checked);
    audit.finish(
        |index| scope.exact(&options.allow[index].file),
        |index| {
            let entry = &options.allow[index];
            let mut selector = entry.file.clone();
            if let Some(line) = entry.line {
                selector.push_str(&format!(":{line}"));
            }
            if let Some(callee) = &entry.callee {
                selector.push_str(&format!("#{callee}"));
            }
            selector
        },
    )
}
