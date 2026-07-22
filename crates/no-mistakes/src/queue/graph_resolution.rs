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
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Vec<InternalProducer> {
    let mut out = Vec::new();
    for (path, facts) in facts {
        let local = local_queues(path, facts, queue_defs, resolver, remapper);
        for site in &facts.producers {
            let queue = local.get(&site.binding).cloned();
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
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Vec<InternalWorker> {
    let by_name = queue_defs
        .iter()
        .flat_map(|(file, exports)| exports.values().map(|name| (name.clone(), file.clone())))
        .collect::<HashMap<_, _>>();
    let mut out = Vec::new();
    for (path, facts) in facts {
        for site in &facts.workers {
            let queue = site.queue_name.as_ref().and_then(|name| {
                by_name.get(name).map(|file| QueueKey {
                    queue_file: file.clone(),
                    queue_name: name.clone(),
                })
            });
            let mut site = site.clone();
            site.processor_file = site
                .processor_specifier
                .as_ref()
                .and_then(|spec| resolver.resolve(spec, path))
                .map(|path| remapper.remap(&path));
            out.push(InternalWorker { site, queue });
        }
    }
    out
}

fn local_queues<R: ImportResolverFacade>(
    path: &Path,
    facts: &FileFacts,
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
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
        let resolved = remapper.remap(&resolved);
        if let Some(exports) = queue_defs.get(&resolved) {
            if let Some(queue_name) = exports.get(&import.imported) {
                map.insert(
                    import.local.clone(),
                    QueueKey {
                        queue_file: resolved,
                        queue_name: queue_name.clone(),
                    },
                );
            }
        }
    }
    map
}
