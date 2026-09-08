/// Projects statically resolved lexical calls into the canonical graph.
///
/// This resolves same-file bindings, direct named and namespace imports, and
/// explicit or unambiguous-star re-export chains. Dynamic calls and computed
/// members deliberately remain absent: connecting those by terminal name would
/// make call traversal unsound.
#[derive(Clone)]
struct CallableFileIndex {
    known_scopes: std::collections::HashSet<String>,
    class_scopes: std::collections::HashSet<String>,
    imported:
        std::collections::HashMap<String, crate::codebase::dependencies::extract::ImportedBinding>,
    exported:
        std::collections::HashMap<String, crate::codebase::dependencies::extract::ExportedBinding>,
    aliases: std::collections::HashMap<(usize, String), String>,
    /// Class bindings resolve to their internal class scope and exact parser
    /// identity. A display scope can repeat in sibling blocks.
    class_bindings: std::collections::HashMap<(usize, String), ClassBindingTarget>,
    lexical_scope_parents: std::collections::HashMap<usize, Option<usize>>,
    stars: Vec<String>,
}

#[derive(Clone)]
struct ClassBindingTarget {
    scope: String,
    class_id: crate::codebase::dependencies::extract::CallableId,
    static_member_ids: std::collections::HashMap<String, crate::codebase::dependencies::extract::CallableId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedLocalCallee {
    callee: String,
    callable_id: Option<crate::codebase::dependencies::extract::CallableId>,
}

impl CallableFileIndex {
    fn from_facts(file: &crate::codebase::ts_source::facts::TsFileFacts) -> Self {
        let class_scope_by_id = file
            .callable_scope_ids
            .iter()
            .filter(|(_, scope)| file.class_scopes.contains(scope))
            .map(|(id, scope)| (*id, scope.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        Self {
            known_scopes: file.callable_scopes.iter().cloned().collect(),
            class_scopes: file.class_scopes.iter().cloned().collect(),
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
            aliases: file
                .callable_aliases
                .iter()
                .map(|alias| {
                    (
                        (alias.binding_scope, alias.local.clone()),
                        alias.target.clone(),
                    )
                })
                .collect(),
            class_bindings: file
                .callable_bindings
                .iter()
                .filter_map(|(scope, binding, id)| {
                    class_scope_by_id
                        .get(id)
                        .map(|class_scope| {
                            let static_member_ids = file
                                .class_member_callable_ids
                                .iter()
                                .filter(|(candidate_class_id, _, _)| *candidate_class_id == *id)
                                .map(|(_, member, member_id)| (member.clone(), *member_id))
                                .collect();
                            ((*scope, binding.clone()), ClassBindingTarget {
                                scope: class_scope.clone(),
                                class_id: *id,
                                static_member_ids,
                            })
                        })
                })
                .collect(),
            lexical_scope_parents: file.lexical_scope_parents.iter().copied().collect(),
            stars: file.star_reexport_specifiers.clone(),
        }
    }

    fn resolve_class_binding(&self, binding_scope: Option<usize>, callee: &str) -> Option<ResolvedLocalCallee> {
        let binding_scope = binding_scope?;
        let (binding, member) = callee
            .split_once('.')
            .map_or((callee, None), |(binding, member)| (binding, Some(member)));
        let scope = self
            .class_bindings
            .get(&(binding_scope, binding.to_string()))?;
        let callable_id = match member {
            Some(member) => scope.static_member_ids.get(member).copied()?,
            None => scope.class_id,
        };
        Some(ResolvedLocalCallee {
            callee: member.map_or_else(
                || scope.scope.clone(),
                |member| format!("{}.{}", scope.scope, member),
            ),
            callable_id: Some(callable_id),
        })
    }

}

include!("edge_calls/alias_resolution.rs");

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
    Callable(std::path::PathBuf, String),
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
