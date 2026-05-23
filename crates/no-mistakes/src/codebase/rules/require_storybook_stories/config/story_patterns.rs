use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectExpression, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::Path;

mod export_config;

pub(in crate::codebase::rules::require_storybook_stories) fn extract_storybook_story_patterns(
    source: &str,
) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return Vec::new();
    }
    export_config::stories_expression(&parsed.program)
        .map(|expression| story_patterns_from_expression(expression, source))
        .unwrap_or_default()
}

pub(in crate::codebase::rules::require_storybook_stories) fn project_relative_pattern(
    project_root: &Path,
    base: &Path,
    pattern: &str,
) -> String {
    let project_root = normalize_path(project_root);
    let pattern_path = Path::new(pattern);
    if pattern_path.is_absolute() {
        return relative_slash_path(&project_root, &normalize_path(pattern_path));
    }
    let joined = base.join(pattern_path);
    relative_slash_path(&project_root, &normalize_path(&joined))
}

fn story_patterns_from_expression(expression: &Expression<'_>, source: &str) -> Vec<String> {
    let expression = parenthesized_expression(expression);
    if let Some(pattern) = optional_string(expression, source) {
        return vec![pattern];
    }
    match expression {
        Expression::ArrayExpression(array) => array
            .elements
            .iter()
            .filter_map(|element| story_pattern_from_element(element, source))
            .collect(),
        Expression::ObjectExpression(object) => story_pattern_from_object(object, source)
            .into_iter()
            .collect(),
        Expression::ArrowFunctionExpression(arrow) => {
            story_patterns_from_statements(&arrow.body.statements, source)
        }
        Expression::FunctionExpression(function) => function
            .body
            .as_ref()
            .map(|body| story_patterns_from_statements(&body.statements, source))
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn story_patterns_from_statements(statements: &[Statement<'_>], source: &str) -> Vec<String> {
    for statement in statements {
        match statement {
            Statement::ReturnStatement(return_statement) => {
                if let Some(argument) = &return_statement.argument {
                    return story_patterns_from_expression(argument, source);
                }
            }
            Statement::ExpressionStatement(expression) => {
                let patterns = story_patterns_from_expression(&expression.expression, source);
                if !patterns.is_empty() {
                    return patterns;
                }
            }
            _ => {}
        }
    }
    Vec::new()
}

fn story_pattern_from_element(
    element: &ArrayExpressionElement<'_>,
    source: &str,
) -> Option<String> {
    match element {
        ArrayExpressionElement::StringLiteral(literal) => Some(literal.value.to_string()),
        ArrayExpressionElement::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(crate::ast::template_literal_text(template, source))
        }
        ArrayExpressionElement::ObjectExpression(object) => {
            story_pattern_from_object(object, source)
        }
        _ => None,
    }
}

fn story_pattern_from_object(object: &ObjectExpression<'_>, source: &str) -> Option<String> {
    let directory = object_string_property(object, "directory", source)?;
    let files = object_string_property(object, "files", source)
        .unwrap_or_else(|| "**/*.@(mdx|stories.@(js|jsx|mjs|ts|tsx))".to_string());
    Some(format!("{}/{}", directory.trim_end_matches('/'), files))
}

fn object_string_property(
    object: &ObjectExpression<'_>,
    name: &str,
    source: &str,
) -> Option<String> {
    object.properties.iter().find_map(|property| {
        let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property.method {
            return None;
        }
        let key = crate::codebase::ts_source::static_property_key_name(&property.key)?;
        (key == name).then(|| optional_string(&property.value, source))?
    })
}

fn optional_string(expression: &Expression<'_>, source: &str) -> Option<String> {
    match parenthesized_expression(expression) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(crate::ast::template_literal_text(template, source))
        }
        _ => None,
    }
}

fn parenthesized_expression<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            parenthesized_expression(&parenthesized.expression)
        }
        _ => expression,
    }
}
