use super::run_all_with_sql_verdicts;
use no_mistakes::codebase::rules::RuleFinding;
use std::path::PathBuf;

const RULE: &str = "postgres-no-mutable-sql-registry";

fn run(mode: &str) -> super::CheckResults {
    run_with_evidence(mode, false)
}

fn run_with_evidence(mode: &str, evidence: bool) -> super::CheckResults {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-mutable-sql-registry/fixture/effort-low");
    crate::test_support::without_hygiene_advisories(
        run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(format!(".no-mistakes-{mode}.yml"))),
            None,
            false,
            false,
            evidence,
        )
        .unwrap(),
    )
}

fn lines(rows: &[RuleFinding]) -> Vec<usize> {
    let mut lines = rows
        .iter()
        .filter(|row| row.rule == RULE)
        .map(|row| row.line)
        .collect::<Vec<_>>();
    lines.sort_unstable();
    lines
}

#[test]
fn literal_sql_is_decidable_and_cross_file_registry_is_explicitly_unsupported() {
    let report = run("report");
    assert_eq!(lines(&report.rules), vec![10, 12], "{:?}", report.rules);
    assert!(report.advisories.is_empty());
    assert!(report
        .rules
        .iter()
        .filter(|row| row.rule == RULE)
        .all(|row| row.message.contains("effort low")));
    let advise = run("advise");
    assert!(lines(&advise.rules).is_empty());
    assert_eq!(lines(&advise.advisories), lines(&report.rules));
    let ignore = run("ignore");
    assert!(lines(&ignore.rules).is_empty());
    assert!(lines(&ignore.advisories).is_empty());
}

#[test]
fn default_and_explicit_max_keep_the_cross_file_write_finding() {
    let default = run("default-max");
    let explicit = run("explicit-max");
    assert_eq!(default.rules, explicit.rules);
    assert_eq!(default.advisories, explicit.advisories);
    assert_eq!(lines(&default.rules), vec![10], "{:?}", default.rules);
}

#[test]
fn low_does_not_prepare_mutation_or_provenance_analysis() {
    use std::sync::Arc;
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let report = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run("report")
    };
    assert_eq!(lines(&report.rules), vec![10, 12]);
    let snapshot = observer.snapshot();
    for key in [
        "postgres.mutable_sql_registry.producers",
        "postgres.mutable_sql_registry.candidates",
        "graph.nodes",
        "graph.edges",
        "postgres.embedded.helper_resolutions",
        "postgres.annotation.profiles",
        "postgres.annotation.calls",
    ] {
        assert_eq!(
            snapshot.work.get(key).copied().unwrap_or(0),
            0,
            "{key}: {:?}",
            snapshot.work
        );
    }
    assert!(
        snapshot
            .timings
            .iter()
            .all(|timing| !timing.label.starts_with("graph.")
                && timing.label != "rules.canonical_dependency_graph"
                && !timing.label.starts_with("postgres.annotation.")),
        "low-only invocation must not prepare a call graph: {:?}",
        snapshot.timings
    );
}

#[test]
fn mixed_effort_keeps_both_the_max_violation_and_low_unsupported_diagnostic() {
    let maximum = run("explicit-max");
    let mixed = run("mixed");
    for finding in maximum.rules.iter().filter(|row| row.rule == RULE) {
        assert!(
            mixed.rules.contains(finding),
            "lost max finding: {finding:?}"
        );
    }
    assert!(mixed
        .rules
        .iter()
        .any(|row| row.rule == RULE && row.message.contains("effort low")));
    assert!(!mixed
        .rules
        .iter()
        .any(|row| row.file.ends_with("file-disabled.ts")));
}

#[test]
fn literal_and_bound_value_sql_have_identical_decidable_findings() {
    let low = run("decidable-low");
    let max = run("decidable-max");
    assert_eq!(low.rules, max.rules);
    assert_eq!(low.advisories, max.advisories);
    assert!(low.rules.is_empty(), "{:?}", low.rules);
    assert!(low.advisories.is_empty());
}

#[test]
fn requesting_sql_evidence_preserves_findings_and_advisories() {
    for mode in ["report", "advise", "ignore"] {
        let plain = run(mode);
        let evidence = run_with_evidence(mode, true);
        assert_eq!(plain.rules, evidence.rules, "{mode}");
        assert_eq!(plain.advisories, evidence.advisories, "{mode}");
    }
}

#[test]
fn inactive_application_does_not_parse_its_unselected_options() {
    let selected = run("decidable-low");
    let with_inactive = run("inactive-target");
    assert_eq!(selected.rules, with_inactive.rules);
    assert_eq!(selected.advisories, with_inactive.advisories);
}

#[test]
fn legacy_unanalyzable_alias_routes_the_same_advisories() {
    let canonical = run("advise");
    let alias = run("alias");
    assert_eq!(canonical.rules, alias.rules);
    assert_eq!(canonical.advisories, alias.advisories);
}
