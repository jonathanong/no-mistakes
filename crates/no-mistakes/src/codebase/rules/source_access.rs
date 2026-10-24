use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) fn source_store_for_files(files: &[PathBuf]) -> Arc<SourceStore> {
    Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(files))))
}

pub(crate) fn read_source(sources: &SourceStore, path: &Path) -> Option<Arc<str>> {
    sources.read_path(path).ok()
}

/// Required rule inputs must retain their read failure rather than become absent facts.
pub(crate) fn checked_source(
    sources: &SourceStore,
    path: &Path,
    root: &Path,
    rule: &str,
) -> Result<Arc<str>, Box<super::RuleFinding>> {
    sources
        .read_path(path)
        .map_err(|error| Box::new(read_failure(root, path, rule, &error)))
}

pub(crate) fn unanalyzable_source(
    root: &Path,
    path: &Path,
    rule: &str,
    reason: impl std::fmt::Display,
) -> super::RuleFinding {
    input_failure(root, path, rule, reason, "repair source syntax")
}

fn input_failure(
    root: &Path,
    path: &Path,
    rule: &str,
    reason: impl std::fmt::Display,
    action: &str,
) -> super::RuleFinding {
    super::RuleFinding {
        rule: rule.to_string(),
        file: crate::codebase::ts_source::relative_slash_path(root, path),
        line: 1,
        message: format!(
            "cannot verify {rule}: {reason}; {action} so this check can verify the file"
        ),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}

pub(crate) fn read_failure(
    root: &Path,
    path: &Path,
    rule: &str,
    error: &std::io::Error,
) -> super::RuleFinding {
    let mut finding = input_failure(
        root,
        path,
        rule,
        format!("cannot read source: {error}"),
        "restore read access and save valid UTF-8 source",
    );
    if let Some(error) = invalid_utf8(error) {
        finding.line = error.as_bytes()[..error.utf8_error().valid_up_to()]
            .iter()
            .filter(|&&byte| byte == b'\n')
            .count()
            + 1;
    }
    finding
}

pub(crate) fn invalid_utf8(error: &std::io::Error) -> Option<&std::string::FromUtf8Error> {
    error
        .get_ref()
        .and_then(|error| error.downcast_ref::<std::string::FromUtf8Error>())
}

pub(crate) fn suppress_invalid_utf8(
    findings: &mut Vec<super::RuleFinding>,
    error: &std::io::Error,
) {
    if let Some(error) = invalid_utf8(error) {
        super::suppress_rule_findings_with_source(
            findings,
            &String::from_utf8_lossy(error.as_bytes()),
        );
    }
}
