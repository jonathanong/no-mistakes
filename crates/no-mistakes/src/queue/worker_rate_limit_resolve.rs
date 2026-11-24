use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::codebase::ts_source::FrozenPathRemapper;
use crate::queue::extract::FileFacts;
use crate::queue::extract_model::WorkerRateLimitTarget;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[path = "worker_rate_limit_resolve_follow.rs"]
mod follow;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum ResolveMode {
    Instance,
    Constructor,
    CallReturn,
}

pub(super) enum ResolvedExport {
    Worker(PathBuf, Option<usize>),
    QueueUnresolved,
    NonWorker,
}

pub(super) struct ResolveCtx<'a, R> {
    pub(super) facts: &'a HashMap<PathBuf, FileFacts>,
    pub(super) resolver: &'a R,
    pub(super) remapper: &'a FrozenPathRemapper,
}

pub(in crate::queue) fn resolve<R: ImportResolverFacade>(
    facts: &mut HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &FrozenPathRemapper,
) {
    let ctx = ResolveCtx {
        facts,
        resolver,
        remapper,
    };
    let mut updates: Vec<(PathBuf, usize, Option<WorkerRateLimitTarget>)> = Vec::new();
    for (path, file) in facts.iter() {
        for (index, site) in file.worker_rate_limits.iter().enumerate() {
            let resolved = match &site.target {
                WorkerRateLimitTarget::Imported { local } => {
                    ctx.resolve_imported(path, file, local, ResolveMode::Instance)
                }
                WorkerRateLimitTarget::Namespace { source, exported } => {
                    ctx.resolve_export(path, source, exported, ResolveMode::Instance)
                }
                WorkerRateLimitTarget::ConstructedImported { local } => {
                    ctx.resolve_imported(path, file, local, ResolveMode::Constructor)
                }
                WorkerRateLimitTarget::ConstructedNamespace { source, exported } => {
                    ctx.resolve_export(path, source, exported, ResolveMode::Constructor)
                }
                WorkerRateLimitTarget::CallImported { local } => {
                    ctx.resolve_imported(path, file, local, ResolveMode::CallReturn)
                }
                WorkerRateLimitTarget::CallNamespace { source, exported } => {
                    ctx.resolve_export(path, source, exported, ResolveMode::CallReturn)
                }
                _ => continue,
            };
            updates.push((path.clone(), index, resolved));
        }
    }
    for (path, index, resolved) in updates.into_iter().rev() {
        let file = facts.get_mut(&path).expect("path came from facts");
        match resolved {
            Some(target) => file.worker_rate_limits[index].target = target,
            None => {
                file.worker_rate_limits.remove(index);
            }
        }
    }
}

impl<R: ImportResolverFacade> ResolveCtx<'_, R> {
    fn resolve_imported(
        &self,
        path: &Path,
        file: &FileFacts,
        local: &str,
        mode: ResolveMode,
    ) -> Option<WorkerRateLimitTarget> {
        let binding = file.imports.iter().find(|imp| imp.local == local)?;
        self.resolve_export(path, &binding.source, &binding.imported, mode)
    }

    fn resolve_export(
        &self,
        path: &Path,
        specifier: &str,
        exported: &str,
        mode: ResolveMode,
    ) -> Option<WorkerRateLimitTarget> {
        let mut visited = HashSet::new();
        match self.follow_source(path, specifier, exported, mode, false, &mut visited) {
            ResolvedExport::Worker(file, worker_index) => {
                Some(WorkerRateLimitTarget::ResolvedWorker { file, worker_index })
            }
            ResolvedExport::QueueUnresolved => Some(WorkerRateLimitTarget::Unresolved),
            ResolvedExport::NonWorker => None,
        }
    }
}
