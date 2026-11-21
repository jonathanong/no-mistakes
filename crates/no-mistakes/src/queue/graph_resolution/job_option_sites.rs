use super::local_queues;
use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::queue::extract::FileFacts;
use crate::queue::types::QueueKey;
use std::collections::HashMap;
use std::path::PathBuf;

pub(in crate::queue) fn resolve_job_option_sites<R: ImportResolverFacade>(
    facts: &mut HashMap<PathBuf, FileFacts>,
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
    definitions: &HashMap<String, PathBuf>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) {
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
                    site.binding
                        .as_ref()
                        .and_then(|binding| local.get(binding).cloned())
                });
            let binding = key.is_none().then(|| site.binding.clone()).flatten();
            slots.push((path.clone(), index, key, binding));
        }
    }
    for slot in &mut slots {
        if slot.2.is_some() {
            continue;
        }
        let Some(binding) = slot.3.as_deref() else {
            continue;
        };
        slot.2 = crate::queue::factory_resolve::binding_queue(
            facts,
            resolver,
            remapper,
            &slot.0,
            binding,
            definitions,
        );
    }
    for (path, index, key, _) in slots {
        let Some(key) = key else {
            continue;
        };
        let queue_file = crate::queue::job_options::defined_queue_file(facts, &key.queue_name)
            .unwrap_or(key.queue_file);
        if let Some(file) = facts.get_mut(&path) {
            let site = &mut file.job_option_sites[index];
            site.queue_file = Some(queue_file);
            site.queue_name = Some(key.queue_name);
        }
    }
}
