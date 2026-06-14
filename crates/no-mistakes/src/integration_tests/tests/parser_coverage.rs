use super::*;
use std::collections::BTreeSet;

fn coverage_files(prefix: &str, suffix: &str) -> Vec<String> {
    let mut files: Vec<_> = std::fs::read_dir(fixture("coverage"))
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            entry
                .file_type()
                .unwrap()
                .is_file()
                .then(|| entry.file_name().to_string_lossy().into_owned())
        })
        .filter(|name| name.starts_with(prefix) && name.ends_with(suffix))
        .collect();
    files.sort();
    files
}

#[test]
fn playwright_config_parser_covers_project_defaults() {
    let root = fixture("coverage");
    let expected_errors = BTreeSet::from([
        "playwright.empty-match-invalid.ts",
        "playwright.empty-test-match.ts",
        "playwright.invalid.ts",
        "playwright.object-testignore-invalid.ts",
    ]);
    let mut policy_names = BTreeSet::new();

    for file in coverage_files("playwright.", ".ts") {
        let path = root.join(&file);
        let source = std::fs::read_to_string(&path).unwrap();
        let result = parse_playwright_fixture(&source, &path, &root);
        if expected_errors.contains(file.as_str()) {
            assert!(result.is_err(), "expected {file} to be rejected");
            continue;
        }
        let parsed = result.unwrap_or_else(|error| panic!("{file} should parse: {error:#}"));
        for project in parsed.into_projects(&root, &file) {
            if let Some(policy_name) = project.policy_name {
                policy_names.insert(policy_name);
            }
        }
    }

    for expected in [
        "absolute",
        "imported",
        "pw-root-call-import",
        "pw-object-call-destructure-body",
        "pw-member-spread-named",
    ] {
        assert!(
            policy_names.contains(expected),
            "missing Playwright policy {expected}"
        );
    }
    assert!(!policy_names.contains("root-spread-missing"));
}

#[test]
fn vitest_config_parser_covers_root_and_nested_projects() {
    let root = fixture("coverage");
    let expected_errors = BTreeSet::from([
        "vitest.empty-array-invalid.mts",
        "vitest.invalid.mts",
        "vitest.invalid-project.mts",
        "vitest.project-exclude-invalid.mts",
    ]);
    let mut policy_names = BTreeSet::new();

    for file in coverage_files("vitest.", ".mts") {
        let path = root.join(&file);
        let source = std::fs::read_to_string(&path).unwrap();
        let result = parse_vitest_fixture(&source, &path, &root);
        if expected_errors.contains(file.as_str()) {
            assert!(result.is_err(), "expected {file} to be rejected");
            continue;
        }
        let projects = result.unwrap_or_else(|error| panic!("{file} should parse: {error:#}"));
        for project in projects {
            if let Some(policy_name) = project.policy_name {
                policy_names.insert(policy_name);
            }
        }
    }

    for expected in [
        "root-vitest",
        "nested",
        "vitest-root-call-import",
        "vitest-object-call-destructure-body",
        "vitest-member-spread-named",
        "vitest-test-sourced-reexport",
    ] {
        assert!(
            policy_names.contains(expected),
            "missing Vitest policy {expected}"
        );
    }
    assert!(!policy_names.contains("vitest-root-spread-missing"));
}
