use super::RuleFinding;
use crate::codebase::comment_only::{classify_content, ContentKind};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "no-empty-or-comments-only-files";

const DEFAULT_EXTENSIONS: &[&str] = &[
    ".ts", ".mts", ".cts", ".tsx", ".js", ".jsx", ".sql", ".rs", ".css",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) extensions: Vec<String>,
    pub(crate) intentionally_empty: Vec<String>,
}

mod execution;
pub use execution::check;
#[doc(hidden)]
pub use execution::diagnostics_with_files_and_sources;
super::diagnostic_source_rule_entrypoints!();

fn effective_extensions(opts: &Options) -> Vec<&str> {
    if opts.extensions.is_empty() {
        DEFAULT_EXTENSIONS.to_vec()
    } else {
        opts.extensions.iter().map(String::as_str).collect()
    }
}

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let extensions = effective_extensions(opts);
    let exempt: HashSet<&str> = opts
        .intentionally_empty
        .iter()
        .map(String::as_str)
        .collect();
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| {
            let rel = relative_slash_path(root, path);
            if exempt.contains(rel.as_str()) {
                return Vec::new();
            }
            let has_ext = extensions.iter().any(|ext| rel.ends_with(*ext));
            if !has_ext {
                return Vec::new();
            }
            check_file_with_sources(path, root, sources)
        })
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(findings)
}

fn check_file_with_sources(
    path: &Path,
    root: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let content = match super::source_access::checked_source(sources, path, root, RULE_ID) {
        Ok(source) => source,
        Err(finding) => return vec![*finding],
    };
    check_source(path, root, &content)
}

fn check_source(path: &Path, root: &Path, content: &str) -> Vec<RuleFinding> {
    let file = relative_slash_path(root, path);
    let ext_no_dot = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let message = match classify_content(content, ext_no_dot) {
        ContentKind::HasContent => return Vec::new(),
        ContentKind::Empty => "file is empty",
        ContentKind::CommentsOnly => "file contains only comments — add real content or remove it",
    };
    vec![RuleFinding {
        rule: RULE_ID.to_string(),
        file,
        line: 1,
        message: message.to_string(),
        import: None,
        target: None,
    }]
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_support;
