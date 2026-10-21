use super::run_all_with_session;
use crate::check_runner::CheckResults;
use anyhow::Result;
use std::path::PathBuf;

pub(crate) fn run_all_with_evidence(
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
    include_suppressed: bool,
    include_runner_config_deadlines: bool,
) -> Result<CheckResults> {
    run_all_with_sql_verdicts(
        root,
        config_path,
        tsconfig_path,
        include_suppressed,
        include_runner_config_deadlines,
        false,
    )
}

pub(crate) fn run_all_with_sql_verdicts(
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
    include_suppressed: bool,
    include_runner_config_deadlines: bool,
    include_sql_verdicts: bool,
) -> Result<CheckResults> {
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::new(
        no_mistakes::diagnostics::current(),
    );
    run_all_with_session(
        &session,
        root,
        config_path,
        tsconfig_path,
        include_suppressed,
        include_runner_config_deadlines,
        include_sql_verdicts,
    )
}
