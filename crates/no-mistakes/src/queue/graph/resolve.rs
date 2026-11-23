use crate::queue::extract::FileFacts;
use crate::queue::graph_model::{InternalProducer, InternalWorker};
use crate::queue::graph_resolution::{queue_definitions, resolve_producers, resolve_workers};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(in crate::queue) fn resolve_queue_relationships_with_resolver<
    T,
    R: crate::codebase::ts_resolver::ImportResolverFacade,
>(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    builder: impl FnOnce(
        &Path,
        Vec<InternalProducer>,
        Vec<InternalWorker>,
        &HashMap<PathBuf, FileFacts>,
    ) -> T,
) -> T {
    let mut facts = facts.clone();
    let remapper =
        crate::codebase::ts_source::FrozenPathRemapper::from_paths(facts.keys().cloned());
    crate::queue::factory_resolve::resolve_imported_queue_names(&mut facts, resolver, &remapper);
    let queue_defs = queue_definitions(&facts);
    let definitions = crate::queue::factory_resolve::definition_files(&facts);
    let producers = resolve_producers(&mut facts, &queue_defs, &definitions, resolver, &remapper);
    let workers = resolve_workers(&facts, &definitions, resolver, &remapper);
    builder(root, producers, workers, &facts)
}
