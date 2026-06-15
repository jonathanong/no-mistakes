use super::super::{TestOccurrenceScope, TestStatus};
use super::build_call_context_index;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn parse_index(source: &str) -> std::collections::HashMap<u32, super::CallContext> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    build_call_context_index(&ret.program, source)
}

fn ctx_at(
    index: &std::collections::HashMap<u32, super::CallContext>,
    line: u32,
) -> super::CallContext {
    *index
        .get(&line)
        .unwrap_or_else(|| panic!("no context recorded for line {line}; have {index:?}"))
}

#[test]
fn records_file_scope_calls() {
    let source = "doSomething();\nfetch('/x');\n";
    let index = parse_index(source);
    let line_1 = ctx_at(&index, 1);
    assert_eq!(line_1.scope, TestOccurrenceScope::File);
    assert_eq!(line_1.status, TestStatus::Active);
    let line_2 = ctx_at(&index, 2);
    assert_eq!(line_2.scope, TestOccurrenceScope::File);
}

#[test]
fn records_test_scope_calls() {
    let source = "import { test } from '@playwright/test';\n\
                  test('case', async () => {\n  fetch('/x');\n});\n";
    let index = parse_index(source);
    // `fetch('/x')` is on line 3 inside the test callback.
    assert_eq!(ctx_at(&index, 3).scope, TestOccurrenceScope::Test);
}

#[test]
fn records_setup_hook_scope() {
    let source = "import { test } from '@playwright/test';\n\
                  test.beforeEach(async () => {\n  fetch('/x');\n});\n";
    let index = parse_index(source);
    assert_eq!(ctx_at(&index, 3).scope, TestOccurrenceScope::Hook);
}

#[test]
fn records_teardown_hook_scope() {
    let source = "import { test } from '@playwright/test';\n\
                  test.afterEach(async () => {\n  fetch('/cleanup');\n});\n";
    let index = parse_index(source);
    assert_eq!(ctx_at(&index, 3).scope, TestOccurrenceScope::TeardownHook);
}

#[test]
fn skipped_tests_propagate_status() {
    let source = "import { test } from '@playwright/test';\n\
                  test.skip('case', async () => {\n  fetch('/x');\n});\n";
    let index = parse_index(source);
    let ctx = ctx_at(&index, 3);
    assert_eq!(ctx.status, TestStatus::Skipped);
}

#[test]
fn if_statement_demotes_to_conditional() {
    let source = "import { test } from '@playwright/test';\n\
                  test('case', () => {\n  if (cond) {\n    fetch('/x');\n  }\n});\n";
    let index = parse_index(source);
    let ctx = ctx_at(&index, 4);
    assert_eq!(ctx.status, TestStatus::Conditional);
}

#[test]
fn describe_block_does_not_mask_inner_scope() {
    let source = "import { test } from '@playwright/test';\n\
                  test.describe('group', () => {\n  \
                    test('inner', () => {\n    \
                      fetch('/x');\n  \
                    });\n\
                  });\n";
    let index = parse_index(source);
    assert_eq!(ctx_at(&index, 4).scope, TestOccurrenceScope::Test);
}

#[test]
fn conditional_expression_demotes_to_conditional() {
    // The ternary's consequent and alternate are visited under
    // status_for_if_branch, so a call inside either branch should surface
    // as Conditional even at file scope.
    let source = "const x = cond ? fetch('/yes') : fetch('/no');\n";
    let index = parse_index(source);
    let ctx = ctx_at(&index, 1);
    assert_eq!(ctx.status, TestStatus::Conditional);
}

#[test]
fn logical_expression_right_arm_demotes_to_conditional() {
    let source = "cond && fetch('/x');\n";
    let index = parse_index(source);
    let ctx = ctx_at(&index, 1);
    assert_eq!(ctx.status, TestStatus::Conditional);
}

#[test]
fn annotation_call_demotes_following_calls() {
    // `test.skip()` called as a runtime annotation inside a test body — not
    // as a test wrapper — should mark calls after it as Skipped.
    let source = "import { test } from '@playwright/test';\n\
                  test('case', async () => {\n  \
                    test.skip();\n  \
                    fetch('/x');\n\
                  });\n";
    let index = parse_index(source);
    assert_eq!(ctx_at(&index, 4).status, TestStatus::Skipped);
}

#[test]
fn annotation_call_visits_non_callback_args() {
    // `test.skip(condition, reason)` annotations carry non-callback
    // arguments (boolean conditions, string messages) that still need to
    // be visited so any nested calls inside them get recorded.
    let source = "import { test } from '@playwright/test';\n\
                  test('case', () => {\n  \
                    test.skip(shouldSkip(), describe('why'));\n\
                  });\n";
    let index = parse_index(source);
    // `shouldSkip()` is on line 3 and visited as a non-callback arg of
    // the annotation. The scope stays Test (the surrounding test() body);
    // the call gets recorded — exactly what coverage cares about here.
    let ctx = ctx_at(&index, 3);
    assert_eq!(ctx.scope, TestOccurrenceScope::Test);
}

#[test]
fn if_else_alternate_branch_visited() {
    // The alternate branch of an if/else should also be visited under the
    // demoted Conditional status.
    let source = "import { test } from '@playwright/test';\n\
                  test('case', () => {\n  \
                    if (cond) { fetch('/a'); } else { fetch('/b'); }\n\
                  });\n";
    let index = parse_index(source);
    let ctx = ctx_at(&index, 3);
    assert_eq!(ctx.status, TestStatus::Conditional);
}

#[test]
fn setup_hook_scope_outranks_inner_test_scope() {
    // A test() inside a beforeEach is unusual but the merge should keep
    // the more restrictive Hook scope when both contexts touch the same
    // line. This also covers scope_rank's Hook arm.
    let source = "import { test } from '@playwright/test';\n\
                  test.beforeEach(async () => { test('inner', () => {}); });\n";
    let index = parse_index(source);
    // Line 2 hosts both the beforeEach call (Hook) and the inner test()
    // call (Test); Hook outranks Test, so the recorded scope is Hook.
    assert_eq!(ctx_at(&index, 2).scope, TestOccurrenceScope::Hook);
}

#[test]
fn nested_calls_keep_most_restrictive_scope() {
    // A `test.afterEach` containing both an inner call and the hook itself
    // would record line 2 twice. The first record (the hook call itself) is
    // File scope; the inner fetch is TeardownHook. The merge keeps the
    // more restrictive scope on the line that hosts both.
    let source = "import { test } from '@playwright/test';\n\
                  test.afterEach(async () => { fetch('/x'); });\n";
    let index = parse_index(source);
    // The fetch on line 2 should win out at TeardownHook (vs File for the
    // surrounding hook call).
    assert_eq!(ctx_at(&index, 2).scope, TestOccurrenceScope::TeardownHook);
}
