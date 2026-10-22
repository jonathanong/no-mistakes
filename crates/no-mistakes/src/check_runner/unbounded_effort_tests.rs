use super::run_all_with_sql_verdicts;
use std::{path::PathBuf, sync::Arc};
const RULE: &str = "postgres-no-unbounded-sql-identifier";
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/effort-low")
}
fn run(config: &str, evidence: bool) -> super::CheckResults {
    let root = root();
    run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(config)),
        None,
        false,
        false,
        evidence,
    )
    .unwrap()
}
#[test]
fn unbounded_effort_policies_preserve_proven_violations_and_route_unknowns() {
    let report = run(".no-mistakes.yml", false);
    let advise = run(".no-mistakes-advise.yml", false);
    let ignore = run(".no-mistakes-ignore.yml", false);
    let legacy = run(".no-mistakes-legacy.yml", false);
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    assert_eq!(advise.rules, ignore.rules);
    assert_eq!(advise.rules, legacy.rules);
    assert_eq!(advise.advisories, legacy.advisories);
    assert!(
        !advise.rules.is_empty(),
        "proven violations survive ignore/advise"
    );
    assert!(
        !advise.advisories.is_empty(),
        "ordinary reports retain low advice"
    );
    assert!(ignore.advisories.is_empty());
    assert!(report.rules.iter().filter(|row| row.rule == RULE).count() > advise.rules.len());
    assert!(advise
        .advisories
        .iter()
        .all(|row| row.message.contains("one local const")));
    assert!(!report.rules.iter().any(
        |row| row.file.ends_with("excluded.ts") || row.file.ends_with("malformed-unrelated.ts")
    ));
    // Fatal parser recovery may retain no imports/calls. Match max's boundary:
    // no SQL proof or synthetic SQL finding is created for that input.
    assert!(!report
        .rules
        .iter()
        .any(|row| row.file.ends_with("malformed-configured.ts")));
    for file in ["src/query.ts", "src/selection.ts"] {
        let source = std::fs::read_to_string(root().join(file)).unwrap();
        for (index, line) in source.lines().enumerate() {
            let found = report
                .rules
                .iter()
                .any(|row| row.file == file && row.line == index + 1);
            if line.contains("// clear")
                || line.contains("// unrelated")
                || line.contains("// suppressed")
            {
                assert!(!found, "{file}:{}: {line}", index + 1);
            }
            if line.contains("// unsupported") || line.contains("// violation") {
                assert!(found, "{file}:{}: {line}", index + 1);
            }
        }
    }
    let detailed = run(".no-mistakes-advise.yml", true);
    assert_eq!(advise.rules, detailed.rules);
    assert_eq!(advise.advisories, detailed.advisories);
}
#[test]
fn unbounded_effort_max_is_unchanged_and_decidable_findings_match_low() {
    let default = run(".no-mistakes-default.yml", false);
    let max = run(".no-mistakes-max.yml", false);
    assert_eq!(default.rules, max.rules);
    assert_eq!(default.advisories, max.advisories);
    let low = run(".no-mistakes-ignore.yml", false);
    // Compare only the declared common positive-proof domain. Unsupported low
    // shapes (including shadowed process and composite relation tokens) retain
    // max's historical behavior without becoming low positive proofs.
    let source = std::fs::read_to_string(root().join("src/query.ts")).unwrap();
    let common = max
        .rules
        .iter()
        .filter(|row| {
            row.file == "src/query.ts"
                && source
                    .lines()
                    .nth(row.line - 1)
                    .is_some_and(|line| line.contains("// violation:"))
        })
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        low.rules, common,
        "decidable direct env relation findings retain exact metadata"
    );
    let mixed = run(".no-mistakes-mixed.yml", false);
    assert!(mixed
        .rules
        .iter()
        .any(|row| row.message.contains("unanalyzable at effort low")));
    for finding in max.rules {
        assert!(mixed.rules.contains(&finding));
    }
}
#[test]
fn unbounded_effort_low_does_not_prepare_deep_graph_or_provenance() {
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        let _ = run(".no-mistakes.yml", false);
    }
    let snapshot = observer.snapshot();
    assert!(
        snapshot
            .work
            .get("postgres.sql_identifier.low.calls")
            .copied()
            .unwrap_or(0)
            > 0
    );
    assert_eq!(
        snapshot
            .work
            .get("postgres.unbounded_sql_identifier.candidates")
            .copied()
            .unwrap_or(0),
        0
    );
    assert!(
        snapshot
            .timings
            .iter()
            .all(|row| !row.label.starts_with("graph.")
                && !row.label.contains("canonical_dependency_graph")),
        "{:?}",
        snapshot.timings
    );
}

#[path = "unbounded_effort_tests/scopes.rs"]
mod scopes;
