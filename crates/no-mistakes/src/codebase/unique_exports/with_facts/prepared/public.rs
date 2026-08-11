use super::{analyze_project_with_optional_prepared_facts_prepared, PreparedResolution};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::config::Config;
use crate::codebase::unique_exports::UniqueExportFinding;
use anyhow::Result;
use std::path::Path;

pub(super) fn analyze_project_with_optional_prepared_facts(
    root: &Path,
    config: &Config,
    resolution: PreparedResolution<'_>,
    shared: &CheckFactMap,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    session: &AnalysisSession,
    defer_suppression: bool,
) -> Result<Vec<UniqueExportFinding>> {
    Ok(analyze_project_with_optional_prepared_facts_prepared(
        root,
        config,
        resolution,
        shared,
        inferred_roots,
        session,
        defer_suppression,
    )?
    .into_iter()
    .map(|prepared| prepared.finding)
    .collect())
}
