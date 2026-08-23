use super::finding;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeSet;

mod collect;

pub(super) use collect::frameworks;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    File,
    Directory,
    Glob,
}

pub(super) struct Ref {
    pub(super) field: String,
    pub(super) kind: Kind,
    pub(super) value: String,
}

pub(super) fn lint(
    config: &NoMistakesConfig,
    tracked: &BTreeSet<String>,
    config_file: &str,
) -> Result<Vec<RuleFinding>> {
    let mut refs = Vec::new();
    collect::collect(config, &mut refs);
    Ok(refs
        .into_iter()
        .filter_map(|item| missing(item, tracked, config_file))
        .collect())
}

fn missing(item: Ref, tracked: &BTreeSet<String>, config_file: &str) -> Option<RuleFinding> {
    let present = match item.kind {
        Kind::Glob => glob_matches(&item.value, tracked),
        Kind::File => tracked.contains(item.value.trim_start_matches("./")),
        Kind::Directory => directory_present(&item.value, tracked),
    };
    (!present).then(|| {
        finding(
            config_file,
            format!(
                "{}: missing {} `{}`",
                item.field,
                kind_label(item.kind),
                item.value
            ),
        )
    })
}

fn kind_label(kind: Kind) -> &'static str {
    match kind {
        Kind::File => "file",
        Kind::Directory => "directory",
        Kind::Glob => "path",
    }
}

fn directory_present(value: &str, tracked: &BTreeSet<String>) -> bool {
    let prefix = value.trim_start_matches("./").trim_end_matches('/');
    if prefix.is_empty() || prefix == "." {
        return true;
    }
    tracked.contains(prefix)
        || tracked
            .iter()
            .any(|rel| rel == prefix || rel.starts_with(&format!("{prefix}/")))
}

fn glob_matches(pattern: &str, tracked: &BTreeSet<String>) -> bool {
    crate::codebase::rules::path_filter::GlobMatcher::new(
        &[pattern.to_string()],
        "no-mistakes-config path",
    )
    .map(|matcher| tracked.iter().any(|rel| matcher.is_match(rel)))
    .unwrap_or(false)
}
