use crate::codebase::rules::read_source;
use crate::codebase::ts_source::SourceStore;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) fn lang_source(sources: &SourceStore, path: &Path) -> Option<Arc<str>> {
    read_source(sources, path)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LangFileFacts {
    pub path: PathBuf,
    pub package: Option<String>,
    pub module: Option<String>,
    pub imports: Vec<String>,
    pub declarations: Vec<String>,
    pub references: Vec<String>,
    pub route_handlers: Vec<(String, String)>,
    pub queue_enqueues: Vec<String>,
    pub queue_workers: Vec<String>,
    pub mods: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LangFactMap {
    pub files: BTreeMap<PathBuf, LangFileFacts>,
    pub declarations: HashMap<String, BTreeSet<PathBuf>>,
    pub files_by_module: HashMap<String, BTreeSet<PathBuf>>,
    pub files_by_package: HashMap<String, BTreeSet<PathBuf>>,
}

impl LangFactMap {
    pub(crate) fn index_file(&mut self, file: LangFileFacts) {
        if let Some(package) = &file.package {
            self.files_by_package
                .entry(package.clone())
                .or_default()
                .insert(file.path.clone());
        }
        if let Some(module) = &file.module {
            if !file
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("_test.go"))
            {
                self.files_by_module
                    .entry(module.clone())
                    .or_default()
                    .insert(file.path.clone());
            }
        }
        for declaration in &file.declarations {
            self.declarations
                .entry(declaration.clone())
                .or_default()
                .insert(file.path.clone());
            if file.path.extension().and_then(|ext| ext.to_str()) == Some("php") {
                self.files_by_module
                    .entry(declaration.clone())
                    .or_default()
                    .insert(file.path.clone());
            }
        }
        self.files.insert(file.path.clone(), file);
    }
}

pub(crate) fn configured_roots(root: &Path, entries: &[String]) -> Vec<PathBuf> {
    entries
        .iter()
        .map(|entry| {
            crate::codebase::ts_resolver::normalize_path(&root.join(entry.trim_end_matches('/')))
        })
        .collect()
}

pub(crate) fn index_parsed_files(mut files: Vec<LangFileFacts>) -> LangFactMap {
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let mut facts = LangFactMap::default();
    for file in files {
        facts.index_file(file);
    }
    facts
}

pub(crate) fn collect_files_parallel<F>(files: Vec<PathBuf>, parse: F) -> LangFactMap
where
    F: Fn(&Path) -> Option<LangFileFacts> + Sync,
{
    use rayon::prelude::*;
    let parsed: Vec<LangFileFacts> = files.par_iter().filter_map(|path| parse(path)).collect();
    index_parsed_files(parsed)
}

pub(crate) fn files_under(
    all_files: &[PathBuf],
    roots: &[PathBuf],
    extension: &str,
) -> Vec<PathBuf> {
    if roots.is_empty() {
        return Vec::new();
    }
    all_files
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some(extension))
        .filter(|path| roots.iter().any(|root| path.starts_with(root)))
        .cloned()
        .collect()
}

pub(crate) fn module_from_path(package_root: &Path, path: &Path) -> Option<String> {
    module_from_path_inner(package_root, path, false)
}

pub(crate) fn rust_module_from_path(package_root: &Path, path: &Path) -> Option<String> {
    module_from_path_inner(package_root, path, true)
}

fn module_from_path_inner(package_root: &Path, path: &Path, rust: bool) -> Option<String> {
    let rel = path.strip_prefix(package_root).ok()?;
    let mut parts: Vec<String> = rel
        .iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect();
    if let Some(last) = parts.last_mut() {
        if let Some((stem, _)) = last.rsplit_once('.') {
            *last = stem.to_string();
        }
    }
    if parts.last().is_some_and(|part| part == "__init__") {
        parts.pop();
    }
    if rust && parts.last().is_some_and(|part| part == "mod") {
        parts.pop();
    }
    if rust && parts.len() == 1 && matches!(parts[0].as_str(), "lib" | "main") {
        return None;
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("."))
}

pub(crate) fn owning_package<'a>(
    path: &'a Path,
    roots: &'a [PathBuf],
    names: &'a [String],
) -> Option<String> {
    roots
        .iter()
        .zip(names.iter())
        .filter(|(root, _)| path.starts_with(root))
        .max_by_key(|(root, _)| root.components().count())
        .map(|(_, name)| name.trim_end_matches('/').to_string())
}
