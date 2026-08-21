use super::{SourceFile, RULE_ID};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path, TS_JS_EXTENSIONS};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[cfg(test)]
pub(super) mod test_support;

pub(super) fn filter_source_files(files: &[PathBuf]) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| TS_JS_EXTENSIONS.contains(&ext))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
pub(super) fn collect_source_files_from_facts(
    root: &Path,
    files: &[PathBuf],
    shared: &crate::codebase::check_facts::CheckFactMap,
    defer_suppression: bool,
) -> Result<Vec<SourceFile>> {
    collect_source_files_from_facts_with_sources(root, files, shared, defer_suppression, None)
}

pub(super) fn collect_source_files_from_facts_with_sources(
    root: &Path,
    files: &[PathBuf],
    shared: &crate::codebase::check_facts::CheckFactMap,
    defer_suppression: bool,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<SourceFile>> {
    let nextjs_projects = NextJsProjectLookup::with_sources(root, files, shared.files(), sources);
    let mut source_files = Vec::new();
    for path in files {
        let Some(facts) = shared.ts.get(path) else {
            anyhow::bail!("missing shared facts for {}", path.display());
        };
        let Some(source) = facts.source.clone() else {
            anyhow::bail!("missing source facts for {}", path.display());
        };
        let disabled = has_disable_file_comment(&source, RULE_ID);
        let symbols = if let Some(error) = &facts.parse_error {
            if !disabled {
                anyhow::bail!("failed to parse {}: {error}", path.display());
            }
            Default::default()
        } else {
            let Some(symbols) = facts.symbols.clone() else {
                anyhow::bail!("missing symbol facts for {}", path.display());
            };
            symbols
        };
        source_files.push(SourceFile {
            path: normalize_path(path),
            rel: relative_slash_path(root, path),
            source: source.to_string(),
            disabled,
            defer_suppression,
            is_nextjs_project: nextjs_projects.contains_file(path),
            symbols,
        });
    }
    Ok(source_files)
}

pub(super) struct NextJsProjectLookup {
    directories: HashMap<PathBuf, bool>,
}

impl NextJsProjectLookup {
    #[cfg(test)]
    pub(super) fn new(root: &Path, files: &[PathBuf], visible_files: &[PathBuf]) -> Self {
        Self::with_sources(root, files, visible_files, None)
    }

    pub(super) fn with_sources(
        root: &Path,
        files: &[PathBuf],
        visible_files: &[PathBuf],
        sources: Option<&crate::codebase::ts_source::SourceStore>,
    ) -> Self {
        let root = normalize_path(root);
        let visible_files = visible_files
            .iter()
            .map(|path| normalize_path(path))
            .collect::<HashSet<_>>();
        let mut directories = HashSet::from([root.clone()]);
        for path in files {
            let mut current = path
                .parent()
                .map(normalize_path)
                .unwrap_or_else(|| root.clone());
            loop {
                directories.insert(current.clone());
                if current == root || !current.pop() {
                    break;
                }
            }
        }

        let mut sorted: Vec<_> = directories.into_iter().collect();
        sorted.sort_by_key(|path| path.components().count());
        let mut directories = HashMap::new();
        for directory in sorted {
            let parent_is_nextjs = directory
                .parent()
                .and_then(|parent| directories.get(&normalize_path(parent)))
                .copied()
                .unwrap_or(false);
            let manifest = normalize_path(&directory.join("package.json"));
            directories.insert(
                directory,
                parent_is_nextjs
                    || (visible_files.contains(&manifest)
                        && package_json_has_next_from(&manifest, sources)),
            );
        }
        Self { directories }
    }

    pub(super) fn contains_file(&self, path: &Path) -> bool {
        path.parent()
            .map(normalize_path)
            .and_then(|directory| self.directories.get(&directory).copied())
            .unwrap_or(false)
    }
}

#[cfg(test)]
pub(super) fn package_json_has_next_dependency(path: &Path) -> bool {
    package_json_has_next_from(path, None)
}

fn package_json_has_next_from(
    path: &Path,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> bool {
    let Some(package_json) =
        crate::codebase::ts_source::SourceStore::parse_json_optional(sources, path)
    else {
        return false;
    };
    for field in ["dependencies", "devDependencies", "peerDependencies"] {
        let Some(dependencies) = package_json.get(field).and_then(|value| value.as_object()) else {
            continue;
        };
        if !dependencies.contains_key("next") {
            continue;
        }
        return true;
    }
    false
}

pub(super) fn sorted_paths<'a>(paths: impl Iterator<Item = &'a PathBuf>) -> Vec<&'a PathBuf> {
    let mut paths: Vec<_> = paths.collect();
    paths.sort();
    paths
}
