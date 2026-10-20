use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::queue::extract::FileFacts;
use crate::queue::graph_model::{InternalProducer, InternalWorker};
use crate::queue::types::QueueKey;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn queue_definitions(
    facts: &HashMap<PathBuf, FileFacts>,
) -> HashMap<PathBuf, HashMap<String, String>> {
    facts
        .iter()
        .map(|(path, facts)| (path.clone(), facts.queue_exports.clone()))
        .collect()
}

pub(super) fn resolve_producers<R: ImportResolverFacade>(
    facts: &HashMap<PathBuf, FileFacts>,
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
    definitions: &HashMap<String, PathBuf>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Vec<InternalProducer> {
    let mut out = Vec::new();
    for (path, file) in facts {
        let local = local_queues(path, file, queue_defs, definitions, resolver, remapper);
        for site in &file.producers {
            let queue = local.get(&site.binding).cloned().or_else(|| {
                crate::queue::factory_resolve::binding_queue(
                    facts,
                    resolver,
                    remapper,
                    path,
                    &site.binding,
                    definitions,
                )
            });
            out.push(InternalProducer {
                site: site.clone(),
                queue,
            });
        }
    }
    out
}

pub(super) fn resolve_workers<R: ImportResolverFacade>(
    facts: &HashMap<PathBuf, FileFacts>,
    definitions: &HashMap<String, PathBuf>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Vec<InternalWorker> {
    let mut out = Vec::new();
    for (path, file) in facts {
        for site in &file.workers {
            let queue = site.queue_name.as_ref().and_then(|name| {
                definitions.get(name).map(|file| QueueKey {
                    queue_file: file.clone(),
                    queue_name: name.clone(),
                })
            });
            let mut site = site.clone();
            site.processor_file = site
                .processor_specifier
                .as_ref()
                .and_then(|spec| resolver.resolve(spec, path))
                .and_then(|path| remapper.remap(&path));
            adopt_processor_jobs(&mut site, facts);
            out.push(InternalWorker { site, queue });
        }
    }
    out
}

fn adopt_processor_jobs(
    site: &mut crate::queue::extract::WorkerSite,
    facts: &HashMap<PathBuf, FileFacts>,
) {
    if !site.jobs.is_empty() {
        return;
    }
    let Some(file) = &site.processor_file else {
        return;
    };
    // `processor_file` is remapped from `facts.keys()`.
    let processor = &facts[file];
    let jobs = processor.referenced_jobs.clone();
    if jobs.is_empty() {
        return;
    }
    site.jobs = jobs;
    site.wildcard = processor.referenced_jobs_dynamic;
}

fn local_queues<R: ImportResolverFacade>(
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
                    queue_file: path.to_path_buf(),
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
