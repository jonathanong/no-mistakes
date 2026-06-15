use super::common::{assert_success, fixture, run, run_in, run_json, stdout};

#[test]
fn data_pw_json_reports_source_and_test() {
    let root = fixture("data-pw");
    let value = run_json(&root, &["data-pw", "search-bar"]);
    assert_eq!(value["value"], "search-bar");

    let source: Vec<&str> = value["source"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["file"].as_str().unwrap())
        .collect();
    assert_eq!(source, vec!["app/search.tsx", "components/widget.tsx"]);

    let test: Vec<&str> = value["test"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["file"].as_str().unwrap())
        .collect();
    assert_eq!(test, vec!["e2e/search.spec.ts"]);
}

#[test]
fn data_pw_include_filters_sections() {
    let root = fixture("data-pw");
    let value = run_json(&root, &["data-pw", "search-bar", "--include", "test"]);
    assert!(value.get("source").is_none());
    assert!(value.get("test").is_some());
}

#[test]
fn data_pw_attribute_override() {
    let root = fixture("data-pw");
    let value = run_json(
        &root,
        &["data-pw", "search-bar", "--attribute", "data-testid"],
    );
    let source: Vec<&str> = value["source"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["file"].as_str().unwrap())
        .collect();
    assert_eq!(source, vec!["components/widget.tsx"]);
}

#[test]
fn data_pw_paths_format() {
    let root = fixture("data-pw");
    let root_arg = root.to_string_lossy();
    let output = run(&[
        "data-pw",
        "search-bar",
        "--root",
        root_arg.as_ref(),
        "--format",
        "paths",
    ]);
    assert_success(&output);
    let text = stdout(&output);
    assert!(text.contains("app/search.tsx"));
    assert!(text.contains("e2e/search.spec.ts"));
}

#[test]
fn data_pw_human_and_md_formats() {
    let root = fixture("data-pw");
    let human = run_in(&root, &["data-pw", "search-bar"]);
    assert_success(&human);
    assert!(stdout(&human).contains("app/search.tsx:3"));

    let md = run_in(&root, &["data-pw", "search-bar", "--format", "md"]);
    assert_success(&md);
    assert!(stdout(&md).contains("# data-pw `search-bar`"));
}

#[test]
fn data_pw_yml_and_filtered_sections() {
    let root = fixture("data-pw");
    let yml = run_in(&root, &["data-pw", "search-bar", "--format", "yml"]);
    assert_success(&yml);
    assert!(stdout(&yml).contains("value: search-bar"));

    // --include test drops the source section (covers the None-section print path).
    let human = run_in(&root, &["data-pw", "search-bar", "--include", "test"]);
    assert_success(&human);
    let md = run_in(
        &root,
        &[
            "data-pw",
            "search-bar",
            "--include",
            "test",
            "--format",
            "md",
        ],
    );
    assert_success(&md);
}

#[test]
fn data_pw_without_configured_attributes_errors() {
    // The effects fixture has no tests.playwright.selectors.testIds configured.
    let root = fixture("effects");
    let root_arg = root.to_string_lossy();
    let output = run(&["data-pw", "search-bar", "--root", root_arg.as_ref()]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no selector attributes"));
}

#[test]
fn data_pw_value_not_found_is_empty_success() {
    let root = fixture("data-pw");
    let value = run_json(&root, &["data-pw", "missing-value"]);
    assert!(value["source"].as_array().unwrap().is_empty());
    assert!(value["test"].as_array().unwrap().is_empty());
}
