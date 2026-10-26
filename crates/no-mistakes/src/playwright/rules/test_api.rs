use super::{check_with_facts_outcome, check_with_prepared_facts_outcome, PreparedPlaywrightRules};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

pub(crate) fn check_with_facts(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    Ok(check_with_facts_outcome(root, config_path, config, facts)?.findings)
}

pub(crate) fn check_with_prepared_facts(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
    prepared: &PreparedPlaywrightRules,
) -> Result<Vec<RuleFinding>> {
    Ok(check_with_prepared_facts_outcome(root, config_path, config, facts, prepared)?.findings)
}
