/// Projects statically resolved lexical calls into the canonical graph.
///
/// This resolves same-file bindings, direct named and namespace imports, and
/// explicit or unambiguous-star re-export chains. Dynamic calls and computed
/// members deliberately remain absent: connecting those by terminal name would
/// make call traversal unsound.
#[derive(Clone)]
struct CallableFileIndex {
    known_scopes: std::collections::HashSet<String>,
    imported:
        std::collections::HashMap<String, crate::codebase::dependencies::extract::ImportedBinding>,
    exported:
        std::collections::HashMap<String, crate::codebase::dependencies::extract::ExportedBinding>,
    aliases: std::collections::HashMap<(Option<String>, String), String>,
    stars: Vec<String>,
}

impl CallableFileIndex {
    fn from_facts(file: &crate::codebase::ts_source::facts::TsFileFacts) -> Self {
        Self {
            known_scopes: file.callable_scopes.iter().cloned().collect(),
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
                        (alias.scope.clone(), alias.local.clone()),
                        alias.target.clone(),
                    )
                })
                .collect(),
            stars: file.star_reexport_specifiers.clone(),
        }
    }

    fn resolve_alias(&self, caller: Option<&str>, callee: &str) -> Option<String> {
        if callee.contains('.') {
            return None;
        }
        let mut scope = caller.map(str::to_string);
        let mut visited = std::collections::HashSet::new();
        let mut target = callee.to_string();
        loop {
            let key = (scope.clone(), target.clone());
            let alias = self.aliases.get(&key)?;
            if !visited.insert(key) {
                return None;
            }
            target = alias.clone();
            if self.aliases.contains_key(&(scope.clone(), target.clone())) {
                continue;
            }
            if target.contains('.')
                || self.imported.contains_key(&target)
                || resolve_local_call_scope(scope.as_deref(), &target, &self.known_scopes).is_some()
            {
                return Some(target);
            }
            if let Some(parent) = scope
                .as_deref()
                .and_then(|current| current.rsplit_once('/').map(|(parent, _)| parent))
            {
                scope = Some(parent.to_string());
            } else if scope.is_some() {
                scope = None;
            } else {
                return None;
            }
        }
    }
}

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
    Unknown,
}

impl ExportedCallableResolution {
    fn callable(self) -> Option<(std::path::PathBuf, String)> {
        match self {
            Self::Callable(path, scope) => Some((path, scope)),
            Self::Absent | Self::Unknown => None,
        }
    }
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
