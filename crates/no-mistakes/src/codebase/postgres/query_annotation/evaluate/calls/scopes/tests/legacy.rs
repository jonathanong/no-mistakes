use super::*;

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
    for step in &function.body {
        if let Step::Var(name) = step {
            locals.insert(name.clone(), Value::Unknown);
        }
    }
    locals
}

pub(super) fn arguments(locals: &mut FxHashMap<String, Value>, function: &Function, value: Value) {
    if !function.arrow {
        locals.insert("arguments".into(), value);
    }
}

pub(super) fn bind_this(
    locals: &mut FxHashMap<String, Value>,
    function: &Function,
    receiver: Option<&Value>,
) {
    if !function.arrow {
        locals.insert("this".into(), receiver.cloned().unwrap_or(Value::Unknown));
    }
}
