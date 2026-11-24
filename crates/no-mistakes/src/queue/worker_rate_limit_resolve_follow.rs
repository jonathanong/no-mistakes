use super::{FollowState, ResolveCtx, ResolveMode, ResolvedExport};
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
        state: FollowState,
        visited: &mut HashSet<(PathBuf, String, ResolveMode)>,
    ) -> ResolvedExport {
        if is_queue_package(specifier) {
            return if mode == ResolveMode::Constructor
                && matches!(exported, "Worker" | "TestWorker")
            {
                ResolvedExport::Worker(path.to_path_buf(), None)
            } else {
                ResolvedExport::NonWorker
            };
        }
        let Some(target) = self
            .resolver
            .resolve(specifier, path)
            .and_then(|resolved| self.remapper.remap(&resolved))
        else {
            return unresolved_if_queue(state.seen_queue && state.from_class_base);
        };
        if !visited.insert((target.clone(), exported.to_string(), mode)) {
            return unresolved_if_queue(state.seen_queue);
        }
        self.follow_file(&target, exported, mode, state, visited)
    }

    fn follow_file(
        &self,
        path: &Path,
        exported: &str,
        mode: ResolveMode,
        state: FollowState,
        visited: &mut HashSet<(PathBuf, String, ResolveMode)>,
    ) -> ResolvedExport {
        let file = &self.facts[path];
        let state = FollowState {
            seen_queue: state.seen_queue || super::super::is_queue_module(file),
            ..state
        };
        if let Some(res) = self.follow_mode_aliases(path, file, exported, mode, state, visited) {
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
            return self.follow_source(path, spec, imp, mode, state, visited);
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
            return self.follow_local(path, file, local, mode, state, visited);
        }
        let mut fallback = unresolved_if_queue(state.seen_queue);
        for specifier in &file.star_exports {
            match self.follow_source(path, specifier, exported, mode, state, visited) {
                worker @ ResolvedExport::Worker(..) => return worker,
                ResolvedExport::QueueUnresolved => fallback = ResolvedExport::QueueUnresolved,
                ResolvedExport::NonWorker => {}
            }
        }
        fallback
    }

    fn follow_mode_aliases(
        &self,
        path: &Path,
        file: &FileFacts,
        exported: &str,
        mode: ResolveMode,
        state: FollowState,
        visited: &mut HashSet<(PathBuf, String, ResolveMode)>,
    ) -> Option<ResolvedExport> {
        let (imports, namespaces, next_mode, from_class_base) = match mode {
            ResolveMode::Constructor => (
                &file.worker_class_imports,
                &file.worker_class_namespaces,
                ResolveMode::Constructor,
                true,
            ),
            ResolveMode::Instance => (
                &file.worker_constructed_aliases,
                &file.worker_constructed_namespaces,
                ResolveMode::Constructor,
                state.from_class_base,
            ),
            ResolveMode::CallReturn => return None,
        };
        let next_state = FollowState {
            from_class_base,
            ..state
        };
        if let Some(local) = imports.get(exported) {
            return Some(self.follow_local(path, file, local, next_mode, next_state, visited));
        }
        let (spec, imp) = namespaces.get(exported)?;
        Some(self.follow_source(path, spec, imp, next_mode, next_state, visited))
    }

    fn follow_local(
        &self,
        path: &Path,
        file: &FileFacts,
        local: &str,
        mode: ResolveMode,
        state: FollowState,
        visited: &mut HashSet<(PathBuf, String, ResolveMode)>,
    ) -> ResolvedExport {
        let Some(imp) = file.imports.iter().find(|imp| imp.local == local) else {
            return unresolved_if_queue(state.seen_queue);
        };
        self.follow_source(path, &imp.source, &imp.imported, mode, state, visited)
    }
}

fn unresolved_if_queue(seen_queue: bool) -> ResolvedExport {
    if seen_queue {
        ResolvedExport::QueueUnresolved
    } else {
        ResolvedExport::NonWorker
    }
}
