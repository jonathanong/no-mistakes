use super::*;
use oxc_ast::ast::CallExpression;
use oxc_ast_visit::{walk, Visit};
use std::path::Path;

#[test]
fn hook_callback_index_matches_setup_and_teardown_hooks() {
    let source = r#"
        test.beforeEach(async ({ page }) => {});
        test.beforeAll(async ({ page }) => {});
        test.afterEach(async ({ page }) => {});
        test.afterAll(async ({ page }) => {});
    "#;

    let hooks =
        crate::playwright::ast::with_program(Path::new("fixture.ts"), source, |program, _| {
            let mut visitor = HookVisitor::default();
            visitor.visit_program(program);
            visitor.hooks
        })
        .expect("fixture parses");

    assert_eq!(
        hooks,
        vec![
            ("test.beforeEach".to_string(), true),
            ("test.beforeAll".to_string(), true),
            ("test.afterEach".to_string(), true),
            ("test.afterAll".to_string(), true),
        ]
    );
}

#[derive(Default)]
struct HookVisitor {
    hooks: Vec<(String, bool)>,
}

impl<'a> Visit<'a> for HookVisitor {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(path) = crate::playwright::ast::expression_path(&call.callee) {
            if path.first().map(String::as_str) == Some("test") {
                self.hooks
                    .push((path.join("."), hook_callback(call).is_some()));
            }
        }
        walk::walk_call_expression(self, call);
    }
}

#[test]
fn describe_name_extracts_correct_names() {
    let source = r#"
        test.describe('single quotes', () => {});
        test.describe("double quotes", () => {});
        test.describe(`template literal`, () => {});
        test.describe.only('modifier only', () => {});
        test.describe.parallel.skip('multiple modifiers', () => {});
        test.describe();
        test.describe(`${variable} template`, () => {});
        test('not describe', () => {});
        describe('no test prefix', () => {});
    "#;

    let names =
        crate::playwright::ast::with_program(Path::new("fixture.ts"), source, |program, _| {
            let mut visitor = DescribeNameVisitor::default();
            visitor.visit_program(program);
            visitor.names
        })
        .expect("fixture parses");

    assert_eq!(
        names,
        vec![
            Some("single quotes".to_string()),
            Some("double quotes".to_string()),
            Some("template literal".to_string()),
            Some("modifier only".to_string()),
            Some("multiple modifiers".to_string()),
            None,
            None,
            None,
            None,
        ]
    );
}

#[derive(Default)]
struct DescribeNameVisitor {
    names: Vec<Option<String>>,
}

impl<'a> Visit<'a> for DescribeNameVisitor {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(path) = crate::playwright::ast::expression_path(&call.callee) {
            if path.first().map(String::as_str) == Some("test")
                || path.first().map(String::as_str) == Some("describe")
            {
                let name = describe_name(call);
                // Collect results for all test/describe related calls to verify negative cases too
                // (except test.describe() inner parts, so we just check if it's a top level call in our context)
                self.names.push(name);
            }
        }
        walk::walk_call_expression(self, call);
    }
}