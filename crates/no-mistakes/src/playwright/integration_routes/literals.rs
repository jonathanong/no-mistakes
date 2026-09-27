use crate::playwright::ast;
use oxc_ast::ast::Argument;

pub(super) fn literal(argument: &Argument<'_>) -> Option<String> {
    match argument {
        Argument::StringLiteral(value) => Some(value.value.to_string()),
        Argument::TemplateLiteral(value) if value.expressions.is_empty() => value
            .quasis
            .first()?
            .value
            .cooked
            .as_ref()
            .map(ToString::to_string),
        _ => None,
    }
}

pub(super) fn route_literal(argument: &Argument<'_>, source: &str) -> Option<String> {
    let value = match argument {
        Argument::TemplateLiteral(template) => Some(ast::template_literal_text(template, source)),
        Argument::BinaryExpression(binary) => ast::binary_concat_path_text(binary, source),
        _ => literal(argument),
    }?;
    crate::playwright::url::normalize_url(&value, &[])
}
