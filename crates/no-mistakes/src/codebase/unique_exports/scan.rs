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

pub(super) fn collect_source_files_from_facts(
    root: &Path,
    files: &[PathBuf],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<SourceFile>> {
    let nextjs_projects = NextJsProjectLookup::new(root, files, shared.files());
    let mut source_files = Vec::new();
    for path in files {
        let Some(facts) = shared.ts.get(path) else {
            anyhow::bail!("missing shared facts for {}", path.display());
        };
        let Some(source) = facts.source.clone() else {
            anyhow::bail!("missing source facts for {}", path.display());
        };
        let disabled = has_disable_file_comment(&source, RULE_ID);
        if !disabled {
            if let Some(error) = &facts.parse_error {
                anyhow::bail!("failed to parse {}: {error}", path.display());
            }
        }
        let symbols = if disabled {
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
            disabled,
            is_nextjs_project: nextjs_projects.contains_file(path),
            source: source.to_string(),
            symbols,
        });
    }
    Ok(source_files)
}

pub(super) struct NextJsProjectLookup {
    directories: HashMap<PathBuf, bool>,
}

impl NextJsProjectLookup {
    pub(super) fn new(root: &Path, files: &[PathBuf], visible_files: &[PathBuf]) -> Self {
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
                        && package_json_has_next_dependency(&manifest)),
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

pub(super) fn package_json_has_next_dependency(path: &Path) -> bool {
    let Ok(source) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&source) else {
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
