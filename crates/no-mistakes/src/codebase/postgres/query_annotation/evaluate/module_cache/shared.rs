use super::super::value::Values;
use super::super::{Environment, Evaluator, Scope, Value};
use crate::codebase::postgres::query_annotation::module_ids::ModuleId;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub(in crate::codebase::postgres::query_annotation) struct CachedModule {
    pub(in crate::codebase::postgres::query_annotation) deps: Vec<ModuleId>,
    pub(in crate::codebase::postgres::query_annotation) dep_refs: Vec<(ModuleId, usize)>,
    pub(in crate::codebase::postgres::query_annotation) own_scopes: Vec<Scope>,
}

#[derive(Clone, Default)]
pub(in crate::codebase::postgres::query_annotation) struct SharedInit {
    pub(in crate::codebase::postgres::query_annotation) modules:
        FxHashMap<ModuleId, Arc<CachedModule>>,
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation) fn try_install_shared_module(
        &mut self,
        path: &Path,
        module_id: ModuleId,
    ) -> Option<Environment> {
        let cached = self
            .modules
            .shared
            .as_ref()?
            .modules
            .get(&module_id)?
            .clone();
        Some(self.install_cached_module(path, module_id, &cached, None))
    }

    pub(in crate::codebase::postgres::query_annotation) fn install_cached_module(
        &mut self,
        path: &Path,
        module_id: ModuleId,
        cached: &CachedModule,
        mut dep_log: Option<&mut Vec<(ModuleId, usize)>>,
    ) -> Environment {
        if let Some(env) = self.modules.get(&module_id) {
            return *env;
        }
        for &dep_id in &cached.deps {
            if !self.modules.contains_key(&dep_id) {
                let dep_path = self.module_ids().path(dep_id).to_path_buf();
                let dep_cached = self
                    .modules
                    .shared
                    .as_ref()
                    .and_then(|shared| shared.modules.get(&dep_id))
                    .cloned()
                    .expect("cached module dependency must exist in SharedInit");
                self.install_cached_module(&dep_path, dep_id, &dep_cached, dep_log.as_deref_mut());
            }
        }
        let mut remap = Vec::with_capacity(cached.dep_refs.len() + cached.own_scopes.len());
        for &(dep_mod, slot) in &cached.dep_refs {
            let dep_base = self.modules[&dep_mod];
            remap.push(dep_base + slot);
        }
        let base = self.scopes.len();
        for offset in 0..cached.own_scopes.len() {
            remap.push(base + offset);
        }
        for (slot, scope) in cached.own_scopes.iter().enumerate() {
            self.scopes.push(remap_scope(scope, &remap));
            if let Some(log) = dep_log.as_deref_mut() {
                log.push((module_id, slot));
            }
        }
        self.modules.insert(module_id, base);
        self.record_alternative_module_initial(path, base);
        if dep_log.is_none() {
            if let Some(observer) = crate::diagnostics::current() {
                observer.increment("postgres.annotation.shared_init.cache_hits", 1);
            }
        }
        base
    }

    pub(in crate::codebase::postgres::query_annotation) fn is_pure_shared_module(
        &self,
        prefix_scopes: &[Scope],
        dep_module_count: usize,
    ) -> bool {
        let prefix = prefix_scopes.len();
        self.event_log.is_empty()
            && self.modules.len() == dep_module_count + 1
            && self.scopes.len() > prefix
            && self.scopes[prefix..]
                .iter()
                .all(|scope| !scope.contains_reference())
            && self.scopes[..prefix]
                .iter()
                .zip(prefix_scopes)
                .all(|(current, base)| current.same_bindings(base))
            && self.invalidated_builders.is_empty()
            && self.builder_updates.is_empty()
            && self.deleted_argument_slots.is_empty()
            && self.definite_deleted_argument_slots.is_empty()
            && self.disconnected_argument_slots.is_empty()
            && self.recreated_argument_slots.is_empty()
            && self.mapped_arguments.is_empty()
            && self.fresh_mapped_parameters.is_empty()
            && self.argument_objects.is_empty()
            && self.argument_extra_slots.is_empty()
            && self.captured_bindings.is_empty()
            && self.captured_binding_readers.is_empty()
            && self.mapped_argument_owners.is_empty()
            && self.mapped_parameter_indices.is_empty()
    }
}

fn remap_scope(scope: &Scope, remap: &[Environment]) -> Scope {
    if !scope.values().any(|value| needs_remap(value, remap)) {
        return scope.clone();
    }
    let values = scope
        .iter()
        .map(|(name, value)| (name.clone(), remap_value(value, remap)))
        .collect::<FxHashMap<_, _>>();
    Scope::from(values)
}

fn needs_remap(value: &Value, remap: &[Environment]) -> bool {
    match value {
        Value::Function(_, _, env) => remap
            .get(*env)
            .copied()
            .is_some_and(|mapped| mapped != *env),
        Value::Promise(inner) | Value::Evaluated(inner, _) => needs_remap(inner, remap),
        Value::Aggregate(items)
        | Value::References(items)
        | Value::Object(items)
        | Value::Record(items, _)
        | Value::Joined(items)
        | Value::Possible(items) => items.iter().any(|item| needs_remap(item, remap)),
        Value::Prefix(..)
        | Value::Arguments(_)
        | Value::Array(_)
        | Value::Number(_)
        | Value::Boolean(_)
        | Value::Failure(_)
        | Value::Unknown
        | Value::Primitive
        | Value::Unsupported
        | Value::SlotDeletion => false,
    }
}

fn remap_values(items: &Values, remap: &[Environment]) -> Values {
    Values::from(
        items
            .iter()
            .map(|item| remap_value(item, remap))
            .collect::<Vec<_>>(),
    )
}

fn remap_value(value: &Value, remap: &[Environment]) -> Value {
    match value {
        Value::Function(function, path, env) => Value::Function(
            Arc::clone(function),
            Arc::clone(path),
            remap.get(*env).copied().unwrap_or(*env),
        ),
        Value::Promise(inner) => Value::Promise(Box::new(remap_value(inner, remap))),
        Value::Evaluated(inner, exact) => {
            Value::Evaluated(Box::new(remap_value(inner, remap)), *exact)
        }
        Value::Aggregate(items) => Value::Aggregate(remap_values(items, remap)),
        Value::References(items) => Value::References(remap_values(items, remap)),
        Value::Object(items) => Value::Object(remap_values(items, remap)),
        Value::Record(items, keys) => Value::Record(remap_values(items, remap), keys.clone()),
        Value::Joined(items) => Value::Joined(remap_values(items, remap)),
        Value::Possible(items) => Value::Possible(remap_values(items, remap)),
        _ => value.clone(),
    }
}
