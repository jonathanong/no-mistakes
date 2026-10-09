mod initials;
use super::{
    super::{Environment, Value},
    bindings,
};
use crate::fx::FxHashMap;
pub(in crate::codebase::postgres::query_annotation) use initials::Initials;
use std::path::PathBuf;

#[derive(Default)]
pub(super) struct States(FxHashMap<PathBuf, FxHashMap<String, Value>>);
impl States {
    pub fn restore(
        &self,
        scopes: &mut [FxHashMap<String, Value>],
        modules: &FxHashMap<PathBuf, Environment>,
    ) {
        for (path, values) in &self.0 {
            scopes[modules[path]].clone_from(values);
        }
    }
    pub fn remapped(
        &mut self,
        scopes: &[FxHashMap<String, Value>],
        modules: &FxHashMap<PathBuf, Environment>,
    ) {
        for (path, values) in &mut self.0 {
            values.clone_from(&scopes[modules[path]]);
        }
    }
    pub fn join(
        &mut self,
        scopes: &[FxHashMap<String, Value>],
        modules: &FxHashMap<PathBuf, Environment>,
        original: usize,
    ) {
        for (path, env) in modules {
            if *env < original {
                continue;
            }
            match self.0.entry(path.clone()) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(scopes[*env].clone());
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    let initial = vec![entry.get().clone()];
                    bindings::join(
                        std::slice::from_mut(entry.get_mut()),
                        &scopes[*env..=*env],
                        &initial,
                    );
                }
            }
        }
    }
}
