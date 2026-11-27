use super::super::diagnostics::QueryDiagnostics;
use super::super::shared::rel_str;
use super::CallSite;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{CallRoot, DepGraph};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(crate) struct PreparedCallSitesProjection<'a> {
    pub(crate) graph: &'a DepGraph,
    pub(crate) facts: &'a crate::codebase::ts_source::facts::TsFactMap,
    pub(crate) files: &'a [PathBuf],
    pub(crate) diagnostics: &'a std::sync::Arc<QueryDiagnostics>,
}

pub(super) fn project_sites(
    root: &Path,
    file: &Path,
    export_name: &str,
    projection: &PreparedCallSitesProjection<'_>,
) -> Vec<CallSite> {
    let targets = projection
        .graph
        .expand_call_roots(&[CallRoot::Function {
            file: file.to_path_buf(),
            symbol: export_name.to_string(),
        }])
        .into_iter()
        .collect();
    let mut sites: Vec<_> = projection
        .files
        .par_iter()
        .flat_map(|path| {
            sites_for_file(
                path,
                &targets,
                root,
                projection.graph,
                projection.facts.get(path),
            )
        })
        .collect();
    sites.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
    sites
}

fn sites_for_file(
    path: &Path,
    targets: &crate::fx::FxHashSet<crate::codebase::dependencies::NodeId>,
    root: &Path,
    graph: &DepGraph,
    facts: Option<&crate::codebase::ts_source::facts::TsFileFacts>,
) -> Vec<CallSite> {
    let Some(facts) = facts.filter(|facts| facts.parse_error.is_none()) else {
        return Vec::new();
    };
    let offsets: crate::fx::FxHashSet<_> = graph
        .call_sites_in_file(path)
        .iter()
        .filter(|site| site.invocation == InvocationKind::Call)
        .filter(|site| site.target_nodes().any(|node| targets.contains(node)))
        .map(|site| site.offset)
        .collect();
    facts
        .call_sites
        .iter()
        .filter(|raw| offsets.contains(&raw.offset))
        .map(|raw| CallSite {
            file: rel_str(path, root),
            line: raw.line,
            caller: raw.caller.clone(),
            arg_count: raw.arg_count,
            has_spread: raw.has_spread,
            args: raw.args.clone(),
            reference_only: false,
        })
        .collect()
}
