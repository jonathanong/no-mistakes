use super::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use std::path::{Path, PathBuf};

pub(crate) fn collect_with_precollected_ts(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    graph_files_complete: bool,
    plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
    precollected_ts: crate::codebase::ts_source::facts::TsFactMap,
) -> CheckFactMap {
    let sources =
        super::super::collect::request_sources(&files, &graph_files, &plan, Some(&playwright));
    super::collect_with_precollected_ts_and_sources(
        root,
        (files, graph_files, graph_files_complete),
        plan,
        playwright,
        precollected_ts,
        sources,
    )
}
