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

/// Return a portable lexical relative path when both paths have compatible
/// roots. Windows path syntax is parsed independently of the host OS so a
/// cross-volume finding never becomes a misleading `../../` traversal.
pub(crate) fn lexical_relative_slash_path(root: &Path, path: &Path) -> Option<String> {
    let root = LexicalPath::parse(root);
    let path = LexicalPath::parse(path);
    root.prefix.compatible_with(&path.prefix).then(|| {
        let common = root
            .components
            .iter()
            .zip(&path.components)
            .take_while(|(left, right)| root.prefix.component_eq(left, right))
            .count();
        std::iter::repeat_n("..".to_string(), root.components.len() - common)
            .chain(path.components[common..].iter().cloned())
            .collect::<Vec<_>>()
            .join("/")
    })
}

fn lexical_normalized_slash_path(path: &Path) -> String {
    LexicalPath::parse(path).render()
}

#[derive(Debug, PartialEq, Eq)]
enum LexicalPrefix {
    Relative,
    Posix,
    Drive(String),
    Unc(String, String),
}

impl LexicalPrefix {
    fn compatible_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Relative, Self::Relative) | (Self::Posix, Self::Posix) => true,
            (Self::Drive(left), Self::Drive(right)) => left.eq_ignore_ascii_case(right),
            (Self::Unc(left_server, left_share), Self::Unc(right_server, right_share)) => {
                left_server.eq_ignore_ascii_case(right_server)
                    && left_share.eq_ignore_ascii_case(right_share)
            }
            _ => false,
        }
    }

    fn component_eq(&self, left: &str, right: &str) -> bool {
        match self {
            Self::Drive(_) | Self::Unc(_, _) => left.eq_ignore_ascii_case(right),
            Self::Relative | Self::Posix => left == right,
        }
    }
}

struct LexicalPath {
    prefix: LexicalPrefix,
    components: Vec<String>,
}

impl LexicalPath {
    fn parse(path: &Path) -> Self {
        let raw = path.to_string_lossy().replace('\\', "/");
        let (prefix, remainder) = if let Some(remainder) = raw.strip_prefix("//") {
            let mut parts = remainder.splitn(3, '/');
            match (parts.next(), parts.next()) {
                (Some(server), Some(share)) if !server.is_empty() && !share.is_empty() => (
                    LexicalPrefix::Unc(server.to_string(), share.to_string()),
                    parts.next().unwrap_or_default(),
                ),
                _ => (LexicalPrefix::Posix, remainder),
            }
        } else if raw.len() >= 3
            && raw.as_bytes()[0].is_ascii_alphabetic()
            && raw.as_bytes()[1] == b':'
            && raw.as_bytes()[2] == b'/'
        {
            (
                LexicalPrefix::Drive((raw.as_bytes()[0] as char).to_string()),
                &raw[3..],
            )
        } else if let Some(remainder) = raw.strip_prefix('/') {
            (LexicalPrefix::Posix, remainder)
        } else {
            (LexicalPrefix::Relative, raw.as_str())
        };
        let mut components = Vec::new();
        for component in remainder.split('/') {
            match component {
                "" | "." => {}
                ".." if components.last().is_some_and(|part| part != "..") => {
                    components.pop();
                }
                ".." if matches!(&prefix, LexicalPrefix::Relative) => {
                    components.push(component.to_string());
                }
                ".." => {}
                component => components.push(component.to_string()),
            }
        }
        Self { prefix, components }
    }

    fn render(&self) -> String {
        let base = match &self.prefix {
            LexicalPrefix::Relative => String::new(),
            LexicalPrefix::Posix => "/".to_string(),
            LexicalPrefix::Drive(drive) => format!("{drive}:/"),
            LexicalPrefix::Unc(server, share) => format!("//{server}/{share}"),
        };
        if self.components.is_empty() {
            return base;
        }
        match &self.prefix {
            LexicalPrefix::Unc(_, _) => format!("{base}/{}", self.components.join("/")),
            LexicalPrefix::Relative => self.components.join("/"),
            LexicalPrefix::Posix | LexicalPrefix::Drive(_) => {
                format!("{base}{}", self.components.join("/"))
            }
        }
    }
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
