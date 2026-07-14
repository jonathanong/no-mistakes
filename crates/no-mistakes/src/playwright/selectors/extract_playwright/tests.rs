use super::*;
use crate::playwright::ast;

#[test]
fn selector_occurrences_preserve_file_test_and_hook_scope() {
    let source = crate::playwright::test_support::fixture_source(&[
        "ast-snippets",
        "selectors",
        "extract-playwright",
        "scope.ts",
    ]);
    let regexes = crate::playwright::selectors::compile_selector_regexes(
        &["data-testid".to_string()],
        &Default::default(),
    );

    let occurrences = ast::with_program(
        std::path::Path::new("scope.ts"),
        &source,
        |program, source| {
            extract_playwright_selector_occurrences_from_program(
                program,
                source,
                &regexes,
                &["data-testid".to_string()],
                &[],
            )
        },
    )
    .expect("fixture should parse");

    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.selector == "getByTestId(file-scope)"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::File
            && occurrence.test_name.is_none()
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.selector == "getByTestId(setup)"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Hook
            && occurrence.test_name.is_none()
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.selector == "getByTestId(inside-test)"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Test
            && occurrence.test_name.as_deref() == Some("active")
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.selector == "getByTestId(teardown)"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::TeardownHook
            && occurrence.test_name.is_none()
    }));
    assert!(occurrences.iter().any(|occurrence| {
        occurrence.value.selector == "getByTestId(dynamic-test)"
            && occurrence.scope == playwright_tests::TestOccurrenceScope::Test
            && occurrence.test_name.is_none()
    }));
}

#[test]
fn selector_wrappers_match_raw_static_import_sources_without_resolution() {
    let path = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/selector-wrappers/tests/page.spec.ts"),
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let regexes = crate::playwright::selectors::compile_selector_regexes(
        &["data-pw".to_string()],
        &Default::default(),
    );
    let wrapper = |module: &str, export: &str, test_id_argument| {
        crate::config::v2::schema::PlaywrightSelectorWrapper {
            module: module.to_string(),
            export: export.to_string(),
            test_id_argument,
        }
    };
    let wrappers = vec![
        wrapper("./helpers", "getAsideLocator", 1),
        wrapper("@fixture/default-locator", "default", 0),
        wrapper("@fixture/namespace-locators", "byTestId", 1),
        wrapper("@fixture/namespace-locators", "getByTestId", 1),
        wrapper("#selector-helpers", "getAsideLocator", 1),
        wrapper("@fixture/workspace-locators/aside", "workspaceLocator", 1),
    ];

    let selectors = ast::with_program(&path, &source, |program, source| {
        extract_playwright_selector_occurrences_from_program(
            program,
            source,
            &regexes,
            &["data-pw".to_string()],
            &wrappers,
        )
        .into_iter()
        .map(|occurrence| occurrence.value.selector)
        .collect::<Vec<_>>()
    })
    .unwrap();

    for expected in [
        "aside-button",
        "default-button",
        "namespace-button",
        "namespace-native-name",
        "package-import-button",
        "workspace-export-button",
    ] {
        assert!(
            selectors.iter().any(|selector| selector.contains(expected)),
            "missing {expected}: {selectors:?}"
        );
    }
    for unexpected in [
        "ambiguous-button",
        "shadowed-button",
        "recognized-missing-button",
    ] {
        assert!(
            selectors
                .iter()
                .all(|selector| !selector.contains(unexpected)),
            "unexpected {unexpected}: {selectors:?}"
        );
    }
}
