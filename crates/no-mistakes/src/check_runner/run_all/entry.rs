use super::run_all_with_session_options;
use no_mistakes::config::v2::Effort;

#[derive(Default)]
pub(crate) struct AnalysisOptions {
    pub(crate) effort: Option<Effort>,
    pub(crate) include_sql_verdicts: bool,
}
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
    run_all_with_options(
        root,
        config_path,
        tsconfig_path,
        include_suppressed,
        include_runner_config_deadlines,
        AnalysisOptions {
            effort: None,
            include_sql_verdicts,
        },
    )
}

pub(crate) fn run_all_with_options(
    root: PathBuf,
    config_path: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
    include_suppressed: bool,
    include_runner_config_deadlines: bool,
    options: AnalysisOptions,
) -> Result<CheckResults> {
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::new(
        no_mistakes::diagnostics::current(),
    );
    run_all_with_session_options(
        &session,
        root,
        config_path,
        tsconfig_path,
        include_suppressed,
        include_runner_config_deadlines,
        options,
    )
}
