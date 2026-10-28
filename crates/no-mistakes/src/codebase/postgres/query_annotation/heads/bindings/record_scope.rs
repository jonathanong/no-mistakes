use super::Bindings;
use crate::codebase::ts_source::module_facts::ModuleBinding;

impl Bindings<'_> {
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
