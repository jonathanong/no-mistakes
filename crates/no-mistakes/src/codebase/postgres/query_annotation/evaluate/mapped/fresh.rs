use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

fn callbacks(value: &Value) -> Option<Value> {
    match value {
        Value::Function(..) | Value::Arguments(_) => Some(value.clone()),
        Value::Promise(inner) => callbacks(inner).map(|value| Value::Promise(Box::new(value))),
        Value::Evaluated(inner, _) => callbacks(inner),
        Value::Aggregate(values) => {
            let values = values.iter().filter_map(callbacks).collect::<Vec<_>>();
            (!values.is_empty()).then_some(Value::Aggregate(values))
        }
        _ => None,
    }
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn invalidate_mapped_freshness(
        &mut self,
        ids: &FxHashSet<u64>,
    ) {
        for (env, names) in &mut self.fresh_mapped_parameters {
            if let Some(mappings) = self.mapped_arguments.get(env) {
                names.retain(|name| {
                    !mappings
                        .iter()
                        .any(|(id, params)| ids.contains(id) && params.contains(name))
                });
            }
        }
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn invalidate_opaque_mapped_names(
        &mut self,
        children: &[Expr],
        env: Environment,
    ) {
        let mut ids = FxHashSet::default();
        for child in children {
            if let Expr::Name(name) = child {
                for (id, params) in self.mapped_arguments.get(&env).into_iter().flatten() {
                    if params.contains(name) {
                        ids.insert(*id);
                    }
                }
            }
        }
        self.invalidate_mapped_freshness(&ids);
        self.invalidated_builders.extend(ids);
    }

    pub(super) fn inherit_freshness(
        &mut self,
        scope: Environment,
        captured: Environment,
        mappings: &[(u64, Vec<String>)],
    ) {
        let mut fresh = self
            .fresh_mapped_parameters
            .get(&captured)
            .cloned()
            .unwrap_or_default();
        fresh.retain(|name| mappings.iter().any(|(_, params)| params.contains(name)));
        if !fresh.is_empty() {
            self.fresh_mapped_parameters.insert(scope, fresh);
        }
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn mapped_parameter_value(
        &self,
        env: Environment,
        name: &str,
    ) -> Option<Value> {
        if !self.mapped_parameter_unknown(env, name) {
            return None;
        }
        let mut possibilities = vec![Value::Unknown];
        for (id, params) in &self.mapped_arguments[&env] {
            if let Some(index) = params.iter().rposition(|param| param == name) {
                if !self.definite_deleted_argument_slots.contains(&(*id, index)) {
                    let value = &self.argument_objects[id][index];
                    if let Some(value) = callbacks(value) {
                        if !possibilities.contains(&value) {
                            possibilities.push(value);
                        }
                    }
                }
            }
        }
        Some(if possibilities.len() == 1 {
            Value::Unknown
        } else {
            Value::Aggregate(possibilities)
        })
    }
}
