use crate::codebase::ts_source::SourceStore;
use anyhow::Result;
use globset::Glob;
use std::path::{Path, PathBuf};

pub(super) fn entries(path: &Path, sources: &SourceStore) -> Vec<String> {
    sources
        .parse_json_path(path)
        .ok()
        .and_then(|value| {
            value
                .get("workspaces")
                .and_then(|value| value.as_array())
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(|entry| entry.as_str())
                        .map(normalize_entry)
                        .collect()
                })
        })
        .unwrap_or_default()
}

pub(crate) fn normalize_entry(entry: &str) -> String {
    let normalized = entry.replace('\\', "/");
    let mut parts = Vec::new();
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                let can_pop = parts.last().is_some_and(|parent: &&str| {
                    *parent != ".."
                        && !parent
                            .chars()
                            .any(|ch| matches!(ch, '*' | '?' | '[' | ']' | '{' | '}'))
                });
                if can_pop {
                    parts.pop();
                } else {
                    parts.push(part);
                }
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

pub(super) fn relative_from(from: &Path, to: &Path) -> String {
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();
    let shared = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    let mut parts = vec![".."; from.len().saturating_sub(shared)];
    parts.extend(
        to[shared..]
            .iter()
            .filter_map(|component| component.as_os_str().to_str()),
    );
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

pub(super) fn contains_wildcard(entry: &str) -> bool {
    entry.contains('*') || entry.contains('?') || entry.contains('[') || entry.contains('{')
}

pub(super) fn wildcard_targets_dependency<'a>(
    root_dir: &Path,
    entry: &str,
    mut targets: impl Iterator<Item = &'a PathBuf>,
) -> Result<bool> {
    let glob = Glob::new(entry)?.compile_matcher();
    Ok(targets.any(|target| glob.is_match(relative_from(root_dir, target))))
}

pub(super) fn line(path: &Path, sources: &SourceStore) -> usize {
    super::super::read_source(sources, path)
        .and_then(|source| {
            source
                .lines()
                .position(|line| line.contains("\"workspaces\""))
                .map(|index| index + 1)
        })
        .unwrap_or(1)
}
