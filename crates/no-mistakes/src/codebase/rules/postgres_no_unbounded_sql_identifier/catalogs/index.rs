use super::*;
use crate::codebase::dependencies::extract::{
    CatalogAccessor, CatalogRead, ImportedBindingKind, ValueParameter,
};

pub(super) struct Index<'a> {
    pub(super) sources: FxHashMap<Source<'a>, &'a LiteralCatalog>,
    pub(super) source_helpers: FxHashMap<Source<'a>, Vec<Helper<'a>>>,
    pub(super) accessors: FxHashMap<Helper<'a>, Vec<&'a CatalogAccessor>>,
    pub(super) flows: FxHashMap<&'a Path, &'a ValueFlowFacts>,
    pub(super) reads: FxHashMap<Source<'a>, Vec<(u32, &'a CatalogRead)>>,
    pub(super) mutated: FxHashSet<Source<'a>>,
    pub(super) calls: FxHashMap<Call<'a>, &'a ValueCall>,
    pub(super) incoming: FxHashMap<Helper<'a>, Vec<Call<'a>>>,
    pub(super) results: FxHashMap<Call<'a>, Vec<&'a ValueBinding>>,
    pub(super) returns: FxHashMap<Helper<'a>, Vec<&'a ValueExpression>>,
    pub(super) bad_helpers: FxHashSet<Helper<'a>>,
    pub(super) bindings: FxHashMap<Source<'a>, Vec<&'a ValueBinding>>,
    pub(super) parameters: FxHashMap<Helper<'a>, Vec<&'a ValueParameter>>,
    pub(super) names: FxHashMap<Source<'a>, Helper<'a>>,
    pub(super) imports: FxHashSet<Source<'a>>,
    pub(super) exports: FxHashSet<Source<'a>>,
    pub(super) namespaces: FxHashSet<Source<'a>>,
    source_writes: FxHashSet<Source<'a>>,
    modules: FxHashSet<&'a Path>,
}

impl<'a> Index<'a> {
    pub(super) fn new(graph: &'a DepGraph, facts: &'a CheckFactMap) -> Self {
        let mut index = Self {
            sources: FxHashMap::default(),
            source_helpers: FxHashMap::default(),
            accessors: FxHashMap::default(),
            flows: FxHashMap::default(),
            reads: FxHashMap::default(),
            mutated: FxHashSet::default(),
            calls: FxHashMap::default(),
            incoming: FxHashMap::default(),
            results: FxHashMap::default(),
            returns: FxHashMap::default(),
            bad_helpers: FxHashSet::default(),
            bindings: FxHashMap::default(),
            parameters: FxHashMap::default(),
            names: FxHashMap::default(),
            imports: FxHashSet::default(),
            exports: FxHashSet::default(),
            namespaces: FxHashSet::default(),
            source_writes: FxHashSet::default(),
            modules: FxHashSet::default(),
        };
        for file in facts.graph_file_universe() {
            let file = file.as_path();
            let Some(ts) = facts.get_ts_facts(file) else {
                continue;
            };
            if ts.parse_error.is_some()
                || ts.operational_error.is_some()
                || ts.value_flow.incomplete_reason.is_some()
            {
                continue;
            }
            let flow = &ts.value_flow;
            index.flows.insert(file, flow);
            if !ts.imports.is_empty()
                || !ts.exported_bindings.is_empty()
                || !ts.star_reexport_specifiers.is_empty()
            {
                index.modules.insert(file);
            }
            for export in &ts.exported_bindings {
                if export.specifier.is_none() {
                    index.source_writes.insert((file, 0, &export.local));
                    index.exports.insert((file, 0, &export.local));
                }
            }
            for (scope, name, owner) in &ts.callable_bindings {
                index.names.insert((file, *scope, name), (file, *owner));
            }
            for import in &ts.imported_bindings {
                if !import.is_type_only {
                    index.imports.insert((file, 0, &import.local));
                    if import.kind == ImportedBindingKind::Namespace {
                        index.namespaces.insert((file, 0, &import.local));
                    }
                }
            }
            for parameter in &flow.parameters {
                index
                    .parameters
                    .entry((file, parameter.owner))
                    .or_default()
                    .push(parameter);
            }
            for catalog in &flow.catalogs.sources {
                index
                    .sources
                    .insert((file, catalog.scope, &catalog.name), catalog);
            }
            for accessor in &flow.catalogs.accessors {
                index
                    .accessors
                    .entry((file, accessor.owner))
                    .or_default()
                    .push(accessor);
                let helpers = index
                    .source_helpers
                    .entry((file, accessor.scope, &accessor.source))
                    .or_default();
                if !helpers.contains(&(file, accessor.owner)) {
                    helpers.push((file, accessor.owner));
                }
            }
            for (at, read) in &flow.catalogs.reads {
                index
                    .reads
                    .entry((file, read.scope, &read.name))
                    .or_default()
                    .push((*at, read));
            }
            for write in &flow.identifier_writes {
                index.mutated.insert((file, write.scope, &write.name));
                index.source_writes.insert((file, write.scope, &write.name));
            }
            for binding in &flow.bindings {
                index
                    .bindings
                    .entry((file, binding.scope, &binding.name))
                    .or_default()
                    .push(binding);
                if let ValueExpression::Call(start, end) = &binding.value {
                    index
                        .results
                        .entry((file, *start, *end))
                        .or_default()
                        .push(binding);
                }
            }
            for call in &flow.calls {
                index.calls.insert((file, call.offset, call.end), call);
            }
            for (owner, value) in &flow.returns {
                index.returns.entry((file, *owner)).or_default().push(value);
            }
        }
        index.index_calls(graph, facts);
        index
    }

    pub(super) fn source_safe(&self, source: Source<'_>, catalog: &LiteralCatalog) -> bool {
        if self.source_writes.contains(&source) || source.1 == 0 && !self.modules.contains(source.0)
        {
            return false;
        }
        let Some(bindings) = self.bindings.get(&source) else {
            return false;
        };
        let [binding] = bindings.as_slice() else {
            return false;
        };
        binding.immutable
            && binding.invalidated_at.is_none()
            && binding.declared_at == catalog.declared_at
            && self.reads.get(&source).is_none_or(|reads| {
                reads.iter().all(|(_, read)| match read.kind {
                    CatalogReadKind::Value => catalog.paths.contains(&read.path),
                    CatalogReadKind::Collection(start, end) => {
                        !catalog.record
                            && read.path.is_empty()
                            && super::pipelines::collection_safe(
                                self, source.0, catalog, start, end,
                            )
                    }
                    CatalogReadKind::Lookup(owner) => {
                        catalog.record
                            && self
                                .accessors
                                .get(&(source.0, owner))
                                .is_some_and(|accessors| {
                                    accessors.iter().all(|accessor| {
                                        accessor.scope == source.1 && accessor.source == source.2
                                    })
                                })
                    }
                    _ => false,
                })
            })
    }
}
