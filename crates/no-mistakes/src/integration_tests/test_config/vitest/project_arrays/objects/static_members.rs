use super::super::{shared, Ctx, ImportBinding};
use super::setup_dependencies;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
use oxc_ast::ast::{Expression, StaticMemberExpression};

pub(super) fn static_member_setup_dependencies(
    member: &StaticMemberExpression<'_>,
    field: VitestSetupField,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Vec<VitestSetupDependency>> {
    let (root, chain) = member_chain(member)?;
    let seen_key = format!("setup:{root}");
    if !ctx.object_seen.insert(seen_key.clone()) {
        return None;
    }
    let result = resolve_member(&root, &chain, field, ctx);
    ctx.object_seen.remove(&seen_key);
    result
}

/// `base.test.setupFiles` walks a same-file const. One member can also be an import.
fn resolve_member(
    root: &str,
    chain: &[String],
    field: VitestSetupField,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Vec<VitestSetupDependency>> {
    if chain.len() == 1 {
        if let Some(import) = ctx.imports.get(root).cloned() {
            return imported_member(&import, &chain[0], field, ctx);
        }
    }
    let expression = local_member(root, chain, ctx)?;
    Some(setup_dependencies(expression, field, ctx))
}

fn imported_member(
    import: &ImportBinding,
    member: &str,
    field: VitestSetupField,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Vec<VitestSetupDependency>> {
    if import.imported == "*" {
        super::setup_imports::imported_setup_dependencies(
            &ImportBinding {
                source: import.source.clone(),
                imported: member.to_string(),
            },
            field,
            ctx,
        )
    } else {
        super::setup_imports::imported_setup_member_dependencies(import, member, field, ctx)
    }
}

/// Copy the program-owned expression out of the binding map before mutably using `ctx`.
fn local_member<'a>(root: &str, chain: &[String], ctx: &Ctx<'a, '_>) -> Option<&'a Expression<'a>> {
    let mut current = *ctx.bindings.get(root)?;
    for name in chain {
        let object = super::expression_object(current, &ctx.bindings)?;
        current = shared::property_expression_deep(object, name, &ctx.bindings)?;
    }
    Some(current)
}

fn member_chain(member: &StaticMemberExpression<'_>) -> Option<(String, Vec<String>)> {
    let mut chain = vec![member.property.name.to_string()];
    let mut cursor = unwrap_ts_wrappers(&member.object);
    loop {
        match cursor {
            Expression::StaticMemberExpression(inner) => {
                chain.push(inner.property.name.to_string());
                cursor = unwrap_ts_wrappers(&inner.object);
            }
            Expression::Identifier(identifier) => {
                chain.reverse();
                return Some((identifier.name.to_string(), chain));
            }
            _ => return None,
        }
    }
}
