use super::{docs_rule_snippets, repo_root};

#[test]
fn catalog_advisory_docs_execute_both_examples_and_preserve_report_channels() {
    let rule = "postgres-materialized-view-shape";
    let page = docs_rule_snippets::load_page(&repo_root(), rule);
    let config = docs_rule_snippets::rule_config_yaml(&page.body, rule);
    assert_eq!(
        serde_yaml::from_str::<serde_yaml::Value>(&config).unwrap()["rules"][0]["severity"],
        "advisory"
    );
    for (heading, expected) in [("Valid example", 0), ("Counterexample", 3)] {
        let section = docs_rule_snippets::heading_section(&page.body, heading);
        let fences = docs_rule_snippets::fences(&section);
        let runs = docs_rule_snippets::snippet_runs(&fences);
        assert_eq!(runs.len(), 1, "{heading}: one catalog example");
        let fixture = docs_rule_snippets::materialize_snippet(&config, &runs[0].1);
        let report = docs_rule_snippets::check_report(fixture.path()).unwrap();
        assert_eq!(
            report["rules"],
            serde_json::json!([]),
            "{heading}: {report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            expected,
            "{heading}: {report}"
        );
        let hits = docs_rule_snippets::run_rule(fixture.path(), rule).unwrap();
        assert_eq!(hits.len(), expected, "{heading}: {hits:?}");
    }
}
