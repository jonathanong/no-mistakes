use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

#[doc(hidden)]
pub fn validate_rule_options_for_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<()> {
    use super::super::no_cross_package_relative_imports as relative_imports;

    if config.rule_configured(relative_imports::RULE_ID) {
        relative_imports::validate_options(root, config_path, config, &session.workspace(root))?;
    }
    Ok(())
}
