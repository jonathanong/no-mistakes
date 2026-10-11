use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/tests-audit")
        .join(name)
}

fn artifacts() -> (TestAuditPlanArtifact, TestAuditObservationsArtifact) {
    (
        read_artifact(&fixture("plan.json")).unwrap(),
        read_artifact(&fixture("observations.json")).unwrap(),
    )
}

#[test]
fn distinguishes_positive_negative_and_unknown_execution_evidence() {
    let (plan, observations) = artifacts();
    let report = audit_test_selection(&plan, &observations).unwrap();
    assert_eq!(report.observed_test_files, 4);
    assert_eq!(report.selected_test_files, 4);
    assert_eq!(
        report.missed_observed_tests,
        vec![TestAuditExecutionEvidence {
            test_file: "tests/missed.test.mts".into(),
            matched_files: vec![],
            matched_symbols: vec![TestAuditSymbol {
                file: "src/b.mts".into(),
                symbol: "changed".into()
            }],
        }]
    );
    assert_eq!(
        report.selected_observed_tests[0].matched_files,
        ["src/a.mts"]
    );
    assert_eq!(
        report.selected_without_observed_execution[0].test_file,
        "tests/unrelated.test.mts"
    );
    assert_eq!(
        report.selected_without_observed_execution[0].reasons,
        plan.plan.selected_tests[1].reasons
    );
    assert_eq!(
        report.selected_without_observations,
        ["tests/absent.test.mts"]
    );
    assert_eq!(
        report.selected_with_incomplete_traces,
        ["tests/incomplete.test.mts"]
    );
    assert!(report
        .limitations
        .iter()
        .any(|text| text.contains("does not prove")));
    assert!(render(&report, AuditFormat::Text).contains("Missed: tests/missed.test.mts"));
    assert!(render(&report, AuditFormat::Json).contains("missed_observed_tests"));
    assert_eq!(
        run(AuditArgs {
            plan: fixture("plan.json"),
            observations: fixture("observations.json"),
            format: AuditFormat::Text
        })
        .unwrap(),
        ExitCode::SUCCESS
    );
}

#[test]
fn order_duplicates_and_impact_plan_empty_inventory_do_not_change_results() {
    let (mut plan, mut observations) = artifacts();
    let baseline =
        serde_json::to_value(audit_test_selection(&plan, &observations).unwrap()).unwrap();
    plan.plan.selected_tests.reverse();
    plan.changed_files.reverse();
    plan.changed_files.push("src/a.mts".into());
    // testsImpact intentionally has no changed-file inventory; the envelope supplies it.
    plan.plan.changed_files.clear();
    observations.tests.reverse();
    let duplicate = observations.tests[2].executed_symbols[0].clone();
    observations.tests[2].executed_symbols.push(duplicate);
    assert_eq!(
        serde_json::to_value(audit_test_selection(&plan, &observations).unwrap()).unwrap(),
        baseline
    );
}

#[test]
fn file_scope_detects_symbol_file_execution_and_no_misses_is_not_a_proof() {
    let (mut plan, mut observations) = artifacts();
    plan.changed_symbols.clear();
    let report = audit_test_selection(&plan, &observations).unwrap();
    assert_eq!(report.selected_observed_tests.len(), 2);
    assert_eq!(report.missed_observed_tests[0].matched_files, ["src/b.mts"]);
    observations
        .tests
        .retain(|test| test.test_file != "tests/missed.test.mts");
    let report = audit_test_selection(&plan, &observations).unwrap();
    assert!(report.missed_observed_tests.is_empty());
    assert!(report
        .limitations
        .iter()
        .any(|text| text.contains("does not prove")));
}

fn rejects(
    mutate: impl FnOnce(&mut TestAuditPlanArtifact, &mut TestAuditObservationsArtifact),
    message: &str,
) {
    let (mut plan, mut observations) = artifacts();
    mutate(&mut plan, &mut observations);
    let error = audit_test_selection(&plan, &observations).unwrap_err();
    assert!(error.to_string().contains(message), "{error}");
}

