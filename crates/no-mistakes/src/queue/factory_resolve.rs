use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::queue::extract::FileFacts;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn resolve_imported_queue_names<R: ImportResolverFacade>(
    facts: &mut HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) {
    for _ in 0..facts.len().saturating_add(1) {
        let updates = pending_updates(facts, resolver, remapper);
        if updates.is_empty() {
            break;
        }
        let mut changed = false;
        for update in updates {
            changed |= apply_update(facts, update);
        }
        if !changed {
            break;
        }
    }
}

enum Update {
    Binding {
        file: PathBuf,
        binding: String,
        queue_name: String,
    },
    Export {
        file: PathBuf,
        exported: String,
        queue_name: String,
    },
    Worker {
        file: PathBuf,
        index: usize,
        queue_name: String,
    },
}

fn pending_updates<R: ImportResolverFacade>(
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Vec<Update> {
    let mut updates = Vec::new();
    for (path, file) in facts {
        for (binding, local) in &file.pending_queue_bindings {
            if file.queue_bindings.contains_key(binding) {
                continue;
            }
            if let Some(queue_name) = lookup(facts, resolver, remapper, path, local) {
                updates.push(Update::Binding {
                    file: path.clone(),
                    binding: binding.clone(),
                    queue_name,
                });
            }
        }
        for (exported, local) in &file.reexports {
            if file.queue_exports.contains_key(exported) {
                continue;
            }
            let queue_name = file
                .queue_bindings
                .get(local)
                .cloned()
                .or_else(|| lookup(facts, resolver, remapper, path, local));
            if let Some(queue_name) = queue_name {
                updates.push(Update::Export {
                    file: path.clone(),
                    exported: exported.clone(),
                    queue_name,
                });
            }
        }
        for (index, worker) in file.workers.iter().enumerate() {
            if worker.queue_name.is_some() {
                continue;
            }
            let Some(local) = &worker.queue_name_import else {
                continue;
            };
            if let Some(queue_name) = lookup(facts, resolver, remapper, path, local) {
                updates.push(Update::Worker {
                    file: path.clone(),
                    index,
                    queue_name,
                });
            }
        }
    }
    updates
}

fn apply_update(facts: &mut HashMap<PathBuf, FileFacts>, update: Update) -> bool {
    match update {
        Update::Binding {
            file,
            binding,
            queue_name,
        } => {
            let Some(file) = facts.get_mut(&file) else {
                return false;
            };
            let inserted = file
                .queue_bindings
                .insert(binding.clone(), queue_name.clone())
                .is_none();
            file.queue_exports.insert(binding, queue_name);
            inserted
        }
        Update::Export {
            file,
            exported,
            queue_name,
        } => {
            let Some(file) = facts.get_mut(&file) else {
                return false;
            };
            file.queue_exports.insert(exported, queue_name).is_none()
        }
        Update::Worker {
            file,
            index,
            queue_name,
        } => {
            let Some(worker) = facts
                .get_mut(&file)
                .and_then(|file| file.workers.get_mut(index))
            else {
                return false;
            };
            if worker.queue_name.is_some() {
                return false;
            }
            worker.queue_name = Some(queue_name);
            true
        }
    }
}

fn lookup<R: ImportResolverFacade>(
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    path: &Path,
    local: &str,
) -> Option<String> {
    let (specifier, imported) = facts.get(path)?.string_imports.get(local)?;
    let resolved = resolver.resolve(specifier, path)?;
    let resolved = remapper.remap(&resolved)?;
    let other = facts.get(&resolved)?;
    other
        .exported_strings
        .get(imported)
        .cloned()
        .or_else(|| other.queue_exports.get(imported).cloned())
}

pub(super) fn definition_files(facts: &HashMap<PathBuf, FileFacts>) -> HashMap<String, PathBuf> {
    let mut paths = facts.keys().cloned().collect::<Vec<_>>();
    paths.sort();
    let mut out = HashMap::new();
    for path in paths {
        let Some(file) = facts.get(&path) else {
            continue;
        };
        for name in file.queue_bindings.values() {
            out.entry(name.clone()).or_insert_with(|| path.clone());
        }
    }
    out
}
