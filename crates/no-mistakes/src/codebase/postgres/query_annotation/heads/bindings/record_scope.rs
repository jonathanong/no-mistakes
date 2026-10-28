use super::Bindings;
use crate::codebase::ts_source::module_facts::ModuleBinding;
use crate::fx::FxHashSet;

#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation::heads) struct ExportIndex {
    root_scopes: FxHashSet<usize>,
    names: FxHashSet<String>,
}

impl ExportIndex {
    pub(in crate::codebase::postgres::query_annotation::heads) fn contains(
        &self,
        owner: &ModuleBinding,
    ) -> bool {
        self.root_scopes.contains(&owner.scope_id) && self.names.contains(&owner.name)
    }
}

impl Bindings<'_> {
    pub(in crate::codebase::postgres::query_annotation::heads) fn export_index(
        &self,
    ) -> ExportIndex {
        let mut index = ExportIndex::default();
        if let Some(module) = self.module {
            if let Some(observer) = crate::diagnostics::current() {
                observer.increment("postgres.annotation.primitive_export_indexes", 1);
                observer.increment(
                    "postgres.annotation.primitive_export_entries_visited",
                    (module.scopes.len() + module.exports.len()) as u64,
                );
            }
            index.root_scopes.extend(
                module
                    .scopes
                    .iter()
                    .filter(|scope| scope.parent_id.is_none())
                    .map(|scope| scope.id),
            );
            index.names.extend(
                module
                    .exports
                    .iter()
                    .filter(|export| !export.type_only && export.specifier.is_none())
                    .map(|export| export.local.clone()),
            );
        }
        index
    }
    pub(in crate::codebase::postgres::query_annotation::heads) fn exported(
        &self,
        owner: &ModuleBinding,
    ) -> bool {
        self.module.is_some_and(|module| {
            let root = module
                .scopes
                .iter()
                .any(|scope| scope.id == owner.scope_id && scope.parent_id.is_none());
            root && module.exports.iter().any(|export| {
                !export.type_only && export.specifier.is_none() && export.local == owner.name
            })
        })
    }
}
