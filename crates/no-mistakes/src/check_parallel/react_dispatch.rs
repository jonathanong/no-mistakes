use crate::check_tasks::{run_react_check, CheckTask};
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::react_traits;

pub(super) struct Inputs<'a> {
    pub(super) root: &'a std::path::Path,
    pub(super) enabled: bool,
    pub(super) facts: &'a CheckFactMap,
    pub(super) prepared: &'a react_traits::PreparedReactCheck,
    pub(super) sources: &'a no_mistakes::codebase::ts_source::SourceStore,
    pub(super) defer_suppression: bool,
}

pub(super) fn run(inputs: Inputs<'_>) -> anyhow::Result<CheckTask<Vec<react_traits::Violation>>> {
    let Inputs {
        root,
        enabled,
        facts,
        prepared,
        sources,
        defer_suppression,
    } = inputs;
    run_react_check(root, enabled, facts, prepared, sources, defer_suppression)
}
