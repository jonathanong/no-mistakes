use crate::queue::extract::FileFacts;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn queue_project_facts_from_ts(
    ts_facts: crate::codebase::ts_source::facts::TsFactMap,
    filter: Option<&globset::GlobSet>,
    root: &Path,
) -> HashMap<PathBuf, FileFacts> {
    ts_facts
        .into_iter()
        .filter_map(|(path, mut facts)| {
            if excluded_by_filter(&path, filter, root) {
                return None;
            }
            facts.queue_project.take().map(|queue| (path, queue))
        })
        .collect()
}

pub(super) fn queue_project_facts_from_shared(
    shared: &crate::codebase::check_facts::CheckFactMap,
    filter: Option<&globset::GlobSet>,
    root: &Path,
) -> HashMap<PathBuf, FileFacts> {
    shared
        .ts
        .iter()
        .filter_map(|(path, facts)| {
            if excluded_by_filter(path, filter, root) {
                return None;
            }
            facts
                .ts
                .queue_project
                .clone()
                .map(|queue| (path.clone(), queue))
        })
        .collect()
}

fn excluded_by_filter(path: &Path, filter: Option<&globset::GlobSet>, root: &Path) -> bool {
    filter.is_some_and(|filter| !filter.is_match(path.strip_prefix(root).unwrap_or(path)))
}

#[cfg(test)]
mod tests;
