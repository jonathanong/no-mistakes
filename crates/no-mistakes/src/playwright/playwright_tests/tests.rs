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
