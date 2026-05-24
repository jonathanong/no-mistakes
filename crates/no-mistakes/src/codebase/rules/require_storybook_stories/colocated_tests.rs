use super::types::Component;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_resolver::normalize_path;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub(super) fn covered_components(
    shared: &CheckFactMap,
    components: &[Component],
) -> BTreeSet<String> {
    let files: HashSet<PathBuf> = shared
        .files()
        .iter()
        .map(|path| normalize_path(path))
        .collect();
    components
        .iter()
        .filter(|component| has_colocated_test(&files, &component.file))
        .map(|component| component.key.clone())
        .collect()
}

fn has_colocated_test(files: &HashSet<PathBuf>, component_file: &Path) -> bool {
    let Some(stem) = component_file.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };
    let Some(parent) = component_file.parent() else {
        return false;
    };
    ["test.tsx", "mock.test.tsx", "test.ts", "mock.test.ts"]
        .into_iter()
        .map(|suffix| parent.join(format!("{stem}.{suffix}")))
        .any(|test_file| files.contains(&normalize_path(&test_file)))
}
