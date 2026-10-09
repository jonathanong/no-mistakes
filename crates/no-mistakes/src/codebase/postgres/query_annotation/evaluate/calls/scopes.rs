use super::super::{Function, Value};
use crate::codebase::postgres::query_annotation::Step;
use crate::fx::{FxHashMap, FxHashSet};

pub(in crate::codebase::postgres::query_annotation::evaluate) struct Shadows<'a> {
    names: FxHashSet<&'a str>,
    pub construction_work: usize,
}
impl Shadows<'_> {
    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }
}
pub(in crate::codebase::postgres::query_annotation::evaluate) fn shadow_names(
    function: &Function,
) -> Shadows<'_> {
    let mut shadowed = Shadows {
        names: FxHashSet::default(),
        construction_work: 0,
    };
    for name in &function.params {
        shadowed.names.insert(name);
        shadowed.construction_work += 1;
    }
    if let Some(name) = &function.self_name {
        shadowed.names.insert(name);
        shadowed.construction_work += 1;
    }
    if !function.arrow {
        shadowed.names.insert("arguments");
        shadowed.construction_work += 1;
    }
    for step in &function.body {
        shadowed.construction_work += 1;
        match step {
            Step::Bind(local, _) | Step::Hoisted(local, _) | Step::Var(local) => {
                shadowed.names.insert(local);
            }
            Step::Reserve(names) => {
                for name in names {
                    shadowed.names.insert(name);
                    shadowed.construction_work += 1;
                }
            }
            _ => {}
        }
    }
    shadowed
}

/// A callee's declarations shadow captured bindings even when a bare var
/// has no initializer. The caller restores actual parameters after this step,
/// so parameter redeclarations still preserve their runtime argument values.
pub(in crate::codebase::postgres::query_annotation::evaluate) fn locals(
    captured: &FxHashMap<String, Value>,
    function: &Function,
) -> FxHashMap<String, Value> {
    let mut locals = captured.clone();
    for step in &function.body {
        match step {
            Step::Bind(name, _) | Step::Hoisted(name, _) | Step::Var(name) => {
                locals.remove(name);
            }
            Step::Reserve(names) => {
                for name in names {
                    locals.remove(name);
                }
            }
            _ => {}
        }
    }
    // A local bare var is proven undefined until initialized. Reserve it as
    // authoritative unknown, then the invocation restores any same-name argument.
    for step in &function.body {
        if let Step::Var(name) = step {
            locals.insert(name.clone(), Value::Unknown);
        }
    }
    locals
}

pub(in crate::codebase::postgres::query_annotation::evaluate) fn arguments(
    locals: &mut FxHashMap<String, Value>,
    function: &Function,
    value: Value,
) {
    if !function.arrow {
        // Arrows retain lexical arguments; regular calls own a fresh object.
        locals.insert("arguments".into(), value);
    }
}

#[cfg(test)]
#[path = "scopes/tests.rs"]
mod tests;
