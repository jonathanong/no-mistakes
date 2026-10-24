use super::{docs_rule_snippets, repo_root};

#[test]
fn uuid_predicate_docs_execute_aggregate_values_and_every_transformed_operand() {
    let rule = "postgres-no-transformed-uuid-predicates";
    let page = docs_rule_snippets::load_page(&repo_root(), rule);
    let config = docs_rule_snippets::rule_config_yaml(&page.body, rule);
    for (heading, expected) in [("Valid example", 0), ("Counterexample", 7)] {
        let section = docs_rule_snippets::heading_section(&page.body, heading);
        let fences = docs_rule_snippets::fences(&section);
        let runs = docs_rule_snippets::snippet_runs(&fences);
        assert_eq!(runs.len(), 1, "{heading}: one catalog-backed SQL fixture");
        let fixture = docs_rule_snippets::materialize_snippet(&config, &runs[0].1);
        let findings = docs_rule_snippets::run_rule(fixture.path(), rule).unwrap();
        assert_eq!(findings.len(), expected, "{heading}: {findings:#?}");
        if expected == 0 {
            continue;
        }
        // Count each documented operand, including both sides of the join and FILTER.
        for (pattern, expected) in [
            ("WHERE applies cast to UUID column users.id", 2),
            (
                "WHERE applies uuid_extract_timestamp to UUID column users.id",
                1,
            ),
            ("JOIN ON applies cast to UUID column sessions.user_id", 1),
            ("JOIN ON applies cast to UUID column users.id", 1),
            (
                "HAVING applies uuid_extract_timestamp to UUID column sessions.user_id",
                1,
            ),
            ("HAVING applies cast to UUID column users.id", 1),
        ] {
            assert_eq!(
                findings.iter().filter(|hit| hit.contains(pattern)).count(),
                expected,
                "{pattern}: {findings:#?}"
            );
        }
    }
}
