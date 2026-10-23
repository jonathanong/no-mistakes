use super::*;

pub(super) fn findings_for_file(
    root: &Path,
    path: &Path,
    source: &str,
    cache_facts: &[NextjsCachingFinding],
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    if !defer_suppression && has_disable_file_comment(source, RULE_ID) {
        return Vec::new();
    }
    let file = relative_slash_path(root, path);
    cache_facts
        .iter()
        .filter(|finding| {
            defer_suppression || !has_disable_comment(source, finding.line as u32, RULE_ID)
        })
        .map(|finding| project_finding(&file, finding))
        .collect()
}

pub(super) fn project_finding(file: &str, finding: &NextjsCachingFinding) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: finding.line,
        message: finding.message.clone(),
        import: None,
        target: None,
    }
}
