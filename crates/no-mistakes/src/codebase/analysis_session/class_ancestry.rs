//! Request-owned declaration facts used exclusively by canonical class ancestry.
use super::*;
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::{ClassDeclaration, NodeId, TsFactLookup};
use crate::codebase::ts_resolver::{ImportResolution, TsConfigCatalog};
use crate::codebase::ts_source::facts::{TsFactContext, TsFactPlan, TsFileFacts};
use crate::fx::{FxHashMap, FxHashSet};
mod namespaces;
mod package;
mod resolution;

#[derive(Clone, Debug, Default)]
pub(crate) struct ClassAncestry {
    pub roots: FxHashSet<NodeId>,
    pub targets: FxHashMap<(PathBuf, CallableId), NodeId>,
    pub classes: Vec<ClassDeclaration>,
    pub expressions: FxHashMap<(PathBuf, u32), String>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct ProjectionKey {
    root: PathBuf,
    paths: Vec<PathBuf>,
    owners: Vec<usize>,
    scopes: Vec<ResolverCacheScopeKey>,
    visible: ResolverCacheScopeKey,
    resolve_bases: bool,
    catalog: Option<crate::codebase::ts_resolver::CatalogProjectionKey>,
}

pub(crate) struct AncestryRequest<'a> {
    pub root: &'a Path,
    pub resolve_bases: bool,
    pub paths: &'a [PathBuf],
    pub facts: &'a dyn TsFactLookup,
    pub resolver: &'a dyn ImportResolution,
    pub tsconfig: &'a TsConfig,
    pub catalog: Option<&'a TsConfigCatalog>,
}

impl AnalysisSession {
    pub(crate) fn class_ancestry(&self, request: AncestryRequest<'_>) -> Arc<ClassAncestry> {
        let mut scopes = Vec::new();
        let mut owners = Vec::new();
        let mut by_owner = FxHashMap::default();
        for path in request.paths {
            let owner = request
                .catalog
                .and_then(|catalog| catalog.resolver_scope_index_for(path));
            let index = *by_owner.entry(owner).or_insert_with(|| {
                let (config, mode, identity) = request
                    .catalog
                    .map_or((request.tsconfig, None, &[][..]), |catalog| {
                        catalog.resolver_scope_at(owner)
                    });
                let index = scopes.len();
                scopes.push(ResolverCacheScopeKey::for_lookup(
                    config, None, mode, identity,
                ));
                index
            });
            owners.push(index);
        }
        let key = ProjectionKey {
            root: request.root.to_path_buf(),
            paths: request.paths.to_vec(),
            owners,
            scopes,
            resolve_bases: request.resolve_bases,
            catalog: request.catalog.map(TsConfigCatalog::projection_key),
            // Snapshot legacy visibility once, never once per source file.
            visible: ResolverCacheScopeKey::for_lookup(
                request.tsconfig,
                request.resolver.visible_files(),
                None,
                &[],
            ),
        };
        let cell = self
            .class_ancestry_projections
            .entry(key)
            .or_default()
            .clone();
        Arc::clone(cell.get_or_init(|| Arc::new(resolution::prepare(self, request))))
    }

    fn declaration_facts(&self, root: &Path, path: &Path) -> Arc<TsFileFacts> {
        let cell = self
            .class_declaration_facts
            .entry(path.to_path_buf())
            .or_default()
            .clone();
        Arc::clone(cell.get_or_init(|| {
            let sources = self.existing_sources_for(root).unwrap_or_else(|| self.supplemental_sources.clone());
            let facts = crate::codebase::ts_source::facts::collect_ts_facts_with_context_sources_and_session(
                self, &[path.to_path_buf()], TsFactPlan { imports: true, function_calls: true, ..Default::default() },
                &TsFactContext { root: root.to_path_buf(), ..Default::default() }, &sources,
            );
            Arc::new(facts.get(path).cloned().unwrap_or_default())
        }))
    }
}