#[test]
fn rejects_wrong_versions_untrusted_scope_and_mismatched_provenance() {
    rejects(
        |plan, _| plan.schema_version = 2,
        "plan artifact has schema_version 2",
    );
    rejects(
        |_, run| run.schema_version = 3,
        "observations artifact has schema_version 3",
    );
    rejects(
        |_, run| run.granularity = "aggregate".into(),
        "per-test-file",
    );
    rejects(|_, run| run.suite = "targeted".into(), "suite 'full'");
    rejects(|_, run| run.complete = false, "complete true");
    rejects(
        |_, run| run.provenance.source_digest = "d".repeat(64),
        "provenance mismatch",
    );
    rejects(
        |plan, _| plan.provenance.checkout_revision = String::new(),
        "checkout_revision",
    );
    rejects(
        |plan, _| plan.provenance.source_digest = "B".repeat(64),
        "source_digest",
    );
    rejects(
        |_, run| run.provenance.scope_digest = "z".repeat(64),
        "scope_digest",
    );
    let (mut plan, mut run) = artifacts();
    plan.provenance.checkout_revision = "1".repeat(64);
    run.provenance = plan.provenance.clone();
    assert!(audit_test_selection(&plan, &run).is_ok());
}

#[test]
fn parallel_trace_classification_preserves_order_across_worker_counts() {
    let (plan, observations) = artifacts();
    let baseline =
        serde_json::to_value(audit_test_selection(&plan, &observations).unwrap()).unwrap();
    for threads in [1, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        let result = pool.install(|| audit_test_selection(&plan, &observations).unwrap());
        assert_eq!(serde_json::to_value(result).unwrap(), baseline);
    }
}

#[test]
fn rejects_missing_changes_ambiguous_paths_and_inconsistent_symbols() {
    rejects(|plan, _| plan.changed_files.clear(), "at least one");
    rejects(
        |plan, _| plan.plan.changed_files.pop().map(drop).unwrap(),
        "embedded TestPlan",
    );
    for path in [
        "", "/a", "a/", "a//b", "./a", "a/../b", "a\\b", "C:/a", "c:a", "a\nb",
    ] {
        rejects(
            |plan, _| plan.changed_files[0] = path.into(),
            "normalized root-relative",
        );
    }
    rejects(
        |plan, _| plan.plan.selected_tests[0].test_file = "../test".into(),
        "normalized root-relative",
    );
    rejects(
        |_, run| run.tests[0].executed_files.push("/bad".into()),
        "normalized root-relative",
    );
    rejects(
        |plan, _| plan.changed_symbols[0].file = "src/missing.mts".into(),
        "belong to changed_files",
    );
    rejects(
        |plan, _| plan.changed_symbols.push(plan.changed_symbols[0].clone()),
        "unique",
    );
    rejects(
        |plan, _| plan.changed_symbols[0].file = "../bad".into(),
        "normalized root-relative",
    );
    rejects(
        |plan, _| plan.changed_symbols[0].symbol = " ".into(),
        "nonempty exact symbol",
    );
    rejects(
        |_, run| run.tests[1].executed_symbols[0].symbol = "a\nb".into(),
        "nonempty exact symbol",
    );
    rejects(
        |_, run| run.tests[1].executed_files.clear(),
        "also appear in executed_files",
    );
    rejects(
        |plan, _| {
            plan.plan
                .selected_tests
                .push(plan.plan.selected_tests[0].clone())
        },
        "Duplicate selected",
    );
    rejects(
        |_, run| run.tests.push(run.tests[0].clone()),
        "Duplicate observed",
    );
    rejects(
        |_, run| run.tests[0].test_file = "./test".into(),
        "normalized root-relative",
    );
}

