use super::RuleFinding;
use crate::codebase::comment_only::{classify_content, ContentKind};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
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
    audit: &super::allow_audit::AllowAudit,
) -> Result<Vec<RuleFinding>> {
    let extensions = effective_extensions(opts);
    let exempt: std::collections::HashSet<_> = opts
        .intentionally_empty
        .iter()
        .map(String::as_str)
        .collect();
    let checked = dashmap::DashSet::new();
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| {
            let rel = relative_slash_path(root, path);
            let has_ext = extensions.iter().any(|ext| rel.ends_with(*ext));
            if !has_ext {
                return Vec::new();
            }
            let content = match super::source_access::checked_source(sources, path, root, RULE_ID) {
                Ok(content) => content,
                Err(finding) => {
                    return if exempt.contains(rel.as_str()) {
                        Vec::new()
                    } else {
                        vec![*finding]
                    };
                }
            };
            if exempt.contains(rel.as_str()) {
                checked.insert(rel);
            }
            check_source(path, root, &content)
                .into_iter()
                .filter(|finding| {
                    !exempt.contains(finding.file.as_str())
                        || !audit.usage.suppress(
                            opts.intentionally_empty
                                .iter()
                                .map(|entry| entry == &finding.file),
                        )
                })
                .collect()
        })
        .collect();
    findings.extend(audit.finish_files(
        |index| checked.contains(&opts.intentionally_empty[index]),
        |index| opts.intentionally_empty[index].clone(),
        |index| opts.intentionally_empty[index].clone(),
    ));
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(findings)
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

#[cfg(test)]
mod allow_tests;
