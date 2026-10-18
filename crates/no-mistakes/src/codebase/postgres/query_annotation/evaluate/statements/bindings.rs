use super::super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Step;
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn reserve_bindings(&mut self, steps: &[Step], env: &Environment) {
        let vars = steps
            .iter()
            .filter_map(|step| match step {
                Step::Var(name) => Some(name.as_str()),
                _ => None,
            })
            .collect::<crate::fx::FxHashSet<_>>();
        // Reserve lexical names before evaluation so TDZ/shadowed bindings do
        // not accidentally resolve to a module helper or trusted tag.
        for step in steps {
            match step {
                Step::Bind(name, _) if vars.contains(name.as_str()) => {
                    // Initialized var redeclarations assign at this statement,
                    // rather than replacing a parameter during hoisting.
                    self.reserve_binding(*env, name.clone(), Value::Unknown);
                }
                Step::Bind(name, _) | Step::Hoisted(name, _) => {
                    self.write_binding(*env, name.clone(), Value::Unknown);
                }
                Step::Var(name) => {
                    self.reserve_binding(*env, name.clone(), Value::Unsupported);
                }
                Step::Reserve(names) => {
                    for name in names {
                        self.reserve_binding(*env, name.clone(), Value::Unknown);
                    }
                }
                _ => {}
            }
        }
    }
}
