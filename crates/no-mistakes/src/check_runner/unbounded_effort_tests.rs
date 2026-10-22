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
    // Max historically reports the shadowed process spelling; that unsupported
    // low shape is deliberately excluded from the common positive-proof domain.
    let source = std::fs::read_to_string(root().join("src/query.ts")).unwrap();
    let shadowed = source
        .lines()
        .position(|line| line.contains("unsupported: shadowed process"))
        .unwrap()
        + 1;
    let common = max
        .rules
        .iter()
        .filter(|row| row.line != shadowed)
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

#[test]
fn unbounded_effort_narrows_files_profiles_and_input_synthesis() {
    let root = root().parent().unwrap().join("effort-narrow");
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let report = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all_with_sql_verdicts(root, None, None, false, false, false).unwrap()
    };
    assert!(report.rules.is_empty(), "{:?}", report.rules);
    let snapshot = observer.snapshot();
    assert_eq!(
        snapshot.work.get("parse.files"),
        Some(&1),
        "{:?}",
        snapshot.work
    );
    assert_eq!(
        snapshot.work.get("postgres.sql_identifier.syntax_files"),
        Some(&1)
    );
    assert_eq!(
        snapshot.work.get("postgres.sql_identifier.syntax_inputs"),
        Some(&1)
    );
}

#[test]
fn unbounded_effort_scoped_executors_and_loaders_are_explicit_unknowns() {
    let report = run(".no-mistakes-scoped.yml", false);
    for file in ["src/scoped.ts", "src/loaders.ts"] {
        assert!(
            report.rules.iter().filter(|row| row.file == file).count() >= 2,
            "{file}: {:?}",
            report.rules
        );
    }
    assert!(report
        .rules
        .iter()
        .all(|row| row.message.contains("unanalyzable at effort low")));
    assert_eq!(
        run(".no-mistakes-inactive.yml", false).rules,
        run(".no-mistakes.yml", false).rules
    );
}
#[test]
fn unbounded_effort_project_scope_prunes_other_sources_before_parse() {
    let root = root().parent().unwrap().join("effort-narrow");
    let config = root.join(".no-mistakes-project.yml");
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let report = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all_with_sql_verdicts(root, Some(config), None, false, false, false).unwrap()
    };
    assert!(report.rules.is_empty());
    assert_eq!(observer.snapshot().work.get("parse.files"), Some(&1));
}
