use super::super::{
    heads::callback_heads::{producer::parameters, uses::Uses, Payload},
    AppendCallback, Expr, Function, Step,
};
use oxc_ast::ast;

pub(super) fn collect(function: &ast::Function<'_>, summary: &Function) -> Option<AppendCallback> {
    let (site, callback, payload) = call_site(summary)?;
    let payload_index = summary.params.iter().position(|name| name == &payload)?;
    let mut tags = Vec::new();
    let mut body = Vec::with_capacity(summary.body.len());
    for (index, step) in summary.body.iter().enumerate() {
        if index > site && !inert_after_call(step, &payload) {
            return None;
        }
        if let Step::Branch(test, left, right) = step {
            if !pure(test, &mut tags)
                || !arm(left, &payload, &mut tags)
                || !arm(right, &payload, &mut tags)
            {
                return None;
            }
            // Conditional suffixes preserve only the head, never complete SQL.
            body.push(Step::Append(payload.clone(), Expr::Unknown));
        } else {
            body.push(step.clone());
        }
    }
    if tags.iter().any(|tag| {
        let name = if tag == "String.raw" { "String" } else { tag };
        summary.params.iter().any(|param| param == name)
            || function.id.as_ref().is_some_and(|id| id.name == name)
            || summary.body.iter().any(|step| matches!(step, Step::Reserve(names) if names.iter().any(|local| local == name)))
    }) {
        return None;
    }
    let names = parameters(function)?;
    let proof = Uses::inspect(
        function.body.as_ref()?,
        names,
        callback,
        &Default::default(),
    )?;
    (proof == Payload::Parameter(payload_index)).then_some(AppendCallback {
        payload: payload_index,
        body,
        tags,
    })
}

// Branchless proof: the callback call is the destructured result, and the same
// parameter was appended in place. Re-lowering stays on the branch path.
pub(super) fn destructured_append(summary: &Function) -> bool {
    let Some((index, _, payload)) = call_site(summary) else {
        return false;
    };
    let Step::Bind(name, _) = &summary.body[index] else {
        return false;
    };
    name == super::super::statements::destructured::RESULT
        && summary
            .body
            .iter()
            .any(|step| matches!(step, Step::Append(bound, _) if bound == &payload))
}

// The callback call may be returned, bound, or a bare statement; only the SQL
// built in place before it matters, never its result.
fn call_site(summary: &Function) -> Option<(usize, usize, String)> {
    summary
        .body
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, step)| {
            let (Step::Return(expr) | Step::Bind(_, expr) | Step::Effect(expr)) = step else {
                return None;
            };
            let Expr::Call { callee, args, .. } = awaited(expr) else {
                return None;
            };
            let (Expr::Name(callback), [Expr::Name(payload)]) = (&**callee, args.as_slice()) else {
                return None;
            };
            let callback = summary.params.iter().position(|name| name == callback)?;
            Some((index, callback, payload.clone()))
        })
}

// After the callback call only the result may be read: never a call, never the
// builder itself.
fn inert_after_call(step: &Step, payload: &str) -> bool {
    match step {
        Step::Return(expr) | Step::Bind(_, expr) => result_read(expr, payload),
        _ => false,
    }
}

fn result_read(expr: &Expr, payload: &str) -> bool {
    match expr {
        Expr::Name(name) => name != payload,
        Expr::Member(inner, _) | Expr::Await(inner) => result_read(inner, payload),
        Expr::Unknown | Expr::Primitive | Expr::Text(_) | Expr::Number(_) | Expr::Boolean(_) => {
            true
        }
        _ => false,
    }
}

fn awaited(expr: &Expr) -> &Expr {
    match expr {
        Expr::Await(inner) => inner,
        _ => expr,
    }
}

fn arm(steps: &[Step], payload: &str, tags: &mut Vec<String>) -> bool {
    steps.iter().all(|step| match step {
        Step::Reserve(names) => names.is_empty(),
        Step::PotentialCalls(_) => true,
        Step::Append(name, tail) => name == payload && pure(tail, tags),
        Step::Branch(test, left, right) => {
            pure(test, tags) && arm(left, payload, tags) && arm(right, payload, tags)
        }
        _ => false,
    })
}

fn pure(expr: &Expr, tags: &mut Vec<String>) -> bool {
    match expr {
        Expr::Text(_) | Expr::Name(_) | Expr::Number(_) | Expr::Boolean(_) | Expr::Primitive => {
            true
        }
        Expr::Binary(_, left, right) => pure(left, tags) && pure(right, tags),
        Expr::Tagged(tag, parts, effects) => {
            tags.push(tag.clone());
            parts.iter().chain(effects).all(|expr| pure(expr, tags))
        }
        _ => false,
    }
}
