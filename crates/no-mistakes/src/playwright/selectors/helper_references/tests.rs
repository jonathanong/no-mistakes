use super::*;
use crate::playwright::{ast, playwright_tests};

#[test]
fn helper_references_skip_counted_locator_calls() {
    let source = r#"
import { test } from '@playwright/test';

function getAsideLocator(page, dataPw) {
  return page.getByTestId(dataPw).first();
}

test('uses helper', async ({ page }) => {
  await page.getByTestId('direct-button').click();
  await page.getByRole('button', { name: 'role-button' }).click();
  await page.getByText('text-button').click();
  await expect(locator).toBeVisible('matcher-noise');
  await toggleSidebar(page, 'toggle-helper').click();
  await getAsideLocator(page, 'helper-button').click();
});
"#;

    let references = ast::with_program(
        std::path::Path::new("helper.ts"),
        source,
        |program, source| {
            extract_playwright_helper_reference_occurrences_from_program(program, source)
        },
    )
    .expect("fixture should parse");

    assert!(references.iter().any(|reference| {
        reference.value.value == "helper-button"
            && reference.value.call == "getAsideLocator(...)"
            && reference.scope == playwright_tests::TestOccurrenceScope::Test
    }));
    assert!(references.iter().any(|reference| {
        reference.value.value == "toggle-helper"
            && reference.value.call == "toggleSidebar(...)"
            && reference.scope == playwright_tests::TestOccurrenceScope::Test
    }));
    for native_value in [
        "direct-button",
        "role-button",
        "text-button",
        "matcher-noise",
    ] {
        assert!(
            references
                .iter()
                .all(|reference| reference.value.value != native_value),
            "native locator calls must not be helper-reference hints: {references:?}"
        );
    }
}

#[test]
fn helper_references_preserve_status_and_scope_edges() {
    let source = r#"
import { test } from '@playwright/test';

getAsideLocator(page, 'file-scope');
maybeHelpers[name]('unpathable');

test.describe('suite', () => {
  test.skip('skipped test', async ({ page }) => {
    await getAsideLocator(page, 'skipped-test');
  });

  test('active test', async ({ page }) => {
    test.skip(process.env.SKIP, 'conditional annotation');
    await getAsideLocator(page, 'annotated-skip');
    if (process.env.FLAG) {
      await getAsideLocator(page, 'if-branch');
    } else {
      await getAsideLocator(page, 'else-branch');
    }
    process.env.FLAG ? getAsideLocator(page, 'conditional-a') : getAsideLocator(page, 'conditional-b');
    process.env.FLAG && getAsideLocator(page, 'logical-branch');
  });
});

test.beforeEach(async ({ page }) => {
  await getAsideLocator(page, 'setup-hook');
});

test.afterEach(async ({ page }) => {
  await getAsideLocator(page, 'teardown-hook');
});
"#;

    let references = ast::with_program(
        std::path::Path::new("helper-scopes.ts"),
        source,
        |program, source| {
            extract_playwright_helper_reference_occurrences_from_program(program, source)
        },
    )
    .expect("fixture should parse");

    assert!(references.iter().any(|reference| {
        reference.value.value == "file-scope"
            && reference.scope == playwright_tests::TestOccurrenceScope::File
    }));
    assert!(references.iter().any(|reference| {
        reference.value.value == "skipped-test"
            && reference.status == playwright_tests::TestStatus::Skipped
            && reference.test_name.as_deref() == Some("skipped test")
            && reference.describe_path == vec!["suite".to_string()]
    }));
    assert!(references.iter().any(|reference| {
        reference.value.value == "annotated-skip"
            && reference.status == playwright_tests::TestStatus::Conditional
            && reference.test_name.as_deref() == Some("active test")
    }));
    assert!(references.iter().any(|reference| {
        reference.value.value == "if-branch"
            && reference.status == playwright_tests::TestStatus::Conditional
    }));
    assert!(references.iter().any(|reference| {
        reference.value.value == "conditional-a"
            && reference.status == playwright_tests::TestStatus::Conditional
    }));
    assert!(references.iter().any(|reference| {
        reference.value.value == "logical-branch"
            && reference.status == playwright_tests::TestStatus::Conditional
    }));
    assert!(references.iter().any(|reference| {
        reference.value.value == "setup-hook"
            && reference.scope == playwright_tests::TestOccurrenceScope::Hook
    }));
    assert!(references.iter().any(|reference| {
        reference.value.value == "teardown-hook"
            && reference.scope == playwright_tests::TestOccurrenceScope::TeardownHook
    }));
    assert!(references
        .iter()
        .all(|reference| reference.value.value != "unpathable"));
}

#[test]
fn helper_reference_call_rejects_empty_paths() {
    let source = "getAsideLocator(page, 'empty-path');";
    ast::with_program(
        std::path::Path::new("empty-path.ts"),
        source,
        |program, _| {
            let statement = program.body.first().expect("statement");
            let oxc_ast::ast::Statement::ExpressionStatement(statement) = statement else {
                panic!("expression statement");
            };
            let oxc_ast::ast::Expression::CallExpression(call) = &statement.expression else {
                panic!("call expression");
            };

            assert!(!is_helper_reference_call(&call.callee, &[]));
        },
    )
    .expect("fixture should parse");
}
