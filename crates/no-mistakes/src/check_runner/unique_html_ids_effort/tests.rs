use super::{run_all_with_options, AnalysisOptions};
use no_mistakes::config::v2::Effort;
use std::path::{Path, PathBuf};

const RULE: &str = "playwright-unique-html-ids";

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/playwright-unique-html-ids/fixture/effort-low"),
    )
}

fn run(root: &Path, config: Option<&str>, effort: Option<Effort>) -> super::CheckResults {
    crate::test_support::without_hygiene_advisories(
        run_all_with_options(
            root.to_path_buf(),
            config.map(|name| root.join(name)),
            None,
            false,
            false,
            AnalysisOptions {
                effort,
                include_sql_verdicts: false,
            },
        )
        .unwrap(),
    )
}

fn targets(results: &super::CheckResults) -> Vec<&str> {
    let mut targets = results
        .rules
        .iter()
        .filter(|row| row.rule == RULE && row.message.contains("duplicated"))
        .map(|row| row.target.as_deref().unwrap())
        .collect::<Vec<_>>();
    targets.sort_unstable();
    targets
}

#[test]
fn default_remains_max_for_existing_html_id_findings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-html-ids/unique-html-ids/fixture");
    let default = run(&root, None, None);
    let max = run(&root, None, Some(Effort::Max));
    assert_eq!(
        serde_json::to_value(&default.rules).unwrap(),
        serde_json::to_value(&max.rules).unwrap()
    );
    assert_eq!(default.advisories, max.advisories);
    assert!(default.rules.iter().any(|row| row.rule == RULE));
}

#[test]
fn max_recovers_duplicate_ids_and_matches_the_default() {
    let root = fixture();
    let default = run(&root, Some(".no-mistakes-default-max.yml"), None);
    let max = run(&root, Some(".no-mistakes-explicit-max.yml"), None);
    assert_eq!(
        serde_json::to_value(&default.rules).unwrap(),
        serde_json::to_value(&max.rules).unwrap()
    );
    assert_eq!(default.advisories, max.advisories);
    assert!(max.advisories.is_empty());
    assert_eq!(
        targets(&max),
        [
            "id=literal-duplicate",
            "id=literal-duplicate",
            "id=recovered-duplicate",
            "id=recovered-duplicate",
        ]
    );
    let override_max = run(&root, Some(".no-mistakes-report.yml"), Some(Effort::Max));
    assert_eq!(
        serde_json::to_value(&override_max.rules).unwrap(),
        serde_json::to_value(&max.rules).unwrap()
    );
}

#[test]
fn low_routes_only_indirect_html_ids_and_max_reports_at_least_as_many() {
    let root = fixture();
    let max = run(&root, Some(".no-mistakes-explicit-max.yml"), None);
    for (policy, finding_count, advisory_count) in
        [("report", 4, 0), ("advise", 2, 2), ("ignore", 2, 0)]
    {
        let low = run(&root, Some(&format!(".no-mistakes-{policy}.yml")), None);
        assert!(low.analysis_fallbacks.is_empty());
        assert_eq!(low.analysis_warning_count, 0);
        assert_eq!(low.rules.len(), finding_count, "{policy}: {:?}", low.rules);
        assert_eq!(low.advisories.len(), advisory_count);
        assert!(max.rules.len() >= low.rules.len());
        assert_eq!(
            targets(&low),
            ["id=literal-duplicate", "id=literal-duplicate"]
        );
        let shapes = low
            .rules
            .iter()
            .chain(&low.advisories)
            .filter(|row| row.message.contains("unanalyzable at effort low"))
            .collect::<Vec<_>>();
        assert_eq!(shapes.len(), if policy == "ignore" { 0 } else { 2 });
        for row in shapes {
            assert_eq!(row.rule, RULE);
            assert!(row.file.ends_with("derived-a.tsx") || row.file.ends_with("derived-b.tsx"));
            assert!(row.message.contains("literal HTML id"), "{}", row.message);
            assert!(row.target.as_deref().unwrap().starts_with("id="));
        }
    }
}

#[test]
fn another_max_rule_keeps_recovery_without_widening_the_low_html_rule() {
    let root = fixture();
    let low = run(&root, Some(".no-mistakes-report.yml"), None);
    let mixed = run(&root, Some(".no-mistakes-mixed.yml"), None);
    let html = mixed
        .rules
        .iter()
        .filter(|row| row.rule == RULE)
        .collect::<Vec<_>>();
    assert_eq!(html.len(), low.rules.len());
    assert_eq!(targets(&mixed), targets(&low));
    // Shared max recovery may enrich the expression's display value; low
    // must still diagnose the same source sites instead of using it as proof.
    let shape_sites = |rows: &[no_mistakes::codebase::rules::RuleFinding]| {
        rows.iter()
            .filter(|row| row.rule == RULE && row.message.contains("unanalyzable at effort low"))
            .map(|row| {
                assert!(row.message.contains("literal HTML id"));
                (row.file.clone(), row.line)
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(shape_sites(&mixed.rules), shape_sites(&low.rules));
    assert_eq!(
        mixed
            .rules
            .iter()
            .filter(|row| row.rule == "playwright-unique-test-ids"
                && row.target.as_deref() == Some("data-testid=recovered-test-control"))
            .count(),
        2
    );
}
