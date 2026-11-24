use super::{ResolveCtx, ResolveMode, ResolvedExport};
use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::queue::extract::FileFacts;
use crate::queue::extract_helpers::is_queue_package;
use crate::queue::extract_model::ExportedWorkerBinding;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

impl<R: ImportResolverFacade> ResolveCtx<'_, R> {
    pub(super) fn follow_source(
        &self,
        path: &Path,
        specifier: &str,
        exported: &str,
        mode: ResolveMode,
        seen_queue: bool,
        visited: &mut HashSet<(PathBuf, String, ResolveMode)>,
    ) -> ResolvedExport {
        if is_queue_package(specifier) {
            return ResolvedExport::NonWorker;
        }
        let Some(target) = self
            .resolver
            .resolve(specifier, path)
            .and_then(|resolved| self.remapper.remap(&resolved))
        else {
            return ResolvedExport::NonWorker;
        };
        if !visited.insert((target.clone(), exported.to_string(), mode)) {
            return unresolved_if_queue(seen_queue);
        }
        self.follow_file(&target, exported, mode, seen_queue, visited)
    }

    fn follow_file(
        &self,
        path: &Path,
        exported: &str,
        mode: ResolveMode,
        seen_queue: bool,
        visited: &mut HashSet<(PathBuf, String, ResolveMode)>,
    ) -> ResolvedExport {
        let file = &self.facts[path];
        let seen_queue = seen_queue || super::super::is_queue_module(file);
        if let Some(res) = self.follow_mode_aliases(path, file, exported, mode, seen_queue, visited)
        {
            return res;
        }
        if mode != ResolveMode::CallReturn {
            if let Some(binding) = file.worker_exports.get(exported) {
                return match (mode, *binding) {
                    (ResolveMode::Instance, ExportedWorkerBinding::Worker(idx)) => {
                        ResolvedExport::Worker(path.to_path_buf(), idx)
                    }
                    (ResolveMode::Constructor, ExportedWorkerBinding::WorkerClass) => {
                        ResolvedExport::Worker(path.to_path_buf(), None)
                    }
                    _ => ResolvedExport::NonWorker,
                };
            }
        }
        if let Some((spec, imp)) = file
            .export_from
            .get(exported)
            .or_else(|| file.worker_namespace_exports.get(exported))
        {
            return self.follow_source(path, spec, imp, mode, seen_queue, visited);
        }
        if let Some(local) = file
            .worker_import_aliases
            .get(exported)
            .map(String::as_str)
            .or_else(|| {
                file.reexports
                    .iter()
                    .find(|(exp, _)| exp == exported)
                    .map(|(_, local)| local.as_str())
            })
        {
            return self.follow_local(path, file, local, mode, seen_queue, visited);
        }
        for specifier in &file.star_exports {
            if is_queue_package(specifier) {
                continue;
            }
            let result = self.follow_source(path, specifier, exported, mode, seen_queue, visited);
            if !matches!(result, ResolvedExport::NonWorker) {
                return result;
            }
        }
        unresolved_if_queue(seen_queue)
    }

    fn follow_mode_aliases(
        &self,
        path: &Path,
        file: &FileFacts,
        exported: &str,
        mode: ResolveMode,
        seen_queue: bool,
        visited: &mut HashSet<(PathBuf, String, ResolveMode)>,
    ) -> Option<ResolvedExport> {
        let (imports, namespaces, next_mode) = match mode {
            ResolveMode::Constructor => (
                &file.worker_class_imports,
                &file.worker_class_namespaces,
                ResolveMode::Constructor,
            ),
            ResolveMode::Instance => (
                &file.worker_constructed_aliases,
                &file.worker_constructed_namespaces,
                ResolveMode::Constructor,
            ),
            ResolveMode::CallReturn => return None,
        };
        if let Some(local) = imports.get(exported) {
            return Some(self.follow_local(path, file, local, next_mode, seen_queue, visited));
        }
        let (spec, imp) = namespaces.get(exported)?;
        Some(self.follow_source(path, spec, imp, next_mode, seen_queue, visited))
    }

    fn follow_local(
        &self,
        path: &Path,
        file: &FileFacts,
        local: &str,
        mode: ResolveMode,
        seen_queue: bool,
        visited: &mut HashSet<(PathBuf, String, ResolveMode)>,
    ) -> ResolvedExport {
        let Some(imp) = file.imports.iter().find(|imp| imp.local == local) else {
            return unresolved_if_queue(seen_queue);
        };
        self.follow_source(path, &imp.source, &imp.imported, mode, seen_queue, visited)
    }
}

fn unresolved_if_queue(seen_queue: bool) -> ResolvedExport {
    if seen_queue {
        ResolvedExport::QueueUnresolved
    } else {
        ResolvedExport::NonWorker
    }
}
