use super::run_all_with_sql_verdicts;
use std::path::PathBuf;
use std::sync::Arc;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/effort-low")
}

fn run(config: &str) -> crate::check_runner::CheckResults {
    let root = root();
    crate::test_support::policy_advisories(
        run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(config)),
            None,
            false,
            false,
            false,
        )
        .unwrap_or_else(|error| panic!("{config}: {error}")),
        "postgres-require-query-annotation",
    )
}

fn marked_lines(source: &str, marker: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains(marker).then_some(index + 1))
        .collect()
}

fn call_lines(rows: &[no_mistakes::codebase::rules::RuleFinding], file: &str) -> Vec<usize> {
    let mut lines = rows
        .iter()
        .filter(|row| row.rule == "postgres-require-query-annotation" && row.file == file)
        .map(|row| row.line)
        .collect::<Vec<_>>();
    lines.sort_unstable();
    lines
}

#[test]
fn low_policy_separates_missing_heads_from_unsupported_shapes_and_honors_suppressions() {
    let root = root();
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let report = run(".no-mistakes-report.yml");
    let mut expected = marked_lines(&source, "EXPECT_REPORT_");
    expected.sort_unstable();
    assert_eq!(
        call_lines(&report.rules, "src/query.mts"),
        expected,
        "accepted direct/template/left-literal/one-const forms and call-site suppressions: {:?}",
        report.rules
    );
    assert!(report.advisories.is_empty(), "{:?}", report.advisories);
    assert!(report.rules.iter().all(|row| {
        if row.line == 43 {
            row.message
                .contains("query execution must start with a /* name */ annotation")
        } else {
            row.message.contains("query(\"/* name */ \" + sql)")
                && row.message.contains("const statement")
        }
    }));
    assert!(
        call_lines(&report.rules, "src/shared.mjs").is_empty(),
        "{:?}",
        report.rules
    );
    assert!(!report
        .rules
        .iter()
        .any(|row| row.file == "src/file-disabled.mjs"));
}

#[test]
fn low_observer_is_optional_and_does_not_prepare_annotation_profiles() {
    let root = root();
    let path = root.join(".no-mistakes-report.yml");
    let without = run(".no-mistakes-report.yml");
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let with = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        crate::test_support::policy_advisories(
            run_all_with_sql_verdicts(root, Some(path), None, false, false, false).unwrap(),
            "postgres-require-query-annotation",
        )
    };
    assert_eq!(without.rules, with.rules);
    assert_eq!(without.advisories, with.advisories);
    let snapshot = observer.snapshot();
    assert!(
        snapshot.timings.iter().all(|timing| {
            ![
                "postgres.annotation.project",
                "postgres.annotation.prepare",
                "postgres.annotation.evaluate",
            ]
            .iter()
            .any(|phase| timing.label.contains(phase))
        }),
        "low must not dispatch evaluator phases: {:?}",
        snapshot.timings
    );
    let work = snapshot.work;
    assert!(
        work.get("postgres.annotation.low.calls")
            .copied()
            .unwrap_or(0)
            > 0,
        "{work:#?}"
    );
    for key in [
        "postgres.annotation.profiles",
        "postgres.annotation.roots",
        "postgres.annotation.modules",
        "postgres.annotation.parallel_passes",
        "postgres.annotation.reachability_walks",
        "postgres.annotation.target_resolutions",
    ] {
        assert_eq!(work.get(key).copied().unwrap_or(0), 0, "{key}: {work:#?}");
    }
}

#[path = "effort_low_tests/policy.rs"]
mod policy;
#[path = "effort_low_tests/tag_profiles.rs"]
mod tag_profiles;
