use crate::codebase::postgres::query_annotation::evaluate::Scope;
mod initials;
use super::{super::Environment, bindings};
use crate::codebase::postgres::query_annotation::module_ids::ModuleId;
use crate::fx::FxHashMap;
pub(in crate::codebase::postgres::query_annotation) use initials::Initials;

#[derive(Default)]
pub(super) struct States(FxHashMap<ModuleId, Scope>);
impl States {
    pub fn restore(&self, scopes: &mut [Scope], modules: &FxHashMap<ModuleId, Environment>) {
        for (path, values) in &self.0 {
            scopes[modules[path]].clone_from(values);
        }
    }
    pub fn remapped(&mut self, scopes: &[Scope], modules: &FxHashMap<ModuleId, Environment>) {
        for (path, values) in &mut self.0 {
            values.clone_from(&scopes[modules[path]]);
        }
    }
    pub fn join(
        &mut self,
        scopes: &[Scope],
        modules: &FxHashMap<ModuleId, Environment>,
        original: usize,
        whole_sql: bool,
    ) {
        for (path, env) in modules {
            if *env < original {
                continue;
            }
            match self.0.entry(*path) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(scopes[*env].clone());
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    let initial = vec![entry.get().clone()];
                    bindings::join_with_sql(
                        std::slice::from_mut(entry.get_mut()),
                        &scopes[*env..=*env],
                        &initial,
                        whole_sql,
                    );
                }
            }
        }
    }
}
