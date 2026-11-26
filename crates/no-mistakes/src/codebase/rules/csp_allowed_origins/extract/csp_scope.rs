use super::csp_strings::{is_dynamic_policy_candidate, CspExprResult};
use super::url::{record_unanalyzable, record_url_expr};
use super::OriginVisitor;
use crate::codebase::rules::csp_allowed_origins::{CspDirective, UnanalyzableReason};
use crate::codebase::ts_source::line_number;
use oxc_ast::ast::{Argument, ArrowFunctionExpression, Expression};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;

impl OriginVisitor<'_> {
    pub(super) fn counts(&self) -> (usize, usize) {
        (
            self.collected.csp_strings.len(),
            self.collected.csp_unanalyzable.len(),
        )
    }

    pub(super) fn note_resolved_binding(&mut self, name: &str, s0: usize, u0: usize) {
        if self.collected.csp_strings.len() > s0 && self.collected.csp_unanalyzable.len() == u0 {
            self.resolved_bindings.insert(name.to_owned());
        }
    }

    pub(super) fn walk_csp_value_expr<'a>(&mut self, expr: &Expression<'a>) {
        let (s0, u0) = self.counts();
        self.visit_expression(expr);
        if self.counts() == (s0, u0) && is_dynamic_policy_candidate(expr, &self.resolved_bindings) {
            self.collected
                .csp_unanalyzable
                .push(line_number(self.source, expr.span().start));
        }
    }

    pub(super) fn walk_csp_arrow<'a>(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if let Some(expr) = arrow.body.as_expression() {
            let prev = std::mem::replace(&mut self.in_csp_fn, false);
            self.walk_csp_value_expr(expr);
            self.in_csp_fn = prev;
            return;
        }
        let prev = std::mem::replace(&mut self.in_csp_fn, true);
        walk::walk_arrow_function_expression(self, arrow);
        self.in_csp_fn = prev;
    }

    pub(super) fn visit_csp_expr<T, W>(&mut self, try_collect: T, walk_children: W)
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

    pub(super) fn record_first_arg(&mut self, args: &[Argument<'_>], directive: CspDirective) {
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
