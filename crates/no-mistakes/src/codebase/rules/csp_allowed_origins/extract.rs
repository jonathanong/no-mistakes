use super::{CollectedOrigins, CspDirective, OriginRef, ParsedOrigin};
use crate::codebase::ts_source::{line_number, unwrap_ts_wrappers};
use oxc_ast::ast::{
    CallExpression, Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
    JSXElementName, JSXOpeningElement, NewExpression, Program, StringLiteral, TemplateLiteral,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use std::borrow::Cow;

pub(crate) fn collect(program: &Program<'_>, source: &str) -> CollectedOrigins {
    let mut visitor = OriginVisitor {
        source,
        collected: CollectedOrigins::default(),
    };
    visitor.visit_program(program);
    visitor.collected
}

struct OriginVisitor<'a> {
    source: &'a str,
    collected: CollectedOrigins,
}

impl<'a> Visit<'a> for OriginVisitor<'_> {
    fn visit_string_literal(&mut self, lit: &StringLiteral<'a>) {
        let value = crate::ast::js_literal(lit);
        if value.contains("-src") {
            self.collected.csp_strings.push(value.into_owned());
        }
    }

    fn visit_template_literal(&mut self, template: &TemplateLiteral<'a>) {
        if let Some(value) = static_template_text(template).filter(|v| v.contains("-src")) {
            self.collected.csp_strings.push(value.into_owned());
        }
        walk::walk_template_literal(self, template);
    }

    fn visit_jsx_opening_element(&mut self, element: &JSXOpeningElement<'a>) {
        if let JSXElementName::Identifier(tag) = &element.name {
            let name = tag.name.as_str();
            let directive_and_attr = match name {
                "script" => Some((CspDirective::Script, "src")),
                "img" => Some((CspDirective::Img, "src")),
                "iframe" => Some((CspDirective::Frame, "src")),
                "audio" | "video" | "source" => Some((CspDirective::Media, "src")),
                "link" => link_directive(element).map(|dir| (dir, "href")),
                _ => None,
            };
            if let Some((directive, attr_name)) = directive_and_attr {
                if let Some((url, span_start)) = jsx_static_attr(element, attr_name) {
                    self.record_url(span_start, directive, &url);
                }
            }
        }
        walk::walk_jsx_opening_element(self, element);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_global_or_bare(&call.callee, &["fetch"]) {
            self.record_first_arg(&call.arguments, CspDirective::Connect);
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        if is_global_or_bare(&expr.callee, &["WebSocket", "EventSource"]) {
            self.record_first_arg(&expr.arguments, CspDirective::Connect);
        }
        walk::walk_new_expression(self, expr);
    }
}

impl OriginVisitor<'_> {
    fn record_first_arg(
        &mut self,
        args: &[oxc_ast::ast::Argument<'_>],
        directive: CspDirective,
    ) {
        if let Some(first) = args.first().and_then(|arg| arg.as_expression()) {
            if let Some(url) = static_string(first) {
                self.record_url(first.span().start, directive, &url);
            }
        }
    }

    fn record_url(&mut self, span_start: u32, directive: CspDirective, url: &str) {
        if ParsedOrigin::parse(url).is_some() {
            self.collected.refs.push(OriginRef {
                line: line_number(self.source, span_start),
                directive,
                url: url.to_owned(),
            });
        }
    }
}

fn link_directive(element: &JSXOpeningElement<'_>) -> Option<CspDirective> {
    let rel = jsx_static_attr(element, "rel")?.0.to_ascii_lowercase();
    let tokens: Vec<&str> = rel.split_whitespace().collect();
    if tokens.contains(&"stylesheet") {
        return Some(CspDirective::Style);
    }
    if tokens.contains(&"modulepreload") {
        return Some(CspDirective::Script);
    }
    if tokens.contains(&"icon") {
        return Some(CspDirective::Img);
    }
    if tokens.contains(&"preload") {
        let as_attr = jsx_static_attr(element, "as")
            .map(|(v, _)| v.to_ascii_lowercase())
            .unwrap_or_default();
        return Some(preload_directive(&as_attr));
    }
    (!is_non_fetch_link_rel(&tokens)).then_some(CspDirective::Default)
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

fn jsx_static_attr<'a>(element: &'a JSXOpeningElement<'a>, name: &str) -> Option<(Cow<'a, str>, u32)> {
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
        let value = attr.value.as_ref()?;
        return match value {
            JSXAttributeValue::StringLiteral(lit) => {
                Some((crate::ast::js_literal(lit), lit.span.start))
            }
            JSXAttributeValue::ExpressionContainer(container) => {
                let expr = container.expression.as_expression()?;
                Some((static_string(expr)?, expr.span().start))
            }
            _ => None,
        };
    }
    None
}

fn is_global_or_bare(callee: &Expression<'_>, names: &[&str]) -> bool {
    match unwrap_ts_wrappers(callee) {
        Expression::Identifier(id) => names.contains(&id.name.as_str()),
        Expression::StaticMemberExpression(member) if !member.optional => {
            names.contains(&member.property.name.as_str())
                && matches!(
                    unwrap_ts_wrappers(&member.object),
                    Expression::Identifier(obj)
                        if matches!(obj.name.as_str(), "window" | "globalThis" | "self")
                )
        }
        _ => false,
    }
}

fn static_template_text<'a>(template: &'a TemplateLiteral<'a>) -> Option<Cow<'a, str>> {
    if !template.expressions.is_empty() {
        return None;
    }
    let quasi = template.quasis.first()?;
    quasi.value.cooked?;
    Some(crate::ast::template_quasi_text(quasi))
}

fn static_string<'a>(expression: &'a Expression<'a>) -> Option<Cow<'a, str>> {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(value) => Some(crate::ast::js_literal(value)),
        Expression::TemplateLiteral(template) => static_template_text(template),
        _ => None,
    }
}
