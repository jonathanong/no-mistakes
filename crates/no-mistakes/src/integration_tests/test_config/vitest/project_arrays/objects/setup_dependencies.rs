use super::super::{shared, Ctx};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_span::GetSpan;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn setup_dependencies(
    value: &Expression<'_>,
    field: VitestSetupField,
    ctx: &mut Ctx<'_, '_>,
) -> Vec<VitestSetupDependency> {
    let value = shared::expression_value(value, &ctx.bindings);
    if let Expression::Identifier(identifier) = unwrap_ts_wrappers(value) {
        if let Some(import) = ctx.imports.get(identifier.name.as_str()).cloned() {
            if let Some(dependencies) =
                super::setup_imports::imported_setup_dependencies(&import, field, ctx)
            {
                return dependencies;
            }
        }
    }
    if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(value) {
        if let Some(dependencies) =
            super::static_members::static_member_setup_dependencies(member, field, ctx)
        {
            return dependencies;
        }
    }
    match unwrap_ts_wrappers(value) {
        Expression::ArrayExpression(array) => array
            .elements
            .iter()
            .flat_map(|element| match element {
                ArrayExpressionElement::Elision(_) => Vec::new(),
                ArrayExpressionElement::SpreadElement(spread) => {
                    setup_dependencies(&spread.argument, field, ctx)
                }
                _ => element
                    .as_expression()
                    .map(|expression| setup_dependencies(expression, field, ctx))
                    .unwrap_or_default(),
            })
            .collect(),
        expression => vec![setup_dependency(expression, field, ctx)],
    }
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
        resolution_base: ctx
            .path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
        declaration_path: ctx.path.to_path_buf(),
        declaration_line,
        trigger_paths,
        resolver_candidate_paths: BTreeSet::new(),
        transitive_trigger_paths: BTreeSet::new(),
    }
}
