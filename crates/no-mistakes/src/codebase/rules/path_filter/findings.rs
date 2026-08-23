use super::{NoMistakesConfig, Result, RulePathFilter};
use std::path::Path;

pub(crate) fn filter_findings(
    root: &Path,
    config: &NoMistakesConfig,
    rule_id: &str,
    findings: Vec<super::super::RuleFinding>,
) -> Result<Vec<super::super::RuleFinding>> {
    let mut filtered = Vec::new();
    for rule in config.rule_applications(rule_id) {
        let filter = RulePathFilter::new(root, config, rule)?;
        filtered.extend(
            findings
                .iter()
                .filter(|finding| filter.is_match(&root.join(&finding.file)))
                .cloned(),
        );
    }
    super::super::sort_findings(&mut filtered);
    Ok(filtered)
}
