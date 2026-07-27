use super::{NoMistakesConfig, Result, RuleDef, RulePathFilter};
use std::path::{Path, PathBuf};

/// Markdown rules intentionally support configured project roots outside the
/// request root. Other filesystem rules retain their request-root boundary.
pub(crate) fn filter_markdown_rule_files(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    files: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let filter = RulePathFilter::new_with_external_projects(root, config, rule)?;
    Ok(files
        .iter()
        .filter(|path| filter.is_match(path))
        .cloned()
        .collect())
}
