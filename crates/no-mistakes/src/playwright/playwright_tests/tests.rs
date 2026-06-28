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
fn extracts_describe_block_names() {
    let source = r#"
        test.describe('my suite', () => {});
        test.describe.only(`template suite`, () => {});
        test.describe.skip('skipped suite', () => {});
        test('not a describe', () => {});
        describe('also not a describe', () => {});
        test.describe(); // no name
        test.describe(123); // not a string
    "#;

    let describes =
        crate::playwright::ast::with_program(Path::new("fixture.ts"), source, |program, _| {
            let mut visitor = DescribeVisitor::default();
            visitor.visit_program(program);
            visitor.describes
        })
        .expect("fixture parses");

    assert_eq!(
        describes,
        vec![
            Some("my suite".to_string()),
            Some("template suite".to_string()),
            Some("skipped suite".to_string()),
            None,
            None,
            None,
            None,
        ]
    );
}

#[derive(Default)]
struct DescribeVisitor {
    describes: Vec<Option<String>>,
}

impl<'a> Visit<'a> for DescribeVisitor {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.describes.push(describe_name(call));
        walk::walk_call_expression(self, call);
    }
}
