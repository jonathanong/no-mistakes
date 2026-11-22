use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::queue::extract::FileFacts;
use crate::queue::graph_model::InternalWorker;
use crate::queue::types::QueueKey;
use std::collections::HashMap;
use std::path::PathBuf;

pub(super) fn queue_definitions(
    facts: &HashMap<PathBuf, FileFacts>,
) -> HashMap<PathBuf, HashMap<String, String>> {
    facts
        .iter()
        .map(|(path, facts)| (path.clone(), facts.queue_exports.clone()))
        .collect()
}

#[path = "graph_resolution_producers.rs"]
mod producers;

pub(super) use producers::resolve_producers;

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
