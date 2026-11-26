use crate::queue::extract::FileFacts;
use crate::queue::extract_model::ProducerBinding;
use crate::queue::graph_model::InternalProducer;
use crate::queue::test_scope::ConfiguredTestScope;
use crate::queue::types::QueueKey;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) fn flow_queue_definitions<'a>(
    root: &Path,
    facts: &'a HashMap<PathBuf, FileFacts>,
    producers: &[&'a InternalProducer],
    test_scope: &ConfiguredTestScope,
) -> HashMap<&'a str, Option<&'a PathBuf>> {
    let flow_names = producers
        .iter()
        .filter_map(|producer| match &producer.site.binding {
            ProducerBinding::Flow(queue) => Some(queue.as_str()),
            _ => None,
        })
        .collect::<HashSet<_>>();
    let mut definitions = HashMap::new();
    if flow_names.is_empty() {
        return definitions;
    }
    for (path, file) in facts
        .iter()
        .filter(|(path, _)| !test_scope.is_match(root, path))
    {
        for queue in crate::queue::factory_resolve::defined_queue_names(file) {
            if flow_names.contains(queue) {
                definitions
                    .entry(queue)
                    .and_modify(|existing| {
                        if *existing != Some(path) {
                            *existing = None;
                        }
                    })
                    .or_insert(Some(path));
            }
        }
    }
    definitions
}

pub(super) fn resolved_flow_queue(
    queue: &QueueKey,
    definition: Option<&Option<&PathBuf>>,
) -> Option<QueueKey> {
    match definition {
        None => Some(queue.clone()),
        Some(path) => Some(QueueKey {
            queue_file: (*path)?.clone(),
            queue_name: queue.queue_name.clone(),
        }),
    }
}

pub(super) fn opaque_queues(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    flow_queues: &HashMap<&str, Option<&PathBuf>>,
    test_scope: &ConfiguredTestScope,
) -> HashSet<QueueKey> {
    facts
        .iter()
        .filter(|(path, _)| !test_scope.is_match(root, path))
        .flat_map(|(_, file)| &file.opaque_priority_queues)
        .filter_map(|(queue_file, queue_name)| {
            let queue = QueueKey {
                queue_file: queue_file.clone(),
                queue_name: queue_name.clone(),
            };
            resolved_flow_queue(&queue, flow_queues.get(queue_name.as_str()))
        })
        .collect()
}
