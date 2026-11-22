use crate::queue::extract::FileFacts;
use crate::queue::graph_model::InternalProducer;
use crate::queue::types::QueueKey;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub(super) fn flow_queue_definitions<'a>(
    facts: &'a HashMap<PathBuf, FileFacts>,
    producers: &'a [InternalProducer],
) -> HashMap<&'a str, Option<&'a PathBuf>> {
    let flow_names = producers
        .iter()
        .filter_map(|producer| producer.site.binding.strip_prefix("__flow__"))
        .collect::<HashSet<_>>();
    let mut definitions = HashMap::new();
    if flow_names.is_empty() {
        return definitions;
    }
    for (path, file) in facts {
        for (binding, queue) in &file.queue_bindings {
            if !binding.starts_with("__flow__") && flow_names.contains(queue.as_str()) {
                definitions
                    .entry(queue.as_str())
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
    facts: &HashMap<PathBuf, FileFacts>,
    flow_queues: &HashMap<&str, Option<&PathBuf>>,
) -> HashSet<QueueKey> {
    facts
        .values()
        .flat_map(|file| &file.opaque_priority_queues)
        .filter_map(|(queue_file, queue_name)| {
            let queue = QueueKey {
                queue_file: queue_file.clone(),
                queue_name: queue_name.clone(),
            };
            resolved_flow_queue(&queue, flow_queues.get(queue_name.as_str()))
        })
        .collect()
}
