use super::super::super::CspDirective;
use super::eval::{eval_static_string, unwrap_join_array_target};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, BinaryOperator, Expression,
    ObjectExpression, ObjectPropertyKind, PropertyKey, TemplateLiteral,
};

pub(in super::super) fn is_static_csp_policy_string(text: &str) -> bool {
    text.split([',', ';']).any(|part| {
        let mut tokens = part.split_whitespace();
        let Some(name) = tokens.next() else {
            return false;
        };
        CspDirective::parse(name).is_some() && tokens.next().is_some()
    })
}

pub(in super::super) fn is_csp_identifier(name: &str) -> bool {
    let words = split_identifier_words(name);
    words.iter().any(|word| word == "csp")
        || words
            .windows(3)
            .any(|win| win == ["content", "security", "policy"])
}

fn split_identifier_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (idx, &ch) in chars.iter().enumerate() {
        if !ch.is_ascii_alphanumeric() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }
        let prev_lower = idx > 0 && chars[idx - 1].is_ascii_lowercase();
        let next_lower = idx + 1 < chars.len() && chars[idx + 1].is_ascii_lowercase();
        if ch.is_ascii_uppercase() && !current.is_empty() && (prev_lower || next_lower) {
            words.push(std::mem::take(&mut current));
        }
        current.push(ch.to_ascii_lowercase());
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn property_key_text<'a>(key: &'a PropertyKey<'a>) -> Option<std::borrow::Cow<'a, str>> {
    match key {
        PropertyKey::StringLiteral(lit) => Some(crate::ast::js_literal(lit)),
        PropertyKey::StaticIdentifier(ident) => Some(std::borrow::Cow::Borrowed(ident.name.as_str())),
        _ => None,
    }
}

pub(super) fn is_csp_header_name(raw: &str) -> bool {
    raw.trim().eq_ignore_ascii_case("content-security-policy")
}

pub(super) fn object_is_csp_source_map(obj: &ObjectExpression<'_>) -> bool {
    obj.properties.iter().any(|prop| match prop {
        ObjectPropertyKind::ObjectProperty(prop) => {
            !prop.computed
                && property_key_text(&prop.key)
                    .is_some_and(|key| CspDirective::parse(key.trim()).is_some())
        }
        ObjectPropertyKind::SpreadProperty(_) => false,
    })
}

pub(super) fn csp_header_object_value<'a>(
    obj: &'a ObjectExpression<'a>,
) -> Option<&'a Expression<'a>> {
    let mut has_csp_key_field = false;
    let mut value_field = None;
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        if prop.computed {
            continue;
        }
        let Some(key) = property_key_text(&prop.key) else {
            continue;
        };
        if is_csp_header_name(&key) {
            return Some(&prop.value);
        }
        if key.eq_ignore_ascii_case("key") || key.eq_ignore_ascii_case("name") {
            if eval_static_string(&prop.value).is_some_and(|val| is_csp_header_name(&val)) {
                has_csp_key_field = true;
            }
        } else if key.eq_ignore_ascii_case("value") {
            value_field = Some(&prop.value);
        }
    }
    has_csp_key_field.then_some(value_field).flatten()
}

pub(super) fn object_contains_csp_source_map(obj: &ObjectExpression<'_>) -> bool {
    object_is_csp_source_map(obj)
        || obj.properties.iter().any(|prop| match prop {
            ObjectPropertyKind::ObjectProperty(prop) => expr_contains_csp_fragment(&prop.value),
            ObjectPropertyKind::SpreadProperty(spread) => {
                expr_contains_csp_fragment(&spread.argument)
            }
        })
}

pub(super) fn arg_contains_csp_fragment(arg: &Argument<'_>) -> bool {
    match arg {
        Argument::SpreadElement(spread) => expr_contains_csp_fragment(&spread.argument),
        other => expr_contains_csp_fragment(other.to_expression()),
    }
}

pub(super) fn array_contains_csp_fragment(array: &ArrayExpression<'_>) -> bool {
    array.elements.iter().any(|element| match element {
        ArrayExpressionElement::SpreadElement(spread) => {
            expr_contains_csp_fragment(&spread.argument)
        }
        other => other
            .as_expression()
            .is_some_and(expr_contains_csp_fragment),
    })
}

pub(super) fn template_contains_csp_fragment(template: &TemplateLiteral<'_>) -> bool {
    template.quasis.iter().any(|quasi| {
        quasi.value.cooked.is_some() && crate::ast::template_quasi_text(quasi).contains("-src")
    }) || template.expressions.iter().any(expr_contains_csp_fragment)
}

pub(super) fn expr_contains_csp_fragment(expr: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(lit) => crate::ast::js_literal(lit).contains("-src"),
        Expression::TemplateLiteral(tpl) => template_contains_csp_fragment(tpl),
        Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
            expr_contains_csp_fragment(&bin.left) || expr_contains_csp_fragment(&bin.right)
        }
        Expression::ConditionalExpression(cond) => {
            expr_contains_csp_fragment(&cond.consequent)
                || expr_contains_csp_fragment(&cond.alternate)
        }
        Expression::LogicalExpression(logical) => {
            expr_contains_csp_fragment(&logical.left) || expr_contains_csp_fragment(&logical.right)
        }
        Expression::ArrayExpression(array) => array_contains_csp_fragment(array),
        Expression::ObjectExpression(obj) => {
            csp_header_object_value(obj).is_none() && object_contains_csp_source_map(obj)
        }
        Expression::CallExpression(call) => {
            if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) {
                if !member.optional
                    && member.property.name.as_str() == "join"
                    && unwrap_join_array_target(&member.object)
                        .is_some_and(|(array, _)| array_contains_csp_fragment(array))
                {
                    return true;
                }
            }
            call.arguments.iter().any(arg_contains_csp_fragment)
        }
        _ => false,
    }
}
