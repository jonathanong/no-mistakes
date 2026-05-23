use super::types::{component_key, GlobMatcher};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_resolver::{normalize_path, ImportResolver};
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::ts_symbols::ExportKind;
use std::collections::{BTreeSet, HashSet, VecDeque};
use std::path::{Path, PathBuf};

pub(super) fn reachable_story_files(
    project_root: &Path,
    shared: &CheckFactMap,
    stories: &GlobMatcher,
    resolver: &ImportResolver<'_>,
    _all_component_keys: &HashSet<String>,
) -> BTreeSet<PathBuf> {
    let mut queue: VecDeque<PathBuf> = story_files_matching(project_root, shared, stories)
        .into_iter()
        .collect();
    let mut seen = BTreeSet::new();
    while let Some(file) = queue.pop_front() {
        if !seen.insert(file.clone()) {
            continue;
        }
        let Some(facts) = shared.ts.get(&file) else {
            continue;
        };
        if facts.parse_error.is_some() {
            continue;
        }
        let Some(storybook) = facts.storybook.as_ref() else {
            continue;
        };
        for import in &storybook.used_runtime_imports {
            let Some(resolved) = resolver
                .resolve(&import.source, &file)
                .map(|p| normalize_path(&p))
            else {
                continue;
            };
            if resolved.starts_with(project_root) && is_story_file(project_root, stories, &resolved)
            {
                queue.push_back(resolved);
            }
        }
        for import in &storybook.side_effect_imports {
            let Some(resolved) = resolver
                .resolve(&import.source, &file)
                .map(|p| normalize_path(&p))
            else {
                continue;
            };
            if resolved.starts_with(project_root) && is_story_file(project_root, stories, &resolved)
            {
                queue.push_back(resolved);
            }
        }
    }
    seen
}

pub(super) fn story_files_matching(
    project_root: &Path,
    shared: &CheckFactMap,
    stories: &GlobMatcher,
) -> BTreeSet<PathBuf> {
    shared
        .files()
        .iter()
        .filter(|path| path.starts_with(project_root))
        .filter(|path| stories.is_match(&relative_slash_path(project_root, path)))
        .map(|path| normalize_path(path))
        .collect()
}

fn is_story_file(project_root: &Path, stories: &GlobMatcher, path: &Path) -> bool {
    let rel = relative_slash_path(project_root, path);
    stories.is_match(&rel)
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains(".stories.") || name.contains(".story."))
}

pub(super) fn directly_covered_components(
    project_root: &Path,
    shared: &CheckFactMap,
    story_files: &BTreeSet<PathBuf>,
    resolver: &ImportResolver<'_>,
    component_keys: &HashSet<String>,
) -> BTreeSet<String> {
    let mut covered = BTreeSet::new();
    for file in story_files {
        let Some(facts) = shared
            .ts
            .get(file)
            .and_then(|facts| facts.storybook.as_ref())
        else {
            continue;
        };
        for import in &facts.used_runtime_imports {
            if import.namespace {
                continue;
            }
            let Some(resolved) = resolver
                .resolve(&import.source, file)
                .map(|p| normalize_path(&p))
            else {
                continue;
            };
            if let Some(key) = resolve_component_key(
                project_root,
                shared,
                &resolved,
                &import.imported,
                component_keys,
                resolver,
                &mut HashSet::new(),
            ) {
                covered.insert(key);
            }
        }
    }
    covered
}

pub(super) fn all_react_component_keys(
    project_root: &Path,
    shared: &CheckFactMap,
) -> HashSet<String> {
    let mut out = HashSet::new();
    for (path, facts) in &shared.ts {
        if !path.starts_with(project_root) {
            continue;
        }
        let Some(react) = facts.react.as_ref() else {
            continue;
        };
        let project_file = relative_slash_path(project_root, path);
        for component in &react.components {
            out.insert(component_key(&project_file, &component.name));
        }
    }
    out
}

fn resolve_component_key(
    project_root: &Path,
    shared: &CheckFactMap,
    file: &Path,
    export_name: &str,
    component_keys: &HashSet<String>,
    resolver: &ImportResolver<'_>,
    visiting: &mut HashSet<(PathBuf, String)>,
) -> Option<String> {
    if !file.starts_with(project_root) {
        return None;
    }
    let project_file = relative_slash_path(project_root, file);
    let direct = component_key(&project_file, export_name);
    if component_keys.contains(&direct) {
        return Some(direct);
    }
    if !visiting.insert((file.to_path_buf(), export_name.to_string())) {
        return None;
    }
    let facts = shared.ts.get(file)?;
    let symbols = facts.symbols.as_ref()?;
    for export in &symbols.exports {
        let ExportKind::ReExport { source, imported } = &export.kind else {
            continue;
        };
        if export.is_type_only {
            continue;
        }
        if export.name != export_name && export.name != "*" {
            continue;
        }
        let resolved = resolver.resolve(source, file).map(|p| normalize_path(&p))?;
        let target_name = if imported == "*" {
            export_name
        } else {
            imported
        };
        if let Some(key) = resolve_component_key(
            project_root,
            shared,
            &resolved,
            target_name,
            component_keys,
            resolver,
            visiting,
        ) {
            return Some(key);
        }
    }
    None
}
