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

fn get_test_callback_status(source: &str) -> Option<TestStatus> {
    crate::playwright::ast::with_program(Path::new("fixture.ts"), source, |program, _| {
        let mut visitor = OuterCallVisitor::default();
        visitor.visit_program(program);
        visitor.status
    })
    .expect("fixture parses")
    .flatten()
}

#[derive(Default)]
struct OuterCallVisitor {
    status: Option<Option<TestStatus>>,
}

impl<'a> Visit<'a> for OuterCallVisitor {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.status.is_none() {
            self.status = Some(test_callback_status(call));
        }
        walk::walk_call_expression(self, call);
    }
}

#[test]
fn test_callback_status_identifies_all_statuses() {
    let cases = vec![
        ("test('active', () => {});", Some(TestStatus::Active)),
        (
            "test.skipIf(condition)('skip if', () => {});",
            Some(TestStatus::Conditional),
        ),
        ("not_a_test('foo', () => {});", None),
        (
            "test.describe('describe', () => {});",
            Some(TestStatus::Active),
        ),
        (
            "test.skip('skip string', () => {});",
            Some(TestStatus::Skipped),
        ),
        ("test.skip(() => {});", None),
        (
            "test.describe.skip('describe skip string', () => {});",
            Some(TestStatus::Skipped),
        ),
        ("test.describe.skip(() => {});", Some(TestStatus::Skipped)),
        (
            "test.fixme('fixme string', () => {});",
            Some(TestStatus::Skipped),
        ),
        ("test.only('only', () => {});", Some(TestStatus::Active)),
        ("test.skip(true, 'skip bool');", None),
        ("test.skip(condition, 'skip conditional');", None),
        ("test('no callback');", None),
        ("test.fail('fail string', () => {});", Some(TestStatus::Active)),
        (
            "test.skip(`template string`, () => {});",
            Some(TestStatus::Skipped),
        ),
        (
            "test.describe.fixme('describe fixme string', () => {});",
            Some(TestStatus::Skipped),
        ),
        ("test.describe.fixme(() => {});", Some(TestStatus::Skipped)),
    ];

    for (source, expected) in cases {
        let status = get_test_callback_status(source);
        assert_eq!(status, expected, "Failed for source: {}", source);
    }
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
