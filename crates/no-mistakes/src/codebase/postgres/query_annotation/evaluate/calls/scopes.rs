use super::super::{Function, Scope, Value};
use crate::codebase::postgres::query_annotation::Step;
use crate::fx::FxHashSet;

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
    shadow_names_in(function, &function.body)
}

pub(in crate::codebase::postgres::query_annotation::evaluate) fn shadow_names_in<'a>(
    function: &'a Function,
    body: &'a [Step],
) -> Shadows<'a> {
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
        shadowed.names.insert("this");
        shadowed.construction_work += 2;
    }
    for step in body {
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

pub(in crate::codebase::postgres::query_annotation::evaluate) fn has_legacy_branch(
    function: &Function,
    precise_records: bool,
) -> bool {
    !precise_records
        && function
            .body
            .iter()
            .any(|step| matches!(step, Step::Branch(_, _, _)) && !early_exit_guard(step))
}

/// An `if (test) return value` only prunes paths, so the statements after it
/// still describe every executed call. The test may run calls; the caller
/// invalidates live builders for those.
pub(in crate::codebase::postgres::query_annotation::evaluate) fn early_exit_guard(
    step: &Step,
) -> bool {
    use crate::codebase::postgres::query_annotation::Expr;
    let Step::Branch(_, left, right) = step else {
        return false;
    };
    let mut arm = left
        .iter()
        .filter(|step| !matches!(step, Step::PotentialCalls(_) | Step::Reserve(_)));
    let (Some(Step::Return(value)), None) = (arm.next(), arm.next()) else {
        return false;
    };
    // A guard that returns SQL text still feeds callers, so only values that
    // cannot be SQL (`undefined`, booleans, numbers) may be skipped.
    right.is_empty() && matches!(value, Expr::Primitive | Expr::Boolean(_) | Expr::Number(_))
        || matches!(value, Expr::Name(name) if name == "undefined") && right.is_empty()
}

pub(in crate::codebase::postgres::query_annotation::evaluate) fn inert(
    expr: &crate::codebase::postgres::query_annotation::Expr,
) -> bool {
    use crate::codebase::postgres::query_annotation::Expr;
    match expr {
        Expr::Boolean(_) | Expr::Number(_) | Expr::Primitive | Expr::Text(_) | Expr::Name(_) => {
            true
        }
        Expr::Discard(inner) => inert(inner),
        Expr::Binary(_, left, right) => inert(left) && inert(right),
        _ => false,
    }
}

/// A callee's declarations shadow captured bindings even when a bare var
/// has no initializer. The caller restores actual parameters after this step,
/// so parameter redeclarations still preserve their runtime argument values.
pub(in crate::codebase::postgres::query_annotation::evaluate) fn locals(
    captured: &Scope,
    function: &Function,
) -> Scope {
    captured.references();
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
    locals: &mut Scope,
    function: &Function,
    value: Value,
) {
    if !function.arrow {
        // Arrows retain lexical arguments; regular calls own a fresh object.
        locals.insert("arguments".into(), value);
    }
}

/// Non-arrow calls take `this` from the member or index receiver. A bare call
/// has no receiver. Arrows keep the lexical `this` already copied into locals.
pub(in crate::codebase::postgres::query_annotation::evaluate) fn bind_this(
    locals: &mut Scope,
    function: &Function,
    receiver: Option<&Value>,
) {
    if function.arrow {
        return;
    }
    locals.insert("this".into(), receiver.cloned().unwrap_or(Value::Unknown));
}

#[cfg(test)]
#[path = "scopes/tests.rs"]
mod tests;
