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
        for update in updates {
            apply_update(facts, update);
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
    ProducerJob {
        file: PathBuf,
        index: usize,
        job: String,
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
            if let Some(queue_name) =
                follow::lookup(facts, resolver, remapper, path, local, follow::Want::Either)
            {
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
            let queue_name = file.queue_bindings.get(local).cloned().or_else(|| {
                follow::lookup(facts, resolver, remapper, path, local, follow::Want::Queue)
            });
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
            if let Some(queue_name) =
                follow::lookup(facts, resolver, remapper, path, local, follow::Want::Either)
            {
                updates.push(Update::Worker {
                    file: path.clone(),
                    index,
                    queue_name,
                });
            }
        }
    }
    updates.extend(follow::star_and_job_updates(facts, resolver, remapper));
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
        Update::ProducerJob { file, index, job } => {
            let Some(producer) = facts
                .get_mut(&file)
                .and_then(|file| file.producers.get_mut(index))
            else {
                return false;
            };
            if producer.job.is_some() {
                return false;
            }
            producer.job = Some(job);
            true
        }
    }
}

#[cfg(test)]
#[path = "factory_resolve_apply_tests.rs"]
mod apply_tests;

#[path = "factory_resolve_follow.rs"]
mod follow;

pub(super) fn binding_queue<R: ImportResolverFacade>(
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    path: &Path,
    binding: &str,
    definitions: &HashMap<String, PathBuf>,
) -> Option<crate::queue::types::QueueKey> {
    let queue_name = follow::lookup(
        facts,
        resolver,
        remapper,
        path,
        binding,
        follow::Want::Queue,
    )?;
    Some(crate::queue::types::QueueKey {
        queue_file: definitions.get(&queue_name)?.clone(),
        queue_name,
    })
}

pub(super) fn definition_files(facts: &HashMap<PathBuf, FileFacts>) -> HashMap<String, PathBuf> {
    let mut paths = facts.keys().cloned().collect::<Vec<_>>();
    paths.sort();
    let mut out = HashMap::new();
    for path in paths {
        for name in facts[&path].queue_bindings.values() {
            out.entry(name.clone()).or_insert_with(|| path.clone());
        }
    }
    out
}
