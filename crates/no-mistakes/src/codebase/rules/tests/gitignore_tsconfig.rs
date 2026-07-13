use super::*;

fn fixture() -> tempfile::TempDir {
    let fixture = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    fixture
}

fn has_finding(
    findings: &[RuleFinding],
    rule: &str,
    import: Option<&str>,
    target: Option<&str>,
) -> bool {
    findings.iter().any(|finding| {
        finding.rule == rule
            && finding.import.as_deref() == import
            && finding.target.as_deref() == target
    })
}

#[test]
fn standalone_rules_ignore_automatic_tsconfig_but_honor_explicit_ignored_config() {
    let fixture = fixture();
    let automatic = run_check(fixture.path(), None, None).unwrap();

    assert!(has_finding(
        &automatic,
        TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
        Some("@lib/lazy"),
        None,
    ));
    assert!(has_finding(
        &automatic,
        REQUIRE_STORYBOOK_STORIES,
        None,
        Some("src/Button.tsx#Button"),
    ));
    assert!(!has_finding(
        &automatic,
        FORBIDDEN_DEPENDENCIES,
        None,
        Some("src/forbidden.ts"),
    ));

    let explicit = run_check(
        fixture.path(),
        None,
        Some(std::path::Path::new("tsconfig.json")),
    )
    .unwrap();
    assert!(has_finding(
        &explicit,
        TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
        Some("@lib/lazy"),
        Some("src/lazy.ts"),
    ));
    assert!(!has_finding(
        &explicit,
        REQUIRE_STORYBOOK_STORIES,
        None,
        Some("src/Button.tsx#Button"),
    ));
    assert!(has_finding(
        &explicit,
        FORBIDDEN_DEPENDENCIES,
        Some("import"),
        Some("src/forbidden.ts"),
    ));
}
