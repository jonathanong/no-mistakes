use crate::playwright::ast;
use crate::playwright::playwright_urls::extract_playwright_urls;

fn selectors(source: &str) -> Vec<String> {
    let regexes = crate::playwright::selectors::compile_selector_regexes(
        &["data-testid".to_string()],
        &Default::default(),
    );
    ast::with_program(
        std::path::Path::new("aliases.spec.ts"),
        source,
        |program, source| {
            crate::playwright::selectors::extract_playwright_selector_occurrences_from_program(
                program,
                source,
                &regexes,
                &["data-testid".to_string()],
                &[],
            )
            .into_iter()
            .map(|occurrence| occurrence.value.selector)
            .collect()
        },
    )
    .expect("fixture should parse")
}

#[test]
fn dotted_calls_are_seen_and_static_aliases_are_not() {
    let source = crate::playwright::test_support::fixture_source(&[
        "playwright",
        "recall-gaps",
        "aliases.spec.ts",
    ]);
    let selectors = selectors(&source);
    let urls = extract_playwright_urls(&source);

    assert!(selectors
        .iter()
        .any(|selector| selector == "getByTestId(save)"));
    assert!(urls.iter().any(|url| url == "/dashboard"));

    // #2521: computed members, a const test id, and a const method alias are silent.
    // Const URL interpolation is independent of the remaining alias gaps.
    assert!(!selectors
        .iter()
        .any(|selector| selector.contains("archive")));
    assert!(!selectors.iter().any(|selector| selector.contains("rename")));
    assert!(!selectors
        .iter()
        .any(|selector| selector.contains("publish")));
    assert!(!urls.iter().any(|url| url == "/settings"));
    assert!(!urls.iter().any(|url| url == "/billing"));
    assert!(urls.iter().any(|url| url == "/blog/post"));
}

#[test]
fn const_url_templates_respect_bindings_and_keep_dynamic_segments() {
    let source = crate::playwright::test_support::fixture_source(&[
        "playwright",
        "recall-gaps",
        "const-urls.spec.ts",
    ]);
    let urls = extract_playwright_urls(&source);
    for expected in [
        "/blog/post",
        "/archive/posts/post",
        "/escaped/ab",
        "/nested/inner",
        "/after/post",
        "/outer/outer",
        "/parameter/${slug}",
        "/let/${mutable}",
        "/var/${legacy}",
        "/number/${number}",
        "/dynamic/${dynamic}",
        "/import/${imported}",
        "/before/${slug}",
        "/written/${written}",
        "/object/${object}",
        "/alias/${alias}",
        "/expression/${slug.toUpperCase()}",
        "/click/${slug}",
    ] {
        assert!(
            urls.iter().any(|url| url == expected),
            "missing {expected}: {urls:?}"
        );
    }
    assert_eq!(urls.len(), 18);
}
