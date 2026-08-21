use crate::codebase::glob_normalize;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) fn prefix_globs(root: &Path, base: &Path, patterns: &[String]) -> Vec<String> {
    let rel = relative_slash_path(root, base);
    if rel.is_empty() || rel == "." {
        return patterns.to_vec();
    }
    patterns
        .iter()
        .map(|pattern| format!("{}/{pattern}", glob_escape_literal(&rel)))
        .collect()
}

pub(crate) fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(&glob_normalize::normalize(pattern))?);
    }
    Ok(builder.build()?)
}

fn glob_escape_literal(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| {
            if matches!(ch, '*' | '?' | '[' | ']' | '{' | '}' | '\\') {
                vec!['\\', ch]
            } else {
                vec![ch]
            }
        })
        .collect()
}

pub(super) fn expand_explicit_config_values(
    root: &Path,
    patterns: &[String],
    visible_files: &HashSet<PathBuf>,
) -> Vec<String> {
    let mut values = Vec::new();
    let mut seen = HashSet::new();
    for pattern in patterns {
        if is_glob(pattern) {
            if let Ok(glob) = Glob::new(pattern) {
                let matcher = glob.compile_matcher();
                let mut matches = Vec::new();
                for file in visible_files {
                    let rel = relative_slash_path(root, file);
                    if matcher.is_match(&rel) {
                        matches.push(rel);
                    }
                }
                matches.sort();
                push_unique(&mut values, &mut seen, matches);
            }
        } else {
            push_unique(&mut values, &mut seen, [pattern.clone()]);
        }
    }
    values
}

fn push_unique(
    values: &mut Vec<String>,
    seen: &mut HashSet<String>,
    items: impl IntoIterator<Item = String>,
) {
    for item in items {
        if seen.insert(item.clone()) {
            values.push(item);
        }
    }
}

fn is_glob(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?') || pattern.contains('[')
}

#[cfg(test)]
mod tests;
