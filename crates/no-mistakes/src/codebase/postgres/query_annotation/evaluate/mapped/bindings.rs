use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::{Function, Step};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    /// Copied evaluation frames share inherited binding ownership. Declarations
    /// and ordinary arguments still create invocation-local bindings.
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn register_captured_bindings(
        &mut self,
        scope: Environment,
        captured: Environment,
        function: &Function,
    ) {
        let bindings = self.scopes[captured]
            .keys()
            .filter(|name| {
                !function.params.contains(name)
                    && function.self_name.as_ref() != Some(*name)
                    && (function.arrow || name.as_str() != "arguments")
                    && !function.body.iter().any(|step| match step {
                        Step::Bind(local, _) | Step::Hoisted(local, _) | Step::Var(local) => {
                            local == *name
                        }
                        Step::Reserve(names) => names.contains(name),
                        _ => false,
                    })
            })
            .map(|name| {
                let origin = self
                    .captured_bindings
                    .get(&captured)
                    .and_then(|bindings| bindings.get(name))
                    .copied()
                    .unwrap_or(captured);
                (name.clone(), origin)
            })
            .collect::<FxHashMap<_, _>>();
        if !bindings.is_empty() {
            self.captured_bindings.insert(scope, bindings);
        }
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn write_captured_binding(
        &mut self,
        env: Environment,
        name: &str,
        value: &Value,
    ) {
        let origin = self
            .captured_bindings
            .get(&env)
            .and_then(|bindings| bindings.get(name))
            .copied()
            .unwrap_or(env);
        let mut frames = vec![origin];
        frames.extend(
            self.captured_bindings
                .iter()
                .filter_map(|(frame, bindings)| {
                    (bindings.get(name) == Some(&origin)).then_some(*frame)
                }),
        );
        for frame in frames {
            self.update_mapped_parameter(frame, name, value);
            self.scopes[frame].insert(name.to_string(), value.clone());
        }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn refresh_captured_bindings(
        &mut self,
    ) {
        for (frame, bindings) in &self.captured_bindings {
            for (name, origin) in bindings {
                let value = self.scopes[*origin]
                    .get(name)
                    .cloned()
                    .unwrap_or(Value::Unknown);
                self.scopes[*frame].insert(name.clone(), value);
            }
        }
    }
}
