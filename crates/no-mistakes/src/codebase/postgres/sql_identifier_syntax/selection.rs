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
    pub advisory_receivers: FxHashSet<usize>,
    pub advisory_factory_defaults: FxHashSet<usize>,
    pub advisory_factory_members: FxHashMap<usize, FxHashSet<String>>,
}
impl Selection {
    pub fn retains_recovery(&self) -> bool {
        !self.selected.is_empty()
            || !self.namespaces.is_empty()
            || !self.factories.is_empty()
            || !self.transports.is_empty()
            || !self.runtime.is_empty()
            || !self.typed.is_empty()
            || !self.unsupported.is_empty()
            || !self.advisory_receivers.is_empty()
            || !self.advisory_factory_defaults.is_empty()
            || !self.advisory_factory_members.is_empty()
    }
    pub fn add_declared_factories(
        &mut self,
        profile: &EmbeddedSqlOptions,
        module: &TypeScriptModuleFacts,
        declared: &[crate::config::v2::schema::EffectTargetConfig],
    ) {
        for import in &module.imports {
            if import.type_only {
                continue;
            }
            let names = declared
                .iter()
                .filter(|target| {
                    target.module == import.specifier
                        && profile.executor_factory_names.contains(&target.export)
                })
                .map(|target| target.export.clone())
                .collect::<FxHashSet<_>>();
            if names.is_empty() {
                continue;
            }
            for binding in &import.bindings {
                if binding.type_only {
                    continue;
                }
                match binding.kind.as_str() {
                    "named" if names.contains(&binding.imported) => {
                        self.factories.insert(binding.binding_id);
                        self.unsupported.insert(binding.binding_id);
                    }
                    "default" => {
                        self.advisory_factory_defaults.insert(binding.binding_id);
                    }
                    "namespace" => {
                        self.advisory_factory_members
                            .insert(binding.binding_id, names.clone());
                    }
                    _ => {}
                }
            }
        }
    }
    pub fn new(profile: &EmbeddedSqlOptions, module: &TypeScriptModuleFacts) -> Self {
        let mut selected = FxHashSet::default();
        let mut namespaces = FxHashSet::default();
        let mut unsupported = FxHashSet::default();
        let mut advisory_receivers = FxHashSet::default();
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
                if exact
                    && !profile.import_specifier.is_empty()
                    && profile.executor_names.iter().any(|name| name == "query")
                    && !import.type_only
                    && !binding.type_only
                {
                    advisory_receivers.insert(binding.binding_id);
                }
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
            advisory_receivers,
            advisory_factory_defaults: FxHashSet::default(),
            advisory_factory_members: FxHashMap::default(),
        }
    }
}
