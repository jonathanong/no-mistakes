use crate::queue::extract::FileFacts;
use crate::queue::graph_model::{InternalProducer, InternalWorker};
use crate::queue::graph_resolution::{queue_definitions, resolve_producers, resolve_workers};
use crate::queue::test_scope::ConfiguredTestScope;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(in crate::queue) fn resolve_queue_relationships_with_resolver<
    T,
    R: crate::codebase::ts_resolver::ImportResolverFacade,
>(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    test_scope: &ConfiguredTestScope,
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
    let definitions = crate::queue::factory_resolve::definition_files(&facts, root, test_scope);
    let producers = resolve_producers(&mut facts, &queue_defs, &definitions, resolver, &remapper);
    crate::queue::graph_resolution::resolve_job_option_sites(
        &mut facts,
        &queue_defs,
        &definitions,
        resolver,
        &remapper,
    );
    crate::queue::job_options::judge_into_facts(root, &mut facts, test_scope);
    crate::queue::worker_rate_limit::resolve(&mut facts, resolver, &remapper);
    crate::queue::job_state::resolve_sites(
        &mut facts,
        &queue_defs,
        &definitions,
        resolver,
        &remapper,
    );
    let workers = resolve_workers(&facts, &definitions, resolver, &remapper);
    builder(root, producers, workers, &facts)
}

pub(super) fn resolve_legacy_queue_relationships<T>(
    root: &Path,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    filter: Option<&globset::GlobSet>,
    shared: &crate::codebase::check_facts::CheckFactMap,
    builder: impl FnOnce(
        &Path,
        Vec<InternalProducer>,
        Vec<InternalWorker>,
        &HashMap<PathBuf, FileFacts>,
        &ConfiguredTestScope,
    ) -> T,
) -> T {
    let facts = super::queue_project_facts_from_shared(shared, filter, root);
    let visible_files: crate::fx::PathSet = shared.files().iter().cloned().collect();
    let base_resolver = super::queue_import_resolver(tsconfig, root, &visible_files);
    let workspace = crate::codebase::workspaces::load_indexed_from_files(root, shared.files())
        .unwrap_or_default();
    let resolver = crate::codebase::ts_resolver::WorkspaceFallbackResolver::new(
        &base_resolver,
        &workspace,
        &visible_files,
    );
    let test_scope = ConfiguredTestScope::empty();
    resolve_queue_relationships_with_resolver(
        root,
        &facts,
        &resolver,
        &test_scope,
        |root, producers, workers, facts| builder(root, producers, workers, facts, &test_scope),
    )
}
