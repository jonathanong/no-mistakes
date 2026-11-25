use super::{CollectedOrigins, CspDirective, ParsedOrigin, UnanalyzableReason};
use crate::codebase::ts_source::line_number;
use oxc_ast::ast::{
    Argument, ArrayExpression, BinaryExpression, CallExpression, ConditionalExpression,
    JSXElementName, JSXOpeningElement, LogicalExpression, NewExpression, Program, StringLiteral,
    TemplateLiteral,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;

mod csp_strings;
mod jsx;
mod url;

use csp_strings::{
    try_collect_array, try_collect_binary, try_collect_conditional, try_collect_join_call,
    try_collect_logical, try_collect_template, CspExprResult,
};
use jsx::{jsx_attr_value, link_directive, JsxAttrValue, LinkDirective};
use url::{is_global_or_bare, record_static_url, record_unanalyzable, record_url_expr};

pub(crate) fn collect(program: &Program<'_>, source: &str) -> CollectedOrigins {
    let mut visitor = OriginVisitor {
        source,
        collected: CollectedOrigins::default(),
        csp_depth: 0,
    };
    visitor.visit_program(program);
    visitor.collected
}

struct OriginVisitor<'a> {
    source: &'a str,
    collected: CollectedOrigins,
    csp_depth: usize,
}

impl<'a> Visit<'a> for OriginVisitor<'_> {
    fn visit_string_literal(&mut self, lit: &StringLiteral<'a>) {
        if self.csp_depth == 0 {
            let value = crate::ast::js_literal(lit);
            if value.contains("-src") {
                self.collected.csp_strings.push(value.into_owned());
            }
        }
    }

    fn visit_template_literal(&mut self, tpl: &TemplateLiteral<'a>) {
        self.visit_csp_expr(
            |out| try_collect_template(tpl, out),
            |this| walk::walk_template_literal(this, tpl),
        );
    }

    fn visit_array_expression(&mut self, array: &ArrayExpression<'a>) {
        self.visit_csp_expr(
            |out| try_collect_array(array, out),
            |this| walk::walk_array_expression(this, array),
        );
    }

    fn visit_binary_expression(&mut self, expr: &BinaryExpression<'a>) {
        self.visit_csp_expr(
            |out| try_collect_binary(expr, out),
            |this| walk::walk_binary_expression(this, expr),
        );
    }

    fn visit_conditional_expression(&mut self, expr: &ConditionalExpression<'a>) {
        self.visit_csp_expr(
            |_| try_collect_conditional(expr),
            |this| walk::walk_conditional_expression(this, expr),
        );
    }

    fn visit_logical_expression(&mut self, expr: &LogicalExpression<'a>) {
        self.visit_csp_expr(
            |_| try_collect_logical(expr),
            |this| walk::walk_logical_expression(this, expr),
        );
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
        self.visit_csp_expr(
            |out| try_collect_join_call(call, out),
            |this| walk::walk_call_expression(this, call),
        );
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        if is_global_or_bare(&expr.callee, &["WebSocket", "EventSource"]) {
            self.record_first_arg(&expr.arguments, CspDirective::Connect);
        }
        walk::walk_new_expression(self, expr);
    }
}

impl OriginVisitor<'_> {
    fn visit_csp_expr<T, W>(&mut self, try_collect: T, walk_children: W)
    where
        T: FnOnce(&mut Vec<String>) -> CspExprResult,
        W: FnOnce(&mut Self),
    {
        let result = if self.csp_depth == 0 {
            try_collect(&mut self.collected.csp_strings)
        } else {
            CspExprResult::Ignored
        };
        match result {
            CspExprResult::Resolved => {
                self.csp_depth += 1;
                walk_children(self);
                self.csp_depth -= 1;
            }
            CspExprResult::Unanalyzable(span) => {
                self.collected
                    .csp_unanalyzable
                    .push(line_number(self.source, span));
                self.csp_depth += 1;
                walk_children(self);
                self.csp_depth -= 1;
            }
            CspExprResult::Ignored => walk_children(self),
        }
    }

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
