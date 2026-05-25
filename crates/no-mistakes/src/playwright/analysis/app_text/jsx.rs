use super::elements::is_component_jsx_element_name;
use crate::playwright::analysis::types::SelectorRef;
use crate::playwright::ast;
use crate::playwright::config::Settings;
use crate::playwright::selectors::scoped_defaults::{
    scoped_static_default_for_identifier, ScopedStaticIdentifierDefault,
};
use crate::playwright::selectors::HTML_ID_ATTRIBUTE;
use oxc_span::GetSpan;

mod attrs;
pub(super) use attrs::{aria_bool_attr, attr_exists_at_runtime, bool_attr, numeric_attr};

pub(super) fn selector_refs(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    source: &str,
    settings: &Settings,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> Vec<SelectorRef> {
    let component = is_component_jsx_element_name(&opening.name);
    let mut refs = Vec::new();
    for item in &opening.attributes {
        let oxc_ast::ast::JSXAttributeItem::Attribute(attribute) = item else {
            continue;
        };
        let Some(name) = jsx_attribute_name(&attribute.name) else {
            continue;
        };
        let mapped = if settings.selector_attributes.iter().any(|attr| attr == name) {
            Some(name)
        } else if settings.html_ids && !component && name == HTML_ID_ATTRIBUTE {
            Some(HTML_ID_ATTRIBUTE)
        } else if component {
            settings
                .component_selector_attributes
                .get(name)
                .map(String::as_str)
        } else {
            None
        };
        let Some(attribute_name) = mapped else {
            continue;
        };
        let Some(value) = jsx_attr_string(
            attribute.value.as_ref(),
            source,
            scoped_static_identifier_defaults,
        ) else {
            continue;
        };
        refs.push(SelectorRef {
            attribute: attribute_name.to_string(),
            value,
        });
    }
    refs.sort();
    refs.dedup();
    refs
}

pub(super) fn string_attr(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    name: &str,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> Option<String> {
    for item in &opening.attributes {
        let oxc_ast::ast::JSXAttributeItem::Attribute(attribute) = item else {
            continue;
        };
        if jsx_attribute_name(&attribute.name) == Some(name) {
            return jsx_attr_string(
                attribute.value.as_ref(),
                source,
                scoped_static_identifier_defaults,
            );
        }
    }
    None
}

fn jsx_attr_string(
    value: Option<&oxc_ast::ast::JSXAttributeValue<'_>>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> Option<String> {
    match value? {
        oxc_ast::ast::JSXAttributeValue::StringLiteral(literal) => Some(literal.value.to_string()),
        oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container) => {
            match &container.expression {
                oxc_ast::ast::JSXExpression::StringLiteral(literal) => {
                    Some(literal.value.to_string())
                }
                oxc_ast::ast::JSXExpression::TemplateLiteral(template)
                    if template.expressions.is_empty() =>
                {
                    Some(ast::template_literal_text(template, source))
                }
                oxc_ast::ast::JSXExpression::Identifier(identifier) => {
                    scoped_static_default_for_identifier(
                        identifier.name.as_str(),
                        identifier.span(),
                        scoped_static_identifier_defaults,
                        source,
                    )
                }
                oxc_ast::ast::JSXExpression::TSAsExpression(expression) => {
                    jsx_expression_string(&expression.expression, source)
                }
                oxc_ast::ast::JSXExpression::TSSatisfiesExpression(expression) => {
                    jsx_expression_string(&expression.expression, source)
                }
                oxc_ast::ast::JSXExpression::TSNonNullExpression(expression) => {
                    jsx_expression_string(&expression.expression, source)
                }
                oxc_ast::ast::JSXExpression::TSTypeAssertion(expression) => {
                    jsx_expression_string(&expression.expression, source)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn jsx_expression_string(
    expression: &oxc_ast::ast::Expression<'_>,
    source: &str,
) -> Option<String> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        oxc_ast::ast::Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        oxc_ast::ast::Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(ast::template_literal_text(template, source))
        }
        _ => None,
    }
}

fn jsx_attribute_name<'a>(name: &'a oxc_ast::ast::JSXAttributeName<'a>) -> Option<&'a str> {
    match name {
        oxc_ast::ast::JSXAttributeName::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}
