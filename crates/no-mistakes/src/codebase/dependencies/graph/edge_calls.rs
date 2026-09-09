/// Projects statically resolved lexical calls into the canonical graph.
///
/// This resolves same-file bindings, direct named and namespace imports, and
/// explicit or unambiguous-star re-export chains. Dynamic calls and computed
/// members deliberately remain absent: connecting those by terminal name would
/// make call traversal unsound.
///
/// Aliases and callable bindings are indexed once per file by lexical
/// `(binding scope, local name)`. `resolve_alias` is the cycle-safe resolver
/// shared with import reachability for plain, dotted, class, import, and
/// deferred aliases. It honors `declared_at` (TDZ) and `invalidated_at`.
/// Class-member maps are built separately by `index_class_members_by_id`.
#[derive(Clone)]
struct CallableFileIndex {
    known_scopes: FxHashSet<String>,
    exported_scopes: FxHashSet<String>,
    class_scopes: FxHashSet<String>,
    callable_bindings: FxHashMap<(usize, String), crate::codebase::dependencies::extract::CallableId>,
    imported: FxHashMap<String, crate::codebase::dependencies::extract::ImportedBinding>,
    exported: FxHashMap<String, crate::codebase::dependencies::extract::ExportedBinding>,
    aliases: FxHashMap<(usize, String), IndexedAlias>,
    binding_declared_at: FxHashMap<(usize, String), u32>,
    invocation_offsets:
        FxHashMap<crate::codebase::dependencies::extract::CallableId, Vec<u32>>,
    /// Class bindings resolve to their internal class scope and exact parser
    /// identity. A display scope can repeat in sibling blocks.
    class_bindings: FxHashMap<(usize, String), ClassBindingTarget>,
    lexical_scope_parents: FxHashMap<usize, Option<usize>>,
    scope_ids_by_display: FxHashMap<String, Vec<crate::codebase::dependencies::extract::CallableId>>,
    stars: Vec<String>,
}

#[derive(Clone)]
struct IndexedAlias {
    target: String,
    declared_at: u32,
    invalidated_at: Option<u32>,
}

#[derive(Clone)]
struct ClassBindingTarget {
    scope: String,
    class_id: crate::codebase::dependencies::extract::CallableId,
    static_member_ids:
        FxHashMap<String, crate::codebase::dependencies::extract::CallableId>,
    /// A simple local `extends Base` relationship. Imported, computed, and
    /// expression bases intentionally stay unresolved here.
    local_base: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedLocalCallee {
    callee: String,
    callable_id: Option<crate::codebase::dependencies::extract::CallableId>,
}

include!("edge_calls/index_build.rs");

impl CallableFileIndex {
    fn from_facts(file: &crate::codebase::ts_source::facts::TsFileFacts) -> Self {
        let class_scope_by_id = file
            .callable_scope_ids
            .iter()
            .filter(|(_, scope)| file.class_scopes.contains(scope))
            .map(|(id, scope)| (*id, scope.clone()))
            .collect::<FxHashMap<_, _>>();
        let members_by_class = index_class_members_by_id(&file.class_member_callable_ids);
        let local_bases = index_local_construct_bases(&file.function_calls);
        let callable_bindings = file
            .callable_bindings
            .iter()
            .map(|(scope, binding, id)| ((*scope, binding.clone()), *id))
            .collect::<FxHashMap<_, _>>();
        let invocation_offsets =
            invocation_offsets_from_bindings(&callable_bindings, &file.function_calls);
        Self {
            known_scopes: file.callable_scopes.iter().cloned().collect(),
            exported_scopes: file.exported_functions.iter().cloned().collect(),
            class_scopes: file.class_scopes.iter().cloned().collect(),
            callable_bindings,
            imported: file
                .imported_bindings
                .iter()
                .filter(|binding| !binding.is_type_only)
                .map(|binding| (binding.local.clone(), binding.clone()))
                .collect(),
            exported: file
                .exported_bindings
                .iter()
                .map(|binding| (binding.exported.clone(), binding.clone()))
                .collect(),
            aliases: index_callable_aliases(&file.callable_aliases),
            binding_declared_at: index_binding_declared_at(&file.callable_binding_declared_at),
            invocation_offsets,
            class_bindings: file
                .callable_bindings
                .iter()
                .filter_map(|(scope, binding, id)| {
                    class_scope_by_id.get(id).map(|class_scope| {
                        (
                            (*scope, binding.clone()),
                            ClassBindingTarget {
                                scope: class_scope.clone(),
                                class_id: *id,
                                static_member_ids: members_by_class
                                    .get(id)
                                    .cloned()
                                    .unwrap_or_default(),
                                local_base: local_bases.get(id).cloned(),
                            },
                        )
                    })
                })
                .collect(),
            lexical_scope_parents: file.lexical_scope_parents.iter().copied().collect(),
            scope_ids_by_display: index_scope_ids_by_display(&file.callable_scope_ids),
            stars: file.star_reexport_specifiers.clone(),
        }
    }
}

include!("edge_calls/class_resolution.rs");
include!("edge_calls/alias_resolution.rs");
include!("edge_calls/alias_liveness.rs");
include!("edge_calls/traversal_filter.rs");

#[derive(Clone, Default)]
struct CallableResolutionIndexes {
    files: dashmap::DashMap<std::path::PathBuf, std::sync::Arc<CallableFileIndex>>,
    exports: dashmap::DashMap<(std::path::PathBuf, String), ExportedCallableResolution>,
}

/// An `export *` branch can prove that a name is exported without proving it
/// is callable. Keep that distinct from an absent name so a callable branch
/// cannot win over a non-callable collision in another barrel.
#[derive(Clone)]
enum ExportedCallableResolution {
    Absent,
    Callable(
        std::path::PathBuf,
        String,
        Option<crate::codebase::dependencies::extract::CallableId>,
    ),
    ExternalModuleExport(String, String),
    Unknown,
}

impl CallableResolutionIndexes {
    fn file(
        &self,
        facts: &dyn TsFactLookup,
        path: &std::path::Path,
    ) -> Option<std::sync::Arc<CallableFileIndex>> {
        if let Some(index) = self.files.get(path) {
            return Some(index.clone());
        }
        let index = std::sync::Arc::new(CallableFileIndex::from_facts(facts.get_ts_facts(path)?));
        self.files.insert(path.to_path_buf(), index.clone());
        Some(index)
    }
}

include!("edge_calls/collection.rs");
include!("edge_calls/types.rs");
include!("edge_calls/roots.rs");
include!("edge_calls/import_resolution.rs");
include!("edge_calls/export_resolution.rs");
include!("edge_calls/local_resolution.rs");
include!("edge_calls/site_index.rs");
#[cfg(feature = "test-instrumentation")]
include!("edge_calls/bench.rs");
