use super::RuleFinding;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) fn suppress_rule_findings(root: &Path, findings: &mut Vec<RuleFinding>) {
    let Some(root) = std::fs::canonicalize(root).ok() else {
        return;
    };
    let mut sources: HashMap<String, Option<String>> = HashMap::new();
    findings.retain(|finding| {
        let source = sources.entry(finding.file.clone()).or_insert_with(|| {
            source_path_for_finding(&root, &finding.file)
                .and_then(|path| std::fs::read_to_string(path).ok())
        });
        !source
            .as_deref()
            .is_some_and(|source| finding_is_suppressed(source, finding))
    });
}

fn source_path_for_finding(root: &Path, file: &str) -> Option<PathBuf> {
    let path = Path::new(file);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::Prefix(_)
                    | std::path::Component::RootDir
                    | std::path::Component::ParentDir
            )
        })
    {
        return None;
    }
    let candidate = std::fs::canonicalize(root.join(path)).ok()?;
    let metadata = std::fs::metadata(&candidate).ok()?;
    (candidate.starts_with(root) && metadata.is_file()).then_some(candidate)
}

fn finding_is_suppressed(source: &str, finding: &RuleFinding) -> bool {
    let line = finding.line.try_into().ok();
    crate::codebase::ts_source::has_disable_file_comment(source, &finding.rule)
        || line.is_some_and(|line| {
            crate::codebase::ts_source::has_disable_comment(source, line, &finding.rule)
                || crate::codebase::ts_source::has_disable_line_comment(source, line, &finding.rule)
        })
}
