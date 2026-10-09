use super::{calls::scopes, Evaluator, Value};
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn opaque_callbacks(&mut self, arguments: &[Value], depth: u8) {
        self.callback_values(arguments, depth, &mut FxHashSet::default());
    }

    fn callback_values(&mut self, values: &[Value], depth: u8, visited: &mut FxHashSet<u64>) {
        for value in values {
            match value {
                Value::Aggregate(values) => self.callback_values(values, depth, visited),
                Value::Promise(value) | Value::Evaluated(value, _) => {
                    self.callback_values(std::slice::from_ref(value.as_ref()), depth, visited);
                }
                Value::Arguments(id) if visited.insert(*id) => {
                    let live = self.argument_objects[id]
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| {
                            !self
                                .definite_deleted_argument_slots
                                .contains(&(*id, *index))
                        })
                        .map(|(_, value)| value.clone())
                        .collect::<Vec<_>>();
                    self.callback_values(&live, depth, visited);
                }
                Value::Function(function, path, captured) => {
                    self.invalidate_captured(*captured, function);
                    let mut locals = scopes::locals(&self.scopes[*captured], function);
                    if let Some(name) = &function.self_name {
                        locals.insert(name.clone(), Value::Unknown);
                    }
                    scopes::arguments(&mut locals, function, Value::Unknown);
                    for name in &function.params {
                        locals.insert(name.clone(), Value::Unknown);
                    }
                    let mut scope = self.environment(locals);
                    self.register_mappings(scope, *captured, function, None);
                    self.steps(&function.body, path, &mut scope, depth, false);
                }
                _ => {}
            }
        }
    }
}
