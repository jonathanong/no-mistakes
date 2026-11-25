use super::{local_queues, producers::resolved_binding};
use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::queue::extract::FileFacts;
use crate::queue::extract_model::{LocalQueue, ProducerBinding};
use crate::queue::types::QueueKey;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

pub(in crate::queue) fn resolve_job_option_sites<R: ImportResolverFacade>(
    facts: &mut HashMap<PathBuf, FileFacts>,
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
    definitions: &HashMap<String, PathBuf>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) {
    let mut declaring_files = HashMap::<&str, BTreeSet<&PathBuf>>::new();
    for (path, file) in facts.iter() {
        for name in crate::queue::factory_resolve::defined_queue_names(file) {
            declaring_files.entry(name).or_default().insert(path);
        }
    }
    let mut slots = Vec::new();
    for (path, file) in facts.iter() {
        let local = local_queues(path, file, queue_defs, definitions, resolver, remapper);
        for (index, site) in file.job_option_sites.iter().enumerate() {
            let key = site
                .queue_name
                .as_ref()
                .map(|name| QueueKey {
                    queue_file: definitions
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| path.clone()),
                    queue_name: name.clone(),
                })
                .or_else(|| {
                    resolved_binding(
                        &local,
                        facts,
                        resolver,
                        remapper,
                        path,
                        site.binding.as_ref()?,
                        definitions,
                    )
                });
            if let Some(key) = key {
                let queue_file = match declaring_files.get(key.queue_name.as_str()) {
                    Some(paths) if paths.len() > 1 => site.binding.as_ref().and_then(|binding| {
                        if matches!(
                            binding,
                            ProducerBinding::Local(declaration)
                                if matches!(file.queue_declarations.get(declaration), Some(LocalQueue::Known(_)))
                        ) {
                            return Some(path.clone());
                        }
                        let binding = imported_binding(file, binding)?;
                        let (origin, name) = crate::queue::factory_resolve::lookup_queue_origin(
                            facts, resolver, remapper, path, binding,
                        )?;
                        (name == key.queue_name && paths.contains(&origin)).then_some(origin)
                    }),
                    Some(paths) => paths.first().map(|path| (*path).clone()),
                    None => Some(key.queue_file),
                };
                slots.push((path.clone(), index, key.queue_name, queue_file));
            }
        }
    }
    for (path, index, queue_name, queue_file) in slots {
        if let Some(file) = facts.get_mut(&path) {
            let site = &mut file.job_option_sites[index];
            site.queue_file = queue_file;
            site.queue_name = Some(queue_name);
        }
    }
}

fn imported_binding<'a>(file: &'a FileFacts, binding: &'a ProducerBinding) -> Option<&'a str> {
    match binding {
        ProducerBinding::Named(name) => Some(name),
        ProducerBinding::Local(declaration) => match file.queue_declarations.get(declaration)? {
            LocalQueue::ImportedQueue(name) => Some(name),
            _ => None,
        },
        ProducerBinding::Flow(_) | ProducerBinding::QueueName(_) | ProducerBinding::Unknown => None,
    }
}
