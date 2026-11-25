use super::{CollectedOrigins, CspDirective, ParsedOrigin, UnanalyzableReason};
use oxc_ast::ast::{
    Argument, ArrayExpression, BinaryExpression, CallExpression, JSXElementName, JSXOpeningElement,
    NewExpression, Program, StringLiteral, TemplateLiteral,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;

mod csp_strings;
mod jsx;
mod url;

use csp_strings::{
    static_template_text, try_collect_array, try_collect_binary, try_collect_join_call,
};
use jsx::{jsx_attr_value, link_directive, JsxAttrValue, LinkDirective};
use url::{is_global_or_bare, record_static_url, record_unanalyzable, record_url_expr};

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

    fn visit_array_expression(&mut self, array: &ArrayExpression<'a>) {
        if try_collect_array(array, &mut self.collected.csp_strings) {
            return;
        }
        walk::walk_array_expression(self, array);
    }

    fn visit_binary_expression(&mut self, expr: &BinaryExpression<'a>) {
        if try_collect_binary(expr, &mut self.collected.csp_strings) {
            return;
        }
        walk::walk_binary_expression(self, expr);
    }

    fn visit_jsx_opening_element(&mut self, element: &JSXOpeningElement<'a>) {
        if let JSXElementName::Identifier(tag) = &element.name {
            match tag.name.as_str() {
                "script" => self.record_jsx_attr(element, "src", CspDirective::Script),
                "img" => self.record_jsx_attr(element, "src", CspDirective::Img),
                "iframe" => self.record_jsx_attr(element, "src", CspDirective::Frame),
                "audio" | "video" | "source" => {
                    self.record_jsx_attr(element, "src", CspDirective::Media);
                }
                "link" => self.record_link_element(element),
                _ => {}
            }
        }
        walk::walk_jsx_opening_element(self, element);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_global_or_bare(&call.callee, &["fetch"]) {
            self.record_first_arg(&call.arguments, CspDirective::Connect);
        }
        if try_collect_join_call(call, &mut self.collected.csp_strings) {
            return;
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
    fn record_link_element(&mut self, element: &JSXOpeningElement<'_>) {
        match link_directive(element) {
            LinkDirective::Ignore => {}
            LinkDirective::Resolved(directive) => self.record_jsx_attr(element, "href", directive),
            LinkDirective::Dynamic => match jsx_attr_value(element, "href") {
                None => {}
                Some(JsxAttrValue::Static(url, _)) if !ParsedOrigin::is_network_like_url(&url) => {}
                Some(JsxAttrValue::Static(_, span)) => record_unanalyzable(
                    &mut self.collected,
                    self.source,
                    span,
                    UnanalyzableReason::DynamicLinkRel,
                ),
                Some(JsxAttrValue::Dynamic(expr)) => record_unanalyzable(
                    &mut self.collected,
                    self.source,
                    expr.span().start,
                    UnanalyzableReason::DynamicLinkRel,
                ),
            },
        }
    }

    fn record_jsx_attr(&mut self, el: &JSXOpeningElement<'_>, attr: &str, dir: CspDirective) {
        match jsx_attr_value(el, attr) {
            None => {}
            Some(JsxAttrValue::Static(url, span)) => {
                record_static_url(&mut self.collected, self.source, span, dir, &url);
            }
            Some(JsxAttrValue::Dynamic(expr)) => {
                record_url_expr(&mut self.collected, self.source, expr, dir);
            }
        }
    }

    fn record_first_arg(&mut self, args: &[Argument<'_>], directive: CspDirective) {
        let Some(arg) = args.first() else {
            return;
        };
        match arg.as_expression() {
            Some(expr) => record_url_expr(&mut self.collected, self.source, expr, directive),
            None => record_unanalyzable(
                &mut self.collected,
                self.source,
                arg.span().start,
                UnanalyzableReason::DynamicUrl(directive),
            ),
        }
    }
}
