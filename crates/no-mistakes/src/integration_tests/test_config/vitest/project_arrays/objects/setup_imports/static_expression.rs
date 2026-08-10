use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ArrayExpressionElement, Expression, Program, Statement};
use std::collections::{BTreeMap, BTreeSet};

/// Imports only replace their use-site declaration when their exported value
/// is a literal setup string/array. Calls and other executable values remain
/// dynamic at the use site so their fallback ownership stays intact.
pub(super) fn is_static_setup_expression(
    expression: &Expression<'_>,
    bindings: &BTreeMap<String, &Expression<'_>>,
    seen: &mut BTreeSet<String>,
) -> bool {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => true,
        Expression::ArrayExpression(array) => array.elements.iter().all(|element| match element {
            ArrayExpressionElement::Elision(_) => true,
            ArrayExpressionElement::SpreadElement(spread) => {
                is_static_setup_expression(&spread.argument, bindings, seen)
            }
            _ => element
                .as_expression()
                .is_some_and(|expression| is_static_setup_expression(expression, bindings, seen)),
        }),
        Expression::Identifier(identifier) => {
            let name = identifier.name.to_string();
            if !seen.insert(name.clone()) {
                return false;
            }
            let static_value = bindings
                .get(&name)
                .is_some_and(|binding| is_static_setup_expression(binding, bindings, seen));
            seen.remove(&name);
            static_value
        }
        _ => false,
    }
}

pub(super) fn exported_setup_expression<'a>(
    program: &'a Program<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    exported: &str,
) -> Option<&'a Expression<'a>> {
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() {
            continue;
        }
        for specifier in &export.specifiers {
            if !specifier.export_kind.is_type() && specifier.exported.name() == exported {
                return bindings.get(specifier.local.name().as_str()).copied();
            }
        }
    }
    if exported == "default" {
        let expression = program.body.iter().find_map(|statement| {
            let Statement::ExportDefaultDeclaration(export) = statement else {
                return None;
            };
            let expression = export.declaration.as_expression()?;
            match expression {
                Expression::Identifier(identifier) => {
                    bindings.get(identifier.name.as_str()).copied()
                }
                _ => Some(expression),
            }
        });
        if expression.is_some() {
            return expression;
        }
    }
    super::commonjs::commonjs_setup_expression(program, exported)
}
