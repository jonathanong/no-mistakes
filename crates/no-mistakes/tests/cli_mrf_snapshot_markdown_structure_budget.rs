use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "markdown-structure-budget";

fn budget_root() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/markdown-structure-budget"),
    )
}

fn fixture_root(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(&budget_root().join(name))
}

fn config(name: &str) -> PathBuf {
    rules_path(RULE, "configs", name)
}

#[test]
fn markdown_structure_budget_json_matches_snapshot() {
    let baseline = config("baseline.yml");
    assert_saved_fixture(
        "markdown_structure_budget",
        &[
            (
                "strict_thresholds",
                budget_root(),
                config("strict-thresholds.yml"),
            ),
            ("cr_line_endings", budget_root(), config("cr-lines.yml")),
            (
                "unicode_scalars",
                budget_root(),
                config("unicode-scalars.yml"),
            ),
            (
                "table_and_mermaid_counts",
                budget_root(),
                config("table-mermaid-counts.yml"),
            ),
            (
                "baseline_exact",
                fixture_root("baseline-match"),
                baseline.clone(),
            ),
            (
                "baseline_excluded",
                fixture_root("baseline-match"),
                config("baseline-excluded.yml"),
            ),
            (
                "baseline_resolved",
                fixture_root("baseline-resolved"),
                baseline.clone(),
            ),
            (
                "baseline_changed",
                fixture_root("baseline-changed"),
                baseline,
            ),
            (
                "external_stale",
                fixture_root("external-request"),
                config("external-stale.yml"),
            ),
        ],
    );
}
