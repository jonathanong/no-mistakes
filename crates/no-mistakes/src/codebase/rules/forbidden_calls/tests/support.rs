use super::Options;
use crate::codebase::rules::call_policy::roots;
use std::path::{Path, PathBuf};

pub(crate) fn expand_roots(
    root: &Path,
    options: &Options,
    graph: &crate::codebase::dependencies::graph::DepGraph,
    files: &[PathBuf],
) -> anyhow::Result<Vec<crate::codebase::dependencies::graph::NodeId>> {
    roots::expand(roots::ExpandRequest {
        rule: super::super::RULE_ID,
        allow_sourceless: false,
        root,
        roots: &options.roots,
        exclude_roots: options.exclude_roots.as_ref(),
        graph,
        vitest: None,
        playwright: None,
        graph_files: files,
        target_roots: &[],
    })
}

pub(in super::super) fn finding_for_site(
    root: &Path,
    application: &crate::config::v2::schema::RuleDef,
    label: &str,
    options: &Options,
    site: &crate::codebase::dependencies::graph::ResolvedCallSite,
) -> Option<no_mistakes_rule_kernel::finding::RuleFinding> {
    super::super::findings::finding_for_site_with_arguments(
        root,
        application,
        label,
        options,
        site,
        None,
        false,
    )
}
