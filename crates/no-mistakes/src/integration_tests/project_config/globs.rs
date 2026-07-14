use crate::codebase::glob_normalize;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

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
