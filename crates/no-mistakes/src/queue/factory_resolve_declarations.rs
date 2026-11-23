use super::{follow, FileFacts, ImportResolverFacade, LocalQueue, ProducerBinding};
use std::collections::HashMap;
use std::path::PathBuf;

pub(super) fn resolve_names<R: ImportResolverFacade>(
    facts: &mut HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) {
    let mut updates = Vec::new();
    for (path, file) in facts.iter() {
        for (&declaration, queue) in &file.queue_declarations {
            if let LocalQueue::ImportedName(imported) = queue {
                if let Some(name) = follow::lookup(
                    facts,
                    resolver,
                    remapper,
                    path,
                    imported,
                    follow::Want::Either,
                ) {
                    updates.push((path.clone(), declaration, name));
                }
            }
        }
    }
    for (path, declaration, name) in updates {
        if let Some(file) = facts.get_mut(&path) {
            file.queue_declarations
                .insert(declaration, LocalQueue::Known(name));
        }
    }
}

pub(in crate::queue) fn defined_queue_names(file: &FileFacts) -> impl Iterator<Item = &str> {
    file.queue_bindings.values().map(String::as_str).chain(
        file.queue_declarations
            .values()
            .filter_map(|queue| match queue {
                LocalQueue::Known(name) => Some(name.as_str()),
                _ => None,
            }),
    )
}

pub(in crate::queue) fn definition_files(
    facts: &HashMap<PathBuf, FileFacts>,
) -> HashMap<String, PathBuf> {
    let mut paths = facts.keys().collect::<Vec<_>>();
    paths.sort();
    let mut out = HashMap::new();
    for path in paths {
        let file = &facts[path];
        let flows = file
            .producers
            .iter()
            .filter_map(|producer| match &producer.binding {
                ProducerBinding::Flow(name) => Some(name.as_str()),
                _ => None,
            });
        for name in defined_queue_names(file).chain(flows) {
            out.entry(name.to_string()).or_insert_with(|| path.clone());
        }
    }
    out
}