#[test]
fn accepts_legal_posix_colons_without_accepting_drive_prefixes() {
    let (mut plan, mut observations) = artifacts();
    plan.changed_files[0] = "routes/foo:bar.ts".into();
    plan.plan.changed_files[0] = plan.changed_files[0].clone();
    observations.tests[0].executed_files[0] = plan.changed_files[0].clone();
    plan.plan.selected_tests[0].test_file = "tests/3:route.test.mts".into();
    observations.tests[0].test_file = plan.plan.selected_tests[0].test_file.clone();
    let report = audit_test_selection(&plan, &observations).unwrap();
    assert_eq!(
        report.selected_observed_tests[0].test_file,
        "tests/3:route.test.mts"
    );
    assert_eq!(
        report.selected_observed_tests[0].matched_files,
        ["routes/foo:bar.ts"]
    );
}

#[test]
fn artifact_io_errors_are_actionable() {
    assert!(
        read_artifact::<TestAuditPlanArtifact>(&fixture("missing.json"))
            .unwrap_err()
            .to_string()
            .contains("Failed to read")
    );
    assert!(
        read_artifact::<TestAuditPlanArtifact>(&fixture("invalid.json"))
            .unwrap_err()
            .to_string()
            .contains("Invalid audit artifact")
    );
    assert!(run(AuditArgs {
        plan: fixture("invalid.json"),
        observations: fixture("observations.json"),
        format: AuditFormat::Json
    })
    .is_err());
    assert!(run(AuditArgs {
        plan: fixture("plan.json"),
        observations: fixture("missing.json"),
        format: AuditFormat::Json
    })
    .is_err());
}

#[test]
fn unrelated_unselected_traces_and_incomplete_selected_hits_remain_distinct() {
    let (plan, mut observations) = artifacts();
    observations.tests[0].trace_complete = false;
    observations.tests.push(TestAuditObservation {
        test_file: "tests/unselected-unrelated.test.mts".into(),
        executed_files: Vec::new(),
        executed_symbols: Vec::new(),
        trace_complete: true,
    });
    let report = audit_test_selection(&plan, &observations).unwrap();
    assert_eq!(report.observed_test_files, 5);
    assert_eq!(
        report.selected_observed_tests[0].test_file,
        "tests/selected.test.mts"
    );
    assert_eq!(report.selected_without_observed_execution.len(), 1);
    assert_eq!(
        report.selected_with_incomplete_traces,
        ["tests/incomplete.test.mts"]
    );
    observations
        .tests
        .retain(|test| test.test_file != "tests/missed.test.mts");
    assert!(!render(
        &audit_test_selection(&plan, &observations).unwrap(),
        AuditFormat::Text
    )
    .contains("Missed:"));
    assert!(run(AuditArgs {
        plan: fixture("mismatched-plan.json"),
        observations: fixture("observations.json"),
        format: AuditFormat::Json
    })
    .unwrap_err()
    .to_string()
    .contains("provenance mismatch"));
}

#[test]
fn artifact_contract_rejects_unknown_envelope_and_trace_fields() {
    let plan: serde_json::Value = read_artifact(&fixture("plan.json")).unwrap();
    let observations: serde_json::Value = read_artifact(&fixture("observations.json")).unwrap();
    let mut unknown_plan = plan.clone();
    unknown_plan["aggregate_coverage"] = serde_json::json!({});
    assert!(serde_json::from_value::<TestAuditPlanArtifact>(unknown_plan).is_err());
    let mut unknown_provenance = plan;
    unknown_provenance["provenance"]["unverified_alias"] = serde_json::json!("latest");
    assert!(serde_json::from_value::<TestAuditPlanArtifact>(unknown_provenance).is_err());
    let mut unknown_run = observations.clone();
    unknown_run["aggregate_coverage"] = serde_json::json!({});
    assert!(serde_json::from_value::<TestAuditObservationsArtifact>(unknown_run).is_err());
    let mut unknown_trace = observations;
    unknown_trace["tests"][0]["passing"] = serde_json::json!(true);
    assert!(serde_json::from_value::<TestAuditObservationsArtifact>(unknown_trace).is_err());
}
