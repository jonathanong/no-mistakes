use super::super::test_flake_helpers::{
    ast::{collect_branch_exprs, find_property, member_chain, non_negative_int},
    make_finding,
};
use super::RULE_ID;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{walk, Visit};

pub(super) fn scan_calls(
    file: &str,
    source: &str,
    program: &Program<'_>,
    max_retries: u64,
    findings: &mut Vec<RuleFinding>,
) {
    let mut visitor = TestVisitor {
        file,
        source,
        max_retries,
        findings,
    };
    visitor.visit_program(program);
}

pub(super) fn check_vitest_retry(
    file: &str,
    source: &str,
    expr: &Expression<'_>,
    max_retries: u64,
    label: &str,
    findings: &mut Vec<RuleFinding>,
) {
    if let Expression::ObjectExpression(obj) = unwrap_ts_wrappers(expr) {
        if let Some(count) = find_property(obj, "count") {
            check_retry_expr(
                file,
                source,
                &count.value,
                max_retries,
                label,
                "retry",
                findings,
            );
            return;
        }
    }
    check_retry_expr(file, source, expr, max_retries, label, "retry", findings);
}

pub(super) fn check_retry_expr(
    file: &str,
    source: &str,
    expr: &Expression<'_>,
    max_retries: u64,
    label: &str,
    target: &str,
    findings: &mut Vec<RuleFinding>,
) {
    let mut branches = Vec::new();
    collect_branch_exprs(expr, &mut branches);
    if let Some((count, span)) = branches
        .into_iter()
        .filter_map(non_negative_int)
        .find(|(value, _)| *value > max_retries)
    {
        findings.push(make_finding(
            RULE_ID,
            file,
            source,
            span,
            format!(
                "{label} is set to {count}, exceeding maxRetries ({max_retries}). Retries hide flaky tests; remove the retry setting and fix the cause of the flake, or add an allow entry with a reason."
            ),
            Some(target.to_string()),
        ));
    }
}

struct TestVisitor<'a> {
    file: &'a str,
    source: &'a str,
    max_retries: u64,
    findings: &'a mut Vec<RuleFinding>,
}

impl<'a> Visit<'a> for TestVisitor<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.check_call(call);
        walk::walk_call_expression(self, call);
    }
}

impl TestVisitor<'_> {
    fn check_call(&mut self, call: &CallExpression<'_>) {
        if call.optional {
            return;
        }
        if let Some(chain) = member_chain(&call.callee) {
            self.check_member_call(call, &chain);
            return;
        }
        if let Expression::CallExpression(inner) = unwrap_ts_wrappers(&call.callee) {
            if let Some(chain) = member_chain(&inner.callee) {
                if matches!(chain.first().copied(), Some("test" | "it" | "describe")) {
                    self.check_vitest_options_arg(call, chain[0]);
                }
            }
        }
    }

    fn check_member_call(&mut self, call: &CallExpression<'_>, chain: &[&str]) {
        if chain == ["jest", "retryTimes"] {
            if let Some(arg) = call.arguments.first().and_then(|arg| arg.as_expression()) {
                check_retry_expr(
                    self.file,
                    self.source,
                    arg,
                    self.max_retries,
                    "`jest.retryTimes`",
                    "jest.retryTimes",
                    self.findings,
                );
            }
            return;
        }
        if chain == ["test", "describe", "configure"] {
            self.check_playwright_configure(call);
            return;
        }
        if matches!(chain.first().copied(), Some("test" | "it" | "describe")) {
            self.check_vitest_options_arg(call, chain[0]);
        }
    }

    fn check_playwright_configure(&mut self, call: &CallExpression<'_>) {
        let Some(Expression::ObjectExpression(obj)) = call
            .arguments
            .first()
            .and_then(|arg| arg.as_expression())
            .map(unwrap_ts_wrappers)
        else {
            return;
        };
        if let Some(prop) = find_property(obj, "retries") {
            check_retry_expr(
                self.file,
                self.source,
                &prop.value,
                self.max_retries,
                "Playwright `test.describe.configure` `retries`",
                "test.describe.configure",
                self.findings,
            );
        }
    }

    fn check_vitest_options_arg(&mut self, call: &CallExpression<'_>, root_name: &str) {
        for index in [1usize, 2usize] {
            let Some(Expression::ObjectExpression(obj)) = call
                .arguments
                .get(index)
                .and_then(|arg| arg.as_expression())
                .map(unwrap_ts_wrappers)
            else {
                continue;
            };
            if let Some(prop) = find_property(obj, "retry") {
                let label = format!("Vitest `{root_name}` option `retry`");
                check_vitest_retry(
                    self.file,
                    self.source,
                    &prop.value,
                    self.max_retries,
                    &label,
                    self.findings,
                );
                return;
            }
        }
    }
}
