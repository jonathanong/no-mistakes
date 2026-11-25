use super::csp_strings::eval_static_string;
use super::super::CspDirective;
use oxc_ast::ast::{
    Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXOpeningElement,
};
use oxc_span::GetSpan;
use std::borrow::Cow;

pub(super) enum JsxAttrValue<'a> {
    Static(Cow<'a, str>, u32),
    Dynamic(&'a Expression<'a>),
}

pub(super) enum LinkDirective {
    Resolved(CspDirective),
    Ignore,
    Dynamic,
}

pub(super) fn link_directive(element: &JSXOpeningElement<'_>) -> LinkDirective {
    let Some(rel_attr) = jsx_attr_value(element, "rel") else {
        return LinkDirective::Ignore;
    };
    let JsxAttrValue::Static(rel_raw, _) = rel_attr else {
        return LinkDirective::Dynamic;
    };
    let rel = rel_raw.to_ascii_lowercase();
    let tokens: Vec<&str> = rel.split_whitespace().collect();
    if tokens.contains(&"stylesheet") {
        return LinkDirective::Resolved(CspDirective::Style);
    }
    if tokens.contains(&"modulepreload") {
        return LinkDirective::Resolved(CspDirective::Script);
    }
    if tokens.contains(&"icon") {
        return LinkDirective::Resolved(CspDirective::Img);
    }
    if tokens.contains(&"preload") {
        return match jsx_attr_value(element, "as") {
            None => LinkDirective::Resolved(CspDirective::Default),
            Some(JsxAttrValue::Static(as_raw, _)) => {
                LinkDirective::Resolved(preload_directive(&as_raw.to_ascii_lowercase()))
            }
            Some(JsxAttrValue::Dynamic(_)) => LinkDirective::Dynamic,
        };
    }
    if is_non_fetch_link_rel(&tokens) {
        LinkDirective::Ignore
    } else {
        LinkDirective::Resolved(CspDirective::Default)
    }
}

fn preload_directive(as_attr: &str) -> CspDirective {
    match as_attr {
        "script" => CspDirective::Script,
        "style" => CspDirective::Style,
        "image" => CspDirective::Img,
        "font" => CspDirective::Font,
        "audio" | "video" => CspDirective::Media,
        "fetch" => CspDirective::Connect,
        _ => CspDirective::Default,
    }
}

fn is_non_fetch_link_rel(tokens: &[&str]) -> bool {
    tokens.iter().any(|token| {
        matches!(
            *token,
            "preconnect" | "dns-prefetch" | "canonical" | "alternate" | "author" | "license"
        )
    })
}

pub(super) fn jsx_attr_value<'a>(
    element: &'a JSXOpeningElement<'a>,
    name: &str,
) -> Option<JsxAttrValue<'a>> {
    for item in &element.attributes {
        let JSXAttributeItem::Attribute(attr) = item else {
            continue;
        };
        let JSXAttributeName::Identifier(ident) = &attr.name else {
            continue;
        };
        if ident.name.as_str() != name {
            continue;
        }
        return match attr.value.as_ref()? {
            JSXAttributeValue::StringLiteral(lit) => {
                Some(JsxAttrValue::Static(crate::ast::js_literal(lit), lit.span.start))
            }
            JSXAttributeValue::ExpressionContainer(container) => {
                let expr = container.expression.as_expression()?;
                match eval_static_string(expr) {
                    Some(value) => Some(JsxAttrValue::Static(value, expr.span().start)),
                    None => Some(JsxAttrValue::Dynamic(expr)),
                }
            }
            _ => None,
        };
    }
    None
}
