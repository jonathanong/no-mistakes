use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::codebase::ts_source::FrozenPathRemapper;
use crate::queue::extract::FileFacts;
use crate::queue::extract_helpers::is_queue_package;
use crate::queue::extract_model::{ExportedWorkerBinding, WorkerRateLimitTarget};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub(in crate::queue) fn resolve<R: ImportResolverFacade>(
    facts: &mut HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &FrozenPathRemapper,
) {
    let mut updates: Vec<(PathBuf, usize, Option<WorkerRateLimitTarget>)> = Vec::new();
    for (path, file) in facts.iter() {
        for (index, site) in file.worker_rate_limits.iter().enumerate() {
            let resolved = match &site.target {
                WorkerRateLimitTarget::Imported { local } => {
                    resolve_imported(facts, resolver, remapper, path, file, local)
                }
                WorkerRateLimitTarget::Namespace { source, exported } => {
                    resolve_export(facts, resolver, remapper, path, source, exported)
                }
                _ => continue,
            };
            updates.push((path.clone(), index, resolved));
        }
    }
    for (path, index, resolved) in updates.into_iter().rev() {
        let Some(file) = facts.get_mut(&path) else {
            continue;
        };
        match resolved {
            Some(target) => file.worker_rate_limits[index].target = target,
            None => {
                file.worker_rate_limits.remove(index);
            }
        }
    }
}

fn resolve_imported<R: ImportResolverFacade>(
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &FrozenPathRemapper,
    path: &Path,
    file: &FileFacts,
    local: &str,
) -> Option<WorkerRateLimitTarget> {
    let Some(binding) = file.imports.iter().find(|imp| imp.local == local) else {
        return Some(WorkerRateLimitTarget::Unresolved);
    };
    resolve_export(
        facts,
        resolver,
        remapper,
        path,
        &binding.source,
        &binding.imported,
    )
}

fn resolve_export<R: ImportResolverFacade>(
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &FrozenPathRemapper,
    path: &Path,
    specifier: &str,
    exported: &str,
) -> Option<WorkerRateLimitTarget> {
    if is_queue_package(specifier) {
        return None;
    }
    let mut visited = HashSet::new();
    match follow_source(
        facts,
        resolver,
        remapper,
        path,
        specifier,
        exported,
        &mut visited,
    ) {
        ResolvedExport::Worker(file, worker_index) => {
            Some(WorkerRateLimitTarget::ResolvedWorker { file, worker_index })
        }
        ResolvedExport::NonWorker => None,
        ResolvedExport::Unresolved => Some(WorkerRateLimitTarget::Unresolved),
    }
}

enum ResolvedExport {
    Worker(PathBuf, usize),
    NonWorker,
    Unresolved,
}

fn follow_source<R: ImportResolverFacade>(
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &FrozenPathRemapper,
    path: &Path,
    specifier: &str,
    exported: &str,
    visited: &mut HashSet<(PathBuf, String)>,
) -> ResolvedExport {
    if is_queue_package(specifier) {
        return ResolvedExport::NonWorker;
    }
    let Some(target) = resolver
        .resolve(specifier, path)
        .and_then(|resolved| remapper.remap(&resolved))
    else {
        return ResolvedExport::Unresolved;
    };
    if !visited.insert((target.clone(), exported.to_string())) {
        return ResolvedExport::Unresolved;
    }
    follow_file(facts, resolver, remapper, &target, exported, visited)
}

fn follow_file<R: ImportResolverFacade>(
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &FrozenPathRemapper,
    path: &Path,
    exported: &str,
    visited: &mut HashSet<(PathBuf, String)>,
) -> ResolvedExport {
    let Some(file) = facts.get(path) else {
        return ResolvedExport::Unresolved;
    };
    if let Some(binding) = file.worker_exports.get(exported) {
        return match *binding {
            ExportedWorkerBinding::Worker(index) => {
                ResolvedExport::Worker(path.to_path_buf(), index)
            }
            ExportedWorkerBinding::NonWorker => ResolvedExport::NonWorker,
        };
    }
    if let Some((specifier, imported)) = file.export_from.get(exported) {
        return follow_source(
            facts, resolver, remapper, path, specifier, imported, visited,
        );
    }
    if let Some(local) = file.worker_import_aliases.get(exported).map(String::as_str) {
        return follow_local_import(facts, resolver, remapper, path, file, local, visited);
    }
    if let Some((_, local)) = file.reexports.iter().find(|(exp, _)| exp == exported) {
        return follow_local_import(facts, resolver, remapper, path, file, local, visited);
    }
    for specifier in &file.star_exports {
        let result = follow_source(
            facts, resolver, remapper, path, specifier, exported, visited,
        );
        if !matches!(result, ResolvedExport::Unresolved) {
            return result;
        }
    }
    ResolvedExport::Unresolved
}

fn follow_local_import<R: ImportResolverFacade>(
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &FrozenPathRemapper,
    path: &Path,
    file: &FileFacts,
    local: &str,
    visited: &mut HashSet<(PathBuf, String)>,
) -> ResolvedExport {
    let Some(imp) = file.imports.iter().find(|imp| imp.local == local) else {
        return ResolvedExport::Unresolved;
    };
    follow_source(
        facts,
        resolver,
        remapper,
        path,
        &imp.source,
        &imp.imported,
        visited,
    )
}
