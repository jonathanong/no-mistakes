use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(crate) fn markdown_files(files: &[PathBuf]) -> Vec<PathBuf> {
    let mut markdown = files
        .iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
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
    roots.sort_by_key(|path| path.components().count());
    roots.dedup();
    roots
}

pub(crate) fn scope_root_for_path<'a>(roots: &'a [PathBuf], path: &Path) -> Option<&'a PathBuf> {
    roots.iter().find(|root| path.starts_with(root))
}

/// Stable rule findings are lexical paths from the request root, including
/// `../` for external projects. Joining the key back to the request root
/// resolves the source file for standard suppression handling.
pub(crate) fn finding_key(root: &Path, path: &Path) -> String {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let path = crate::codebase::ts_resolver::normalize_path(path);
    let mut root_components = root.components().peekable();
    let mut path_components = path.components().peekable();
    while root_components.peek() == path_components.peek() {
        root_components.next();
        path_components.next();
    }
    let mut relative = PathBuf::new();
    relative.extend(root_components.map(|_| ".."));
    relative.extend(path_components.map(|component| component.as_os_str()));
    relative.to_string_lossy().replace('\\', "/")
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
