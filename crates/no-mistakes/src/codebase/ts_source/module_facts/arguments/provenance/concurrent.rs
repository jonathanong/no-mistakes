use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;

pub(super) fn overlap(semantic: &Semantic<'_>, call: &CallExpression<'_>) -> bool {
    let mut concurrent = false;
    let Some(id) = root(&call.callee, &mut concurrent) else {
        return false;
    };
    if !runner(semantic, id) {
        return false;
    }
    if concurrent {
        return true;
    }
    call.arguments.len() >= 2
        && call.arguments.iter().skip(1).any(|argument| {
            let Some(expression) = argument.as_expression() else {
                return true;
            };
            if callback_span(semantic, expression).is_some() {
                return false;
            }
            match expression.get_inner_expression() {
                Expression::NumericLiteral(_) => false,
                Expression::ObjectExpression(object) => object.properties.iter().any(option),
                _ => true,
            }
        })
}

fn option(property: &ObjectPropertyKind<'_>) -> bool {
    let ObjectPropertyKind::ObjectProperty(property) = property else {
        return true;
    };
    if property.computed {
        return true;
    }
    crate::ast::property_key_name(&property.key)
        .as_deref()
        .is_none_or(|name| {
            name == "concurrent"
                && !matches!(property.value.get_inner_expression(),
            Expression::BooleanLiteral(value) if !value.value)
        })
}

fn root<'a, 'b>(
    expression: &'b Expression<'a>,
    concurrent: &mut bool,
) -> Option<&'b IdentifierReference<'a>> {
    match expression.get_inner_expression() {
        Expression::Identifier(id) => Some(id),
        Expression::StaticMemberExpression(member) => {
            *concurrent |= member.property.name == "concurrent";
            root(&member.object, concurrent)
        }
        Expression::ComputedMemberExpression(member) => {
            *concurrent |= !matches!(member.expression.get_inner_expression(),
                Expression::StringLiteral(name) if crate::ast::js_literal(name) != "concurrent");
            root(&member.object, concurrent)
        }
        Expression::CallExpression(call) => root(&call.callee, concurrent),
        Expression::TaggedTemplateExpression(template) => root(&template.tag, concurrent),
        _ => None,
    }
}

fn runner(semantic: &Semantic<'_>, id: &IdentifierReference<'_>) -> bool {
    id.reference_id.get().is_some_and(|reference| {
        let Some(symbol) = semantic.scoping().get_reference(reference).symbol_id() else {
            return test_name(id.name.as_str());
        };
        let declaration = semantic.scoping().symbol_declaration(symbol);
        match (
            semantic.nodes().kind(declaration),
            semantic.nodes().parent_kind(declaration),
        ) {
            (AstKind::ImportSpecifier(binding), AstKind::ImportDeclaration(import))
                if !binding.import_kind.is_type()
                    && !import.import_kind.is_type()
                    && crate::ast::js_literal(&import.source) == "vitest" =>
            {
                test_name(&crate::ast::module_export_name(&binding.imported))
            }
            _ => false,
        }
    })
}

fn test_name(name: &str) -> bool {
    matches!(name, "it" | "test" | "describe" | "suite")
}

fn callback_span(semantic: &Semantic<'_>, expression: &Expression<'_>) -> Option<(u32, u32)> {
    match expression.get_inner_expression() {
        Expression::ArrowFunctionExpression(function) => {
            Some((function.span.start, function.span.end))
        }
        Expression::FunctionExpression(function) => Some((function.span.start, function.span.end)),
        Expression::Identifier(id) => {
            let symbol = semantic
                .scoping()
                .get_reference(id.reference_id.get()?)
                .symbol_id()?;
            for (index, reference) in semantic
                .scoping()
                .get_resolved_references(symbol)
                .enumerate()
            {
                if index >= 256 || reference.flags().is_write() {
                    return None;
                }
            }
            match semantic
                .nodes()
                .kind(semantic.scoping().symbol_declaration(symbol))
            {
                AstKind::Function(function) => Some((function.span.start, function.span.end)),
                AstKind::VariableDeclarator(declaration) => {
                    match declaration.init.as_ref()?.get_inner_expression() {
                        Expression::ArrowFunctionExpression(function) => {
                            Some((function.span.start, function.span.end))
                        }
                        Expression::FunctionExpression(function) => {
                            Some((function.span.start, function.span.end))
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub(super) fn callbacks(semantic: &Semantic<'_>, call: &CallExpression<'_>) -> Vec<(u32, u32)> {
    let mut spans: Vec<_> = call
        .arguments
        .iter()
        .skip(1)
        .filter_map(Argument::as_expression)
        .filter_map(|expression| callback_span(semantic, expression))
        .collect();
    if spans.is_empty() && call.arguments.len() >= 2 {
        spans.push((0, u32::MAX));
    }
    spans
}
