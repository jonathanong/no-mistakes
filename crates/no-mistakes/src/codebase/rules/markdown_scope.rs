use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod lexical;
use lexical::lexical_normalized_slash_path;
pub(crate) use lexical::lexical_relative_slash_path;

pub(crate) fn markdown_files(files: &[PathBuf]) -> Vec<PathBuf> {
    document_files_with_extensions(files, &["md"])
}

pub(crate) fn mermaid_document_files(files: &[PathBuf]) -> Vec<PathBuf> {
    document_files_with_extensions(files, &["md", "markdown", "mdx"])
}

fn document_files_with_extensions(files: &[PathBuf], extensions: &[&str]) -> Vec<PathBuf> {
    let mut markdown = files
        .iter()
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extensions.contains(&extension))
        })
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<Vec<_>>();
    markdown.sort();
    markdown.dedup();
    markdown
}

pub(crate) fn scope_roots(root: &Path, config: &NoMistakesConfig, rule: &RuleDef) -> Vec<PathBuf> {
    let mut roots = super::target_roots(root, config, rule)
        .into_iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
        .collect::<Vec<_>>();
    // A nested project owns its files even when the rule also targets an
    // enclosing repository. Keep the order deterministic for callers that
    // choose the first matching root.
    roots.sort_by(|left, right| {
        right
            .components()
            .count()
            .cmp(&left.components().count())
            .then_with(|| left.cmp(right))
    });
    roots.dedup();
    roots
}

pub(crate) fn scope_root_for_path<'a>(roots: &'a [PathBuf], path: &Path) -> Option<&'a PathBuf> {
    roots.iter().find(|root| path.starts_with(root))
}

/// Assign every Markdown file to its most-specific configured scope exactly
/// once. Graph rules use this partition so overlapping repository and project
/// scopes cannot contribute roots or edges to one another.
pub(crate) fn partition_markdown_by_scope(
    scope_roots: &[PathBuf],
    markdown: &[PathBuf],
) -> std::collections::BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut markdown_by_scope = std::collections::BTreeMap::new();
    for path in markdown {
        let Some(scope_root) = scope_root_for_path(scope_roots, path) else {
            continue;
        };
        markdown_by_scope
            .entry(scope_root.clone())
            .or_insert_with(Vec::new)
            .push(path.clone());
    }
    markdown_by_scope
}

/// Stable rule findings are lexical paths from the request root, including
/// `../` for external projects. Joining the key back to the request root
/// resolves the source file for standard suppression handling.
pub(crate) fn finding_key(root: &Path, path: &Path) -> String {
    lexical_relative_slash_path(root, path).unwrap_or_else(|| lexical_normalized_slash_path(path))
}

/// Baseline entries are portable within their configured effective project.
/// Nested projects retain request-root-relative keys for compatibility.
pub(crate) fn baseline_key(root: &Path, scope_root: &Path, path: &Path) -> String {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    if path.starts_with(&root) {
        crate::codebase::ts_source::relative_slash_path(&root, path)
    } else {
        crate::codebase::ts_source::relative_slash_path(scope_root, path)
    }
}

/// Resolves a baseline key to the request-relative finding path. A baseline key
/// is request-relative for in-request projects, but project-relative for
/// external projects, so more than one configured project can make it ambiguous.
pub(crate) fn baseline_finding_key(
    root: &Path,
    scope_roots: &[PathBuf],
    baseline_key: &str,
    rule_id: &str,
) -> Result<String> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut candidates = BTreeSet::new();
    for scope_root in scope_roots {
        let path = if scope_root.starts_with(&root) {
            root.join(baseline_key)
        } else {
            scope_root.join(baseline_key)
        };
        let path = crate::codebase::ts_resolver::normalize_path(&path);
        if path.starts_with(scope_root) {
            candidates.insert(finding_key(&root, &path));
        }
    }
    match candidates.len() {
        0 => Ok(baseline_key.to_string()),
        1 => Ok(candidates.into_iter().next().unwrap()),
        _ => anyhow::bail!(
            "{rule_id} has ambiguous baseline key `{baseline_key}` across configured project roots; configure separate rule applications"
        ),
    }
}
