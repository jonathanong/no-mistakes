use super::elements::jsx_element_name;
use crate::playwright::analysis::types::SelectorRef;
use crate::playwright::ast;
use crate::playwright::config::Settings;
use crate::playwright::selectors::scoped_defaults::{
    scoped_static_default_for_identifier, ScopedStaticIdentifierDefault,
};
use crate::playwright::selectors::HTML_ID_ATTRIBUTE;
use oxc_span::GetSpan;

pub(super) fn selector_refs(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    source: &str,
    settings: &Settings,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> Vec<SelectorRef> {
    let component = jsx_element_name(&opening.name)
        .and_then(|name| name.chars().next())
        .is_some_and(|ch| !ch.is_ascii_lowercase());
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

pub(super) fn has_attr(opening: &oxc_ast::ast::JSXOpeningElement<'_>, name: &str) -> bool {
    opening.attributes.iter().any(|item| {
        let oxc_ast::ast::JSXAttributeItem::Attribute(attribute) = item else {
            return false;
        };
        jsx_attribute_name(&attribute.name) == Some(name)
    })
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
                _ => None,
            }
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
