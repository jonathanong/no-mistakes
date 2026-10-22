use super::super::RULE_ID;
use crate::codebase::check_facts::PathMembership;
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

pub(super) fn source_files<'a>(
    config: &NoMistakesConfig,
    files: &'a [PathBuf],
) -> Option<PathMembership<'a>> {
    super::super::source_only_eligible(config).then(|| PathMembership::new(files))
}

pub(crate) fn graph_plan(
    config: &NoMistakesConfig,
) -> Option<crate::codebase::dependencies::graph::GraphBuildPlan> {
    config
        .rule_configured(RULE_ID)
        .then(|| crate::codebase::dependencies::graph::GraphBuildPlan {
            calls: !super::super::source_only_eligible(config),
            ..Default::default()
        })
}
