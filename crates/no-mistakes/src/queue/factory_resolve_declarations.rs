use super::{follow, FileFacts, ImportResolverFacade, LocalQueue, ProducerBinding, Update};
use crate::queue::test_scope::ConfiguredTestScope;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    root: &Path,
    test_scope: &ConfiguredTestScope,
) -> HashMap<String, PathBuf> {
    let mut paths = facts.keys().collect::<Vec<_>>();
    paths.sort();
    let (test_paths, prod_paths): (Vec<_>, Vec<_>) = paths
        .into_iter()
        .partition(|path| test_scope.is_match(root, path));
    let mut out = HashMap::new();
    let mut wrappers = std::collections::BTreeSet::new();
    // Module-level queue bindings in non-test files win canonical queue ownership
    // over function-local handles (such as prefixed test/local helpers) and test files.
    for group in [&prod_paths, &test_paths] {
        for path in group {
            for name in facts[*path].queue_bindings.values() {
                out.entry(name.clone()).or_insert_with(|| (*path).clone());
            }
        }
        for path in group {
            let file = &facts[*path];
            for name in defined_queue_names(file) {
                out.entry(name.to_string())
                    .or_insert_with(|| (*path).clone());
            }
            for producer in &file.producers {
                match &producer.binding {
                    ProducerBinding::Flow(name) => {
                        out.entry(name.clone()).or_insert_with(|| (*path).clone());
                    }
                    ProducerBinding::QueueName(name) => {
                        wrappers.insert(name.clone());
                    }
                    _ => {}
                }
            }
        }
    }
    for name in wrappers {
        out.entry(name.clone())
            .or_insert_with(|| PathBuf::from(format!("queue:{name}")));
    }
    out
}

pub(super) fn star_and_job_updates<R: ImportResolverFacade>(
    facts: &std::collections::HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Vec<Update> {
    let mut updates = Vec::new();
    for (path, file) in facts {
        for specifier in &file.star_exports {
            let Some(resolved) = resolver
                .resolve(specifier, path)
                .and_then(|resolved| remapper.remap(&resolved))
            else {
                continue;
            };
            let other = &facts[&resolved];
            for (name, queue_name) in &other.queue_exports {
                if file.queue_exports.contains_key(name) {
                    continue;
                }
                updates.push(Update::Export {
                    file: path.clone(),
                    exported: name.clone(),
                    queue_name: queue_name.clone(),
                });
            }
        }
        for (index, producer) in file.producers.iter().enumerate() {
            if producer.job.is_some() {
                continue;
            }
            let Some(local) = &producer.job_import else {
                continue;
            };
            let Some(job) = super::follow::lookup(
                facts,
                resolver,
                remapper,
                path,
                local,
                super::follow::Want::Either,
            ) else {
                continue;
            };
            updates.push(Update::ProducerJob {
                file: path.clone(),
                index,
                job,
            });
        }
    }
    updates
}
