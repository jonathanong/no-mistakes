use crate::finding::RuleFinding;
use crate::paths::relative_slash_path;
use std::path::Path;
use std::sync::Arc;

pub trait RuleSources {
    fn read_path(&self, path: &Path) -> Result<Arc<str>, Arc<std::io::Error>>;
    fn read_source_path(&self, path: &Path) -> Result<Arc<str>, Arc<std::io::Error>>;
}

impl<S: RuleSources> RuleSources for Arc<S> {
    fn read_path(&self, path: &Path) -> Result<Arc<str>, Arc<std::io::Error>> {
        self.as_ref().read_path(path)
    }

    fn read_source_path(&self, path: &Path) -> Result<Arc<str>, Arc<std::io::Error>> {
        self.as_ref().read_source_path(path)
    }
}

pub fn read_source(sources: &impl RuleSources, path: &Path) -> Option<Arc<str>> {
    sources.read_path(path).ok()
}

/// Invalid UTF-8 is decoded lossily. Other read errors stay absent so a missing path is not a finding.
pub fn read_language_source(sources: &impl RuleSources, path: &Path) -> Option<Arc<str>> {
    sources.read_source_path(path).ok()
}

/// Required rule inputs must retain their read failure rather than become absent facts.
pub fn checked_source(
    sources: &impl RuleSources,
    path: &Path,
    root: &Path,
    rule: &str,
) -> Result<Arc<str>, Box<RuleFinding>> {
    sources
        .read_path(path)
        .map_err(|error| Box::new(read_failure(root, path, rule, &error)))
}

pub fn unanalyzable_source(
    root: &Path,
    path: &Path,
    rule: &str,
    reason: impl std::fmt::Display,
) -> RuleFinding {
    input_failure(root, path, rule, reason, "repair source syntax")
}

fn input_failure(
    root: &Path,
    path: &Path,
    rule: &str,
    reason: impl std::fmt::Display,
    action: &str,
) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: rule.to_string(),
        file: relative_slash_path(root, path),
        line: 1,
        message: format!(
            "cannot verify {rule}: {reason}; {action} so this check can verify the file"
        ),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}

pub fn read_failure(root: &Path, path: &Path, rule: &str, error: &std::io::Error) -> RuleFinding {
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

pub fn invalid_utf8(error: &std::io::Error) -> Option<&std::string::FromUtf8Error> {
    error
        .get_ref()
        .and_then(|error| error.downcast_ref::<std::string::FromUtf8Error>())
}
