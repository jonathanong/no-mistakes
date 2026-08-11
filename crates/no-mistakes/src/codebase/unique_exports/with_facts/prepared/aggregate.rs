use super::{analyze_project_with_optional_prepared_facts_prepared, PreparedResolution};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::config::Config;
use crate::codebase::unique_exports::PreparedUniqueExportFinding;
use anyhow::Result;
use std::path::Path;

/// Analyze aggregate check facts while deferring directive filtering to the
/// check runner's request-wide SourceStore-backed suppression pass.
#[doc(hidden)]
pub fn analyze_project_with_prepared_facts_catalog_and_inferred_and_session_for_check(
    root: &Path,
    config: &Config,
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
    shared: &CheckFactMap,
    inferred_roots: &crate::codebase::config::InferredRoots,
    session: &AnalysisSession,
    defer_suppression: bool,
) -> Result<Vec<PreparedUniqueExportFinding>> {
    analyze_project_with_optional_prepared_facts_prepared(
        root,
        config,
        PreparedResolution {
            catalog: Some(tsconfig_catalog),
            ..Default::default()
        },
        shared,
        Some(inferred_roots),
        session,
        defer_suppression,
    )
}
