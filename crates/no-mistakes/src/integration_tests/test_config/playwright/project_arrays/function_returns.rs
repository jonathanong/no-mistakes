use super::{expression_options, shared, Ctx, FunctionBody, Options, Statement};
use anyhow::Result;
use std::collections::BTreeSet;

pub(super) fn body_return_options(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let body_bindings = shared::function_body_bindings(body);
    if !body_bindings.is_empty() {
        let mut bindings = ctx.bindings.clone();
        bindings.extend(body_bindings);
        let mut local_seen = BTreeSet::new();
        let mut object_seen = BTreeSet::new();
        let mut scoped = Ctx {
            source: ctx.source,
            bindings,
            functions: ctx.functions.clone(),
            imports: ctx.imports.clone(),
            resolver: ctx.resolver,
            path: ctx.path,
            seen: ctx.seen,
            local_seen: &mut local_seen,
            object_seen: &mut object_seen,
        };
        return body_return_options_without_locals(body, &mut scoped);
    }
    body_return_options_without_locals(body, ctx)
}

fn body_return_options_without_locals(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    for statement in &body.statements {
        let Statement::ReturnStatement(return_statement) = statement else {
            continue;
        };
        let Some(argument) = &return_statement.argument else {
            continue;
        };
        return expression_options(argument, ctx);
    }
    Ok(Vec::new())
}
