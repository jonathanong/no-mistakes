use super::*;

fn fixture_root(name: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-ci-path-coverage")
            .join(name),
    )
}

#[test]
fn extract_filters_reports_invalid_workflow_yaml() {
    let root = fixture_root("bad-workflow");
    let source = std::fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();

    let (filters, findings) =
        extract_filters_from_workflow(&root, ".github/workflows/ci.yml", &source, &[]);

    assert!(filters.is_empty());
    assert_eq!(findings.len(), 1);
    assert!(findings[0]
        .message
        .contains("could not parse workflow YAML"));
}

#[test]
fn extract_filters_covers_selector_and_empty_step_branches() {
    let root = fixture_root("fixture");
    let source = r#"
name: CI
on:
  pull_request:
jobs:
  detect:
    runs-on: ubuntu-latest
    steps:
      - id: no-with
        uses: dorny/paths-filter@v3
      - id: no-filters
        uses: dorny/paths-filter@v3
        with:
          token: value
      - id: scalar
        uses: dorny/paths-filter@v3
        with:
          filters: "plain-string"
      - id: selected
        uses: dorny/paths-filter@v3
        with:
          predicate-quantifier: every
          filters: |
            backend:
              - "src/**"
"#;

    let (filters, findings) = extract_filters_from_workflow(
        &root,
        ".github/workflows/ci.yml",
        source,
        &[WorkflowSelector {
            path: ".github/workflows/ci.yml".to_string(),
            job: "detect".to_string(),
            step_id: "selected".to_string(),
        }],
    );

    assert!(findings.is_empty());
    assert_eq!(filters.len(), 1);
    assert_eq!(filters[0].name, "backend");
    assert_eq!(filters[0].quantifier, PredicateQuantifier::Every);
}

#[test]
fn extract_filters_preserves_change_type_alternatives_for_every_quantifier() {
    let root = fixture_root("fixture");
    let source = r#"
jobs:
  detect:
    runs-on: ubuntu-latest
    steps:
      - id: selected
        uses: dorny/paths-filter@v3
        with:
          predicate-quantifier: every
          filters: |
            backend:
              - added|modified:
                  - "src/**"
                  - "lib/**"
"#;

    let (filters, findings) =
        extract_filters_from_workflow(&root, ".github/workflows/ci.yml", source, &[]);

    assert!(findings.is_empty());
    assert_eq!(filters.len(), 1);
    assert!(super::super::globs::selected_by_paths_filter(
        &filters[0].compiled,
        filters[0].quantifier,
        "src/index.ts"
    ));
}

#[test]
fn extract_filters_ignores_unsupported_paths_change_type_key() {
    let root = fixture_root("fixture");
    let source = r#"
jobs:
  detect:
    runs-on: ubuntu-latest
    steps:
      - id: selected
        uses: dorny/paths-filter@v3
        with:
          filters: |
            backend:
              paths:
                - "src/**"
"#;

    let (filters, findings) =
        extract_filters_from_workflow(&root, ".github/workflows/ci.yml", source, &[]);

    assert!(findings.is_empty());
    assert_eq!(filters.len(), 1);
    assert!(!super::super::globs::selected_by_paths_filter(
        &filters[0].compiled,
        filters[0].quantifier,
        "src/index.ts"
    ));
}

#[test]
fn workflow_level_paths_must_allow_the_changed_source_path() {
    let root = fixture_root("fixture");
    let source = r#"
on:
  pull_request:
    paths:
      - "docs/**"
jobs:
  detect:
    runs-on: ubuntu-latest
    steps:
      - id: selected
        uses: dorny/paths-filter@v3
        with:
          filters: |
            backend:
              - "src/**"
"#;

    let (filters, findings) =
        extract_filters_from_workflow(&root, ".github/workflows/ci.yml", source, &[]);

    assert!(findings.is_empty());
    assert_eq!(filters.len(), 1);
    assert!(!filters[0].workflow_allows("src/index.ts"));
    assert!(filters[0].workflow_allows("docs/readme.md"));
}

#[test]
fn extract_filters_covers_non_mapping_filter_values_and_non_string_names() {
    let root = fixture_root("malformed");
    let source = r#"
jobs:
  detect:
    runs-on: ubuntu-latest
    steps:
      - id: sequence
        uses: dorny/paths-filter@v3
        with:
          filters: ".github/sequence-filters.yml"
      - id: numeric-name
        uses: dorny/paths-filter@v3
        with:
          filters: |
            1:
              - "src/**"
"#;

    let (filters, findings) =
        extract_filters_from_workflow(&root, ".github/workflows/ci.yml", source, &[]);

    assert!(filters.is_empty());
    assert!(findings.is_empty());
}

#[test]
fn extract_filters_covers_non_string_filters_and_parse_failures() {
    let root = fixture_root("fixture");
    let source = r#"
jobs:
  detect:
    runs-on: ubuntu-latest
    steps:
      - id: non-string-filters
        uses: dorny/paths-filter@v3
        with:
          filters:
            - "src/**"
      - id: missing-external
        uses: dorny/paths-filter@v3
        with:
          filters: ".github/missing.yml"
"#;

    let (filters, findings) =
        extract_filters_from_workflow(&root, ".github/workflows/ci.yml", source, &[]);

    assert!(filters.is_empty());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("could not be read"));
}

#[test]
fn ci_filters_covers_workflow_selection_and_read_errors() {
    let mut config = NoMistakesConfig::default();
    let root = fixture_root("fixture");
    let (filters, findings) = ci_filters(
        &root,
        &config,
        &[WorkflowSelector {
            path: ".github/workflows/other.yml".to_string(),
            job: String::new(),
            step_id: String::new(),
        }],
    );
    assert!(filters.is_empty());
    assert!(findings.is_empty());

    config.ci.workflow_dirs = vec![".github/bad-workflows".to_string()];
    let root = fixture_root("bad-workflow");
    let (filters, findings) = ci_filters(&root, &config, &[]);
    assert!(filters.is_empty());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("could not read workflow file"));
}

#[test]
fn extract_filters_ignores_workflows_without_jobs_or_steps() {
    let root = fixture_root("fixture");

    let (filters, findings) =
        extract_filters_from_workflow(&root, ".github/workflows/ci.yml", "name: CI", &[]);
    assert!(filters.is_empty());
    assert!(findings.is_empty());

    let (filters, findings) = extract_filters_from_workflow(
        &root,
        ".github/workflows/ci.yml",
        r#"
jobs:
  detect:
    runs-on: ubuntu-latest
"#,
        &[],
    );
    assert!(filters.is_empty());
    assert!(findings.is_empty());
}
