use super::super::{
    heads::callback_heads::{producer::parameters, uses::Uses, Payload},
    AppendCallback, Expr, Function, Step,
};
use oxc_ast::ast;

pub(super) fn collect(function: &ast::Function<'_>, summary: &Function) -> Option<AppendCallback> {
    if !summary
        .body
        .iter()
        .any(|step| matches!(step, Step::Branch(_, _, _)))
    {
        return None;
    }
    let Step::Return(mut returned) = summary.body.last()?.clone() else {
        return None;
    };
    if let Expr::Await(inner) = returned {
        returned = *inner;
    }
    let Expr::Call { callee, args, .. } = returned else {
        return None;
    };
    let Expr::Name(callback) = *callee else {
        return None;
    };
    let [Expr::Name(payload)] = args.as_slice() else {
        return None;
    };
    let callback = summary.params.iter().position(|name| name == &callback)?;
    let payload_index = summary.params.iter().position(|name| name == payload)?;
    let mut tags = Vec::new();
    let mut body = Vec::with_capacity(summary.body.len());
    for step in &summary.body {
        if let Step::Branch(test, left, right) = step {
            if !pure(test, &mut tags)
                || !arm(left, payload, &mut tags)
                || !arm(right, payload, &mut tags)
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
