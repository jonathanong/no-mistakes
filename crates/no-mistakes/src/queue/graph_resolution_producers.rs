use super::{FileFacts, ImportResolverFacade, QueueKey};
use crate::queue::extract_model::{JobIdQueue, LocalQueue, ProducerBinding};
use crate::queue::factory_resolve::follow::{lookup_export, Want};
use crate::queue::graph_model::InternalProducer;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(in crate::queue) fn resolve_producers<R: ImportResolverFacade>(
    facts: &mut HashMap<PathBuf, FileFacts>,
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
    definitions: &HashMap<String, PathBuf>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Vec<InternalProducer> {
    let mut out = Vec::new();
    let mut opaque = Vec::new();
    let mut checks = Vec::new();
    for (path, file) in facts.iter() {
        let local = local_queues(path, file, queue_defs, definitions, resolver, remapper);
        for site in &file.producers {
            let queue = resolved_binding(
                &local,
                facts,
                resolver,
                remapper,
                path,
                &site.binding,
                definitions,
            );
            out.push(InternalProducer {
                site: site.clone(),
                queue,
            });
        }
        for (binding, check) in &file.job_id_sites {
            let resolved = match binding {
                JobIdQueue::Named(binding) => resolved_binding(
                    &local,
                    facts,
                    resolver,
                    remapper,
                    path,
                    binding,
                    definitions,
                )
                .is_some(),
                JobIdQueue::Namespace { source, exported } => lookup_export(
                    facts,
                    resolver,
                    remapper,
                    path,
                    source,
                    exported,
                    Want::Queue,
                )
                .is_some_and(|name| definitions.contains_key(&name)),
            };
            if resolved {
                checks.push((path.clone(), check.clone()));
            }
        }
        let queues = file
            .opaque_priority_bindings
            .iter()
            .filter_map(|binding| {
                resolved_binding(
                    &local,
                    facts,
                    resolver,
                    remapper,
                    path,
                    binding,
                    definitions,
                )
            })
            .map(|queue| (queue.queue_file, queue.queue_name))
            .collect::<Vec<_>>();
        if !queues.is_empty() {
            opaque.push((path.clone(), queues));
        }
    }
    for (path, queues) in opaque {
        if let Some(file) = facts.get_mut(&path) {
            file.opaque_priority_queues = queues;
        }
    }
    for (path, check) in checks {
        if let Some(file) = facts.get_mut(&path) {
            file.checks.push(check);
        }
    }
    out
}

pub(in crate::queue) fn resolved_binding<R: ImportResolverFacade>(
    local: &HashMap<String, QueueKey>,
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    path: &Path,
    binding: &ProducerBinding,
    definitions: &HashMap<String, PathBuf>,
) -> Option<QueueKey> {
    let binding = match binding {
        ProducerBinding::Unknown => return None,
        ProducerBinding::Named(binding) => binding,
        ProducerBinding::QueueName(name) => {
            return Some(QueueKey {
                queue_file: definitions.get(name)?.clone(),
                queue_name: name.clone(),
            });
        }
        ProducerBinding::Flow(queue_name) => {
            return Some(QueueKey {
                queue_file: definitions
                    .get(queue_name)
                    .cloned()
                    .unwrap_or_else(|| path.to_path_buf()),
                queue_name: queue_name.clone(),
            });
        }
        ProducerBinding::Local(declaration) => {
            match facts[path].queue_declarations.get(declaration)? {
                LocalQueue::Known(queue_name) => {
                    return Some(QueueKey {
                        queue_file: definitions
                            .get(queue_name)
                            .cloned()
                            .unwrap_or_else(|| path.to_path_buf()),
                        queue_name: queue_name.clone(),
                    });
                }
                LocalQueue::ImportedQueue(binding) => binding,
                LocalQueue::ImportedName(_) => return None,
            }
        }
    };
    local.get(binding).cloned().or_else(|| {
        crate::queue::factory_resolve::binding_queue(
            facts,
            resolver,
            remapper,
            path,
            binding,
            definitions,
        )
    })
}

pub(in crate::queue) fn local_queues<R: ImportResolverFacade>(
    path: &Path,
    facts: &FileFacts,
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
    definitions: &HashMap<String, PathBuf>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> HashMap<String, QueueKey> {
    let mut map = facts
        .queue_bindings
        .iter()
        .map(|(binding, queue_name)| {
            (
                binding.clone(),
                QueueKey {
                    queue_file: definitions
                        .get(queue_name)
                        .cloned()
                        .unwrap_or_else(|| path.to_path_buf()),
                    queue_name: queue_name.clone(),
                },
            )
        })
        .collect::<HashMap<_, _>>();
    for import in &facts.imports {
        let Some(resolved) = resolver.resolve(&import.source, path) else {
            continue;
        };
        let Some(resolved) = remapper.remap(&resolved) else {
            continue;
        };
        // `resolved` is a remapped fact path, and `queue_defs` is built from those keys.
        let exports = &queue_defs[&resolved];
        if let Some(queue_name) = exports.get(&import.imported) {
            map.insert(
                import.local.clone(),
                QueueKey {
                    queue_file: definitions.get(queue_name).cloned().unwrap_or(resolved),
                    queue_name: queue_name.clone(),
                },
            );
        }
    }
    map
}
