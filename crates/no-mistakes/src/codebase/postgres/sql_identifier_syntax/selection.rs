use super::*;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
#[derive(Default)]
pub(super) struct Selection {
    pub selected: FxHashSet<usize>,
    pub namespaces: FxHashSet<usize>,
    pub factories: FxHashSet<usize>,
    pub transports: FxHashSet<Span>,
    pub runtime: FxHashSet<Span>,
    pub typed: FxHashSet<usize>,
    pub unsupported: FxHashSet<usize>,
}
impl Selection {
    pub fn new(profile: &EmbeddedSqlOptions, module: &TypeScriptModuleFacts) -> Self {
        let mut selected = FxHashSet::default();
        let mut namespaces = FxHashSet::default();
        let mut unsupported = FxHashSet::default();
        let mut factories = FxHashSet::default();
        for import in &module.imports {
            let exact =
                profile.import_specifier.is_empty() || import.specifier == profile.import_specifier;
            let scoped = exact
                || import
                    .specifier
                    .starts_with(&format!("{}/", profile.import_specifier));
            if !scoped {
                continue;
            }
            for binding in &import.bindings {
                if !profile.import_specifier.is_empty()
                    && profile.executor_names.iter().any(|name| name == "query")
                    && binding.kind == "namespace"
                    && !binding.type_only
                {
                    namespaces.insert(binding.binding_id);
                }
                if exact
                    && !binding.type_only
                    && binding.kind == "named"
                    && profile.executor_names.contains(&binding.imported)
                {
                    selected.insert(binding.binding_id);
                }
                if exact
                    && !binding.type_only
                    && !profile.import_specifier.is_empty()
                    && ["withTransaction", "withTransactionOptions"]
                        .contains(&binding.imported.as_str())
                {
                    unsupported.insert(binding.binding_id);
                }
                if (!binding.type_only
                    && profile.executor_factory_names.contains(&binding.imported))
                    || profile.executor_type_names.contains(&binding.imported)
                {
                    unsupported.insert(binding.binding_id);
                    if !binding.type_only
                        && profile.executor_factory_names.contains(&binding.imported)
                    {
                        factories.insert(binding.binding_id);
                    }
                }
            }
        }
        Self {
            selected,
            namespaces,
            factories,
            transports: FxHashSet::default(),
            typed: FxHashSet::default(),
            runtime: FxHashSet::default(),
            unsupported,
        }
    }
}
