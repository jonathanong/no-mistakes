use super::super::{Function, Value};
use crate::codebase::postgres::query_annotation::Step;
use crate::fx::FxHashMap;

/// A callee's declarations shadow captured bindings even when a bare var
/// has no initializer. The caller restores actual parameters after this step,
/// so parameter redeclarations still preserve their runtime argument values.
pub(super) fn locals(
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

pub(super) fn arguments(locals: &mut FxHashMap<String, Value>, function: &Function, value: Value) {
    if !function.arrow {
        // Arrows retain lexical arguments; regular calls own a fresh object.
        locals.insert("arguments".into(), value);
    }
}
