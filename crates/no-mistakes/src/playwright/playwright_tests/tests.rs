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
fn test_callback_identity_extracts_test_name() {
    let source = r#"
        test('my test name', () => {});
        test.only(`my template test name`, () => {});
        test.skip(`my ${dynamic} test name`, () => {});
        it('is not a playwright test', () => {});
        describe('is not a playwright test', () => {});
        test.describe('is a describe, not a test', () => {});
    "#;

    let names =
        crate::playwright::ast::with_program(Path::new("fixture.ts"), source, |program, _| {
            let mut visitor = TestNameVisitor::default();
            visitor.visit_program(program);
            visitor.names
        })
        .expect("fixture parses");

    assert_eq!(
        names,
        vec![
            ("test".to_string(), Some("my test name".to_string())),
            (
                "test.only".to_string(),
                Some("my template test name".to_string())
            ),
            ("test.skip".to_string(), None),
            ("it".to_string(), None),
            ("describe".to_string(), None),
            (
                "test.describe".to_string(),
                Some("is a describe, not a test".to_string())
            ),
        ]
    );
}

#[derive(Default)]
struct TestNameVisitor {
    names: Vec<(String, Option<String>)>,
}

impl<'a> Visit<'a> for TestNameVisitor {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(path) = crate::playwright::ast::expression_path(&call.callee) {
            let path_str = path.join(".");
            // only care about top level calls for testing
            if path.len() <= 2 {
                self.names.push((path_str, test_callback_identity(call)));
            }
        }
        walk::walk_call_expression(self, call);
    }
}
