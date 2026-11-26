use super::super::{CollectedOrigins, CspDirective, ParsedOrigin, UnanalyzableReason};
use super::csp_strings::eval_static_string;
use super::url::{record_static_url, record_unanalyzable, record_url_expr};
use oxc_ast::ast::{
    Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName,
    JSXOpeningElement,
};
use oxc_span::GetSpan;
use std::borrow::Cow;

pub(super) fn record_jsx_element(
    collected: &mut CollectedOrigins,
    source: &str,
    element: &JSXOpeningElement<'_>,
) {
    let JSXElementName::Identifier(tag) = &element.name else {
        return;
    };
    match tag.name.as_str() {
        "script" => record_jsx_attr(collected, source, element, "src", CspDirective::Script),
        "img" => record_jsx_attr(collected, source, element, "src", CspDirective::Img),
        "iframe" => record_jsx_attr(collected, source, element, "src", CspDirective::Frame),
        "audio" | "video" | "source" => {
            record_jsx_attr(collected, source, element, "src", CspDirective::Media);
        }
        "link" => record_link_element(collected, source, element),
        _ => {}
    }
}

fn record_link_element(
    collected: &mut CollectedOrigins,
    source: &str,
    element: &JSXOpeningElement<'_>,
) {
    match link_directive(element) {
        LinkDirective::Ignore => {}
        LinkDirective::Resolved(directive) => {
            record_jsx_attr(collected, source, element, "href", directive);
        }
        LinkDirective::Dynamic => match jsx_attr_value(element, "href") {
            None => {}
            Some(JsxAttrValue::Static(url, _)) if !ParsedOrigin::is_network_like_url(&url) => {}
            Some(JsxAttrValue::Static(_, span) | JsxAttrValue::Spread(span)) => {
                record_unanalyzable(collected, source, span, UnanalyzableReason::DynamicLinkRel);
            }
            Some(JsxAttrValue::Dynamic(expr)) => record_unanalyzable(
                collected,
                source,
                expr.span().start,
                UnanalyzableReason::DynamicLinkRel,
            ),
        },
    }
}

fn record_jsx_attr(
    collected: &mut CollectedOrigins,
    source: &str,
    el: &JSXOpeningElement<'_>,
    attr: &str,
    dir: CspDirective,
) {
    match jsx_attr_value(el, attr) {
        None => {}
        Some(JsxAttrValue::Static(url, span)) => {
            record_static_url(collected, source, span, dir, &url);
        }
        Some(JsxAttrValue::Dynamic(expr)) => {
            record_url_expr(collected, source, expr, dir);
        }
        Some(JsxAttrValue::Spread(span)) => {
            record_unanalyzable(collected, source, span, UnanalyzableReason::DynamicUrl(dir));
        }
    }
}

enum JsxAttrValue<'a> {
    Static(Cow<'a, str>, u32),
    Dynamic(&'a Expression<'a>),
    Spread(u32),
}

enum LinkDirective {
    Resolved(CspDirective),
    Ignore,
    Dynamic,
}

fn link_directive(element: &JSXOpeningElement<'_>) -> LinkDirective {
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
            Some(JsxAttrValue::Dynamic(_) | JsxAttrValue::Spread(_)) => LinkDirective::Dynamic,
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

fn jsx_attr_value<'a>(element: &'a JSXOpeningElement<'a>, name: &str) -> Option<JsxAttrValue<'a>> {
    for item in element.attributes.iter().rev() {
        let attr = match item {
            JSXAttributeItem::SpreadAttribute(spread) => {
                return Some(JsxAttrValue::Spread(spread.span.start));
            }
            JSXAttributeItem::Attribute(attr) => attr,
        };
        let JSXAttributeName::Identifier(ident) = &attr.name else {
            continue;
        };
        if ident.name.as_str() != name {
            continue;
        }
        return match attr.value.as_ref()? {
            JSXAttributeValue::StringLiteral(lit) => Some(JsxAttrValue::Static(
                crate::ast::js_literal(lit),
                lit.span.start,
            )),
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
