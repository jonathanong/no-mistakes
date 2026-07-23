use super::super::{shared, Ctx};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_span::GetSpan;
use std::collections::BTreeSet;
use std::path::Path;

mod bounded_literals;

pub(super) fn setup_dependencies(
    value: &Expression<'_>,
    field: VitestSetupField,
    ctx: &mut Ctx<'_, '_>,
) -> Vec<VitestSetupDependency> {
    let mut remaining = MAX_STATIC_SETUP_DEPENDENCIES;
    setup_dependencies_bounded(value, field, ctx, 0, &mut remaining)
        .unwrap_or_else(|| vec![conservative_setup_dependency(value, field, ctx)])
}

// Keep branch enumeration deterministic and bounded for generated configs.
const MAX_STATIC_SETUP_BRANCH_DEPTH: usize = 32;
const MAX_STATIC_SETUP_DEPENDENCIES: usize = 64;

fn setup_dependencies_bounded(
    value: &Expression<'_>,
    field: VitestSetupField,
    ctx: &mut Ctx<'_, '_>,
    depth: usize,
    remaining: &mut usize,
) -> Option<Vec<VitestSetupDependency>> {
    if depth >= MAX_STATIC_SETUP_BRANCH_DEPTH {
        return None;
    }
    if let Expression::Identifier(identifier) = unwrap_ts_wrappers(value) {
        if let Some(import) = ctx.imports.get(identifier.name.as_str()).cloned() {
            if let Some(dependencies) =
                super::setup_imports::imported_setup_dependencies(&import, field, ctx)
            {
                return take_dependencies(dependencies, remaining);
            }
        }
    }
    let value = shared::expression_value(value, &ctx.bindings);
    if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(value) {
        if let Some(dependencies) =
            super::static_members::static_member_setup_dependencies(member, field, ctx)
        {
            return take_dependencies(dependencies, remaining);
        }
    }
    match unwrap_ts_wrappers(value) {
        Expression::ConditionalExpression(conditional) => {
            let branch_triggers =
                super::dynamic_triggers::dynamic_trigger_paths(&conditional.test, ctx);
            let mut dependencies = setup_dependencies_bounded(
                &conditional.consequent,
                field,
                ctx,
                depth + 1,
                remaining,
            )?;
            dependencies.extend(setup_dependencies_bounded(
                &conditional.alternate,
                field,
                ctx,
                depth + 1,
                remaining,
            )?);
            for dependency in &mut dependencies {
                dependency
                    .trigger_paths
                    .extend(branch_triggers.iter().cloned());
            }
            Some(dependencies)
        }
        Expression::ArrayExpression(array) => {
            let mut dependencies = Vec::new();
            for element in &array.elements {
                let next = match element {
                    ArrayExpressionElement::Elision(_) => Vec::new(),
                    ArrayExpressionElement::SpreadElement(spread) => setup_dependencies_bounded(
                        &spread.argument,
                        field,
                        ctx,
                        depth + 1,
                        remaining,
                    )?,
                    _ => setup_dependencies_bounded(
                        element
                            .as_expression()
                            .expect("non-spread, non-elision array elements are expressions"),
                        field,
                        ctx,
                        depth + 1,
                        remaining,
                    )?,
                };
                dependencies.extend(next);
            }
            Some(dependencies)
        }
        expression => take_dependencies(vec![setup_dependency(expression, field, ctx)], remaining),
    }
}

fn take_dependencies(
    dependencies: Vec<VitestSetupDependency>,
    remaining: &mut usize,
) -> Option<Vec<VitestSetupDependency>> {
    if dependencies.len() > *remaining {
        return None;
    }
    *remaining -= dependencies.len();
    Some(dependencies)
}

fn setup_dependency(
    expression: &Expression<'_>,
    field: VitestSetupField,
    ctx: &Ctx<'_, '_>,
) -> VitestSetupDependency {
    let declaration_line =
        crate::codebase::ts_source::line_number(ctx.source, expression.span().start) as u32;
    let resolved_expression = shared::expression_value(expression, &ctx.bindings);
    let specifier = shared::optional_string(resolved_expression, ctx.source);
    let trigger_paths = if specifier.is_none() {
        super::dynamic_triggers::dynamic_trigger_paths(expression, ctx)
    } else {
        BTreeSet::from([ctx.path.to_path_buf()])
    };
    VitestSetupDependency {
        field,
        specifier,
        resolved_path: None,
        resolution_base: ctx.path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        declaration_path: ctx.path.to_path_buf(),
        declaration_line,
        trigger_paths,
        resolver_candidate_paths: BTreeSet::new(),
        transitive_trigger_paths: BTreeSet::new(),
    }
}

fn conservative_setup_dependency(
    expression: &Expression<'_>,
    field: VitestSetupField,
    ctx: &Ctx<'_, '_>,
) -> VitestSetupDependency {
    let mut dependency = setup_dependency(expression, field, ctx);
    // The dependency budget deliberately stops enumerating literal leaves.
    // Retain the declaration's resolution scope so edits to a literal setup
    // outside the owning project still trigger the conservative fallback.
    dependency
        .trigger_paths
        .insert(dependency.resolution_base.clone());
    dependency
        .trigger_paths
        .extend(bounded_literals::trigger_paths(expression, ctx));
    dependency
}
