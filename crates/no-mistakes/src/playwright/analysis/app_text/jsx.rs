use crate::playwright::analysis::types::SelectorRef;
use crate::playwright::config::Settings;
use crate::playwright::selectors::HTML_ID_ATTRIBUTE;

pub(super) fn direct_child_texts(children: &[oxc_ast::ast::JSXChild<'_>]) -> Vec<String> {
    let mut results = Vec::new();
    let mut current = String::new();

    for child in children {
        match child {
            oxc_ast::ast::JSXChild::Text(text) => {
                current.push_str(text.value.as_str());
            }
            oxc_ast::ast::JSXChild::ExpressionContainer(container) => {
                if let oxc_ast::ast::JSXExpression::StringLiteral(literal) = &container.expression {
                    current.push_str(literal.value.as_str());
                } else if !current.is_empty() {
                    results.push(std::mem::take(&mut current));
                }
            }
            _ => {
                if !current.is_empty() {
                    results.push(std::mem::take(&mut current));
                }
            }
        }
    }

    if !current.is_empty() {
        results.push(current);
    }

    results
}

pub(super) fn selector_refs(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    settings: &Settings,
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
        let Some(value) = jsx_attr_string(attribute.value.as_ref()) else {
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
) -> Option<String> {
    for item in &opening.attributes {
        let oxc_ast::ast::JSXAttributeItem::Attribute(attribute) = item else {
            continue;
        };
        if jsx_attribute_name(&attribute.name) == Some(name) {
            return jsx_attr_string(attribute.value.as_ref());
        }
    }
    None
}

fn jsx_attr_string(value: Option<&oxc_ast::ast::JSXAttributeValue<'_>>) -> Option<String> {
    match value? {
        oxc_ast::ast::JSXAttributeValue::StringLiteral(literal) => Some(literal.value.to_string()),
        oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container) => {
            match &container.expression {
                oxc_ast::ast::JSXExpression::StringLiteral(literal) => {
                    Some(literal.value.to_string())
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

pub(super) fn jsx_element_name<'a>(name: &'a oxc_ast::ast::JSXElementName<'a>) -> Option<&'a str> {
    match name {
        oxc_ast::ast::JSXElementName::Identifier(identifier) => Some(identifier.name.as_str()),
        oxc_ast::ast::JSXElementName::IdentifierReference(identifier) => {
            Some(identifier.name.as_str())
        }
        oxc_ast::ast::JSXElementName::MemberExpression(expression) => {
            jsx_member_expression_root(expression)
        }
        _ => None,
    }
}

pub(super) fn element_role(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    tag: Option<&str>,
) -> Option<String> {
    if let Some(role) = string_attr(opening, "role").and_then(|value| first_role_token(&value)) {
        return Some(role);
    }
    implicit_role(opening, tag).map(str::to_string)
}

fn first_role_token(value: &str) -> Option<String> {
    value.split_whitespace().next().map(str::to_string)
}

fn implicit_role(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    tag: Option<&str>,
) -> Option<&'static str> {
    match tag? {
        "a" | "area" if string_attr(opening, "href").is_some() => Some("link"),
        "button" => Some("button"),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some("heading"),
        "img" if string_attr(opening, "alt").is_some() => Some("img"),
        "input" => input_role(opening),
        "select" => Some("combobox"),
        "textarea" => Some("textbox"),
        _ => None,
    }
}

fn input_role(opening: &oxc_ast::ast::JSXOpeningElement<'_>) -> Option<&'static str> {
    match string_attr(opening, "type").as_deref().unwrap_or("text") {
        "button" | "image" | "reset" | "submit" => Some("button"),
        "checkbox" => Some("checkbox"),
        "radio" => Some("radio"),
        "range" => Some("slider"),
        "search" | "email" | "tel" | "text" | "url" => Some("textbox"),
        _ => None,
    }
}

fn jsx_member_expression_root<'a>(
    expression: &'a oxc_ast::ast::JSXMemberExpression<'a>,
) -> Option<&'a str> {
    match &expression.object {
        oxc_ast::ast::JSXMemberExpressionObject::IdentifierReference(identifier) => {
            Some(identifier.name.as_str())
        }
        oxc_ast::ast::JSXMemberExpressionObject::MemberExpression(expression) => {
            jsx_member_expression_root(expression)
        }
        oxc_ast::ast::JSXMemberExpressionObject::ThisExpression(_) => None,
    }
}
