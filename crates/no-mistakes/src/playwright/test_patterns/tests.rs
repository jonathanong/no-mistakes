use crate::integration_tests::project_config::load_projects;
use crate::integration_tests::types::Framework;
use crate::playwright::analysis::discover::discover_test_files_from_visible;
use crate::playwright::fsutil::VisiblePathSnapshot;
use crate::playwright::playwright_config::load_many;
use crate::playwright::test_support::fixture_path;
use std::path::Path;

fn selected(name: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let root = fixture_path(&["playwright-configs", name]);
    let config_path = root.join("playwright.config.ts");
    let snapshot = VisiblePathSnapshot::new(&root);
    let settings = crate::playwright::config::load_settings_from_visible(
        &root,
        None,
        std::slice::from_ref(&config_path),
        None,
        None,
        &snapshot,
    )
    .unwrap();
    let config = load_many(&root, &[config_path], None).unwrap();
    let selected = discover_test_files_from_visible(&root, &settings, &config, &snapshot)
        .unwrap()
        .iter()
        .map(|file| relative(&root, &file.path))
        .collect::<Vec<_>>();
    let warnings = config
        .projects
        .iter()
        .flat_map(|project| {
            project
                .pattern_options
                .test_match
                .warnings
                .iter()
                .chain(project.pattern_options.test_ignore.warnings.iter())
        })
        .map(|warning| warning.render(&root))
        .collect();
    let catalog = load_projects(&root, Framework::Playwright, None).unwrap();
    let projects = catalog
        .iter()
        .flat_map(|project| {
            let include =
                crate::integration_tests::project_config::build_globset(&project.include).unwrap();
            let exclude =
                crate::integration_tests::project_config::build_globset(&project.exclude).unwrap();
            snapshot
                .paths_for(&root)
                .iter()
                .filter(|path| {
                    let path = relative(&root, path);
                    include.is_match(&path) && !exclude.is_match(&path)
                })
                .map(|path| relative(&root, path))
                .collect::<Vec<_>>()
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    (selected, projects, warnings)
}

fn relative(root: &Path, path: &Path) -> String {
    crate::codebase::ts_source::relative_slash_path(root, path)
}

#[test]
fn playwright_pattern_scanner_and_catalog_select_the_same_literal_matches() {
    for (fixture, expected) in [
        (
            "regexp-match",
            vec![
                "sibling/healthy.spec.ts",
                "tests/example.e2e.ts",
                "tests/glob-skip/skip.e2e.ts",
                "tests/regex-skip.e2e.ts",
            ],
        ),
        (
            "mixed-regexp-match",
            vec![
                "sibling/healthy.spec.ts",
                "tests/example.e2e.ts",
                "tests/example.extra.ts",
                "tests/upper.E2E.ts",
            ],
        ),
        (
            "string-match",
            vec![
                "sibling/healthy.spec.ts",
                "tests/example.e2e.ts",
                "tests/glob-skip/skip.e2e.ts",
                "tests/regex-skip.e2e.ts",
            ],
        ),
        (
            "root-pattern-match",
            vec![
                "sibling/healthy.spec.ts",
                "tests/example.e2e.ts",
                "tests/example.extra.ts",
                "tests/upper.E2E.ts",
            ],
        ),
        ("empty-match", vec!["sibling/healthy.spec.ts"]),
        // A relative-looking anchored regex must not match the testDir-relative path.
        ("absolute-regexp-match", vec!["sibling/healthy.spec.ts"]),
        (
            "stateful-regexp-match",
            vec![
                "sibling/healthy.spec.ts",
                "tests/example.e2e.ts",
                "tests/é.e2e.ts",
            ],
        ),
        (
            "flags-regexp-match",
            vec![
                "sibling/healthy.spec.ts",
                "tests/example.e2e.ts",
                "tests/upper.E2E.ts",
                "tests/é.e2e.ts",
            ],
        ),
    ] {
        let (scanner, catalog, warnings) = selected(fixture);
        assert_eq!(scanner, expected, "scanner {fixture}");
        assert_eq!(catalog, expected, "catalog {fixture}");
        assert!(warnings.is_empty(), "{fixture}: {warnings:?}");
    }
}

#[test]
fn playwright_pattern_unknowns_preserve_project_scope_and_healthy_siblings() {
    for fixture in [
        "unknown-match",
        "unsupported-regexp-match",
        "invalid-glob-match",
        "divergent-regexp-match",
        "unicode-mode-regexp-match",
        "multiline-unknown-match",
    ] {
        let (scanner, catalog, warnings) = selected(fixture);
        assert_eq!(scanner, catalog, "{fixture}");
        assert!(scanner.contains(&"sibling/healthy.spec.ts".to_string()));
        assert!(scanner.contains(&"tests/plain.spec.ts".to_string()));
        assert!(scanner
            .iter()
            .all(|path| path.starts_with("tests/") || path == "sibling/healthy.spec.ts"));
        assert_eq!(warnings.len(), 1, "{fixture}: {warnings:?}");
        assert!(
            warnings[0].starts_with("playwright.config.ts:5:"),
            "{fixture}: {warnings:?}"
        );
        assert!(warnings[0].contains("testMatch"));
    }
    let (scanner, catalog, warnings) = selected("unknown-ignore");
    assert_eq!(scanner, catalog);
    assert!(scanner.contains(&"tests/regex-skip.e2e.ts".to_string()));
    assert!(warnings[0].contains("testIgnore"));
}

#[test]
fn playwright_pattern_empty_matcher_disables_filename_fallback() {
    let root = fixture_path(&["playwright-configs", "empty-match"]);
    let config = crate::config::v2::NoMistakesConfig::default();
    let discovered = crate::codebase::test_discovery::discover_tests(
        &root,
        &config,
        crate::codebase::test_discovery::TestRunner::Playwright,
    )
    .unwrap();
    assert_eq!(
        discovered
            .tests
            .iter()
            .map(|path| relative(&root, path))
            .collect::<Vec<_>>(),
        vec!["sibling/healthy.spec.ts"]
    );
    assert!(!discovered.used_fallback);
}
