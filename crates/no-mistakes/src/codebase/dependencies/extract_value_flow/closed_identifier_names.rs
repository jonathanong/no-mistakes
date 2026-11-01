use super::super::super::simple_callee_name;
use super::{literal_string, plain_call};
use oxc_ast::ast::{Argument, Expression, RegExpFlags};
use oxc_syntax::operator::BinaryOperator;

pub(super) fn member_object(expr: &Expression<'_>) -> Option<String> {
    let object = match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::StaticMemberExpression(member) => &member.object,
        Expression::ComputedMemberExpression(member) => &member.object,
        _ => return None,
    };
    identifier_name(object)
}

pub(super) fn map_get_receiver(expr: &Expression<'_>) -> Option<String> {
    let call = plain_call(expr)?;
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return None;
    };
    if member.property.name != "get" {
        return None;
    }
    identifier_name(&member.object)
}

fn identifier_name(expr: &Expression<'_>) -> Option<String> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        _ => None,
    }
}

pub(super) fn stripped_temp_name(expr: &Expression<'_>) -> bool {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::TemplateLiteral(template) => {
            let Some(first) = template.quasis.first() else {
                return false;
            };
            !template.expressions.is_empty()
                && !crate::ast::template_quasi_text(first).is_empty()
                && template.expressions.iter().all(stripped_uuid)
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            literal_string(&binary.left).is_some_and(|text| !text.is_empty())
                && stripped_uuid(&binary.right)
        }
        _ => false,
    }
}

fn stripped_uuid(expr: &Expression<'_>) -> bool {
    strip_hyphens(expr).is_some_and(bare_random_uuid)
}

fn bare_random_uuid(expr: &Expression<'_>) -> bool {
    let Some(call) = plain_call(expr) else {
        return false;
    };
    if !call.arguments.is_empty() {
        return false;
    }
    matches!(
        simple_callee_name(&call.callee).as_deref(),
        Some("randomUUID" | "crypto.randomUUID")
    )
}

fn strip_hyphens<'a>(expr: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    let call = plain_call(expr)?;
    if call.arguments.len() != 2 {
        return None;
    }
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return None;
    };
    if literal_string(call.arguments[1].as_expression()?).as_deref() != Some("") {
        return None;
    }
    let matches_hyphen = match member.property.name.as_str() {
        "replaceAll" => literal_string(call.arguments[0].as_expression()?).as_deref() == Some("-"),
        "replace" => hyphen_regex(&call.arguments[0]),
        _ => false,
    };
    matches_hyphen.then_some(&member.object)
}

fn hyphen_regex(argument: &Argument<'_>) -> bool {
    let Argument::RegExpLiteral(regex) = argument else {
        return false;
    };
    regex.regex.pattern.text == "-" && regex.regex.flags == RegExpFlags::G
}
