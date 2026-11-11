use super::*;

#[test]
fn timing_metadata_omits_unknown_check_runner_labels() {
    assert_eq!(
        timing_metadata("parse_extract", true),
        Some((
            "facts.prepare",
            no_mistakes::diagnostics::TimingKind::Serial
        ))
    );
    assert_eq!(
        timing_metadata("parse_extract", false),
        Some(("parse", no_mistakes::diagnostics::TimingKind::Serial))
    );
    assert_eq!(
        timing_metadata("queues", false),
        Some((
            "analysis.queues",
            no_mistakes::diagnostics::TimingKind::Parallel,
        ))
    );
    assert_eq!(timing_metadata("not-a-check-timing", false), None);
}

#[test]
fn cli_check_surfaces_precise_invalid_rule_option_type() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/invalid-rule-options");
    let error = run(CheckArgs {
        root,
        config: None,
        tsconfig: None,
        effort: None,
        format: Format::Human,
        json: false,
        include_suppressed: false,
        include_runner_config_deadlines: false,
        include_sql_verdicts: false,
        timings: false,
        verbose_timings: false,
    })
    .expect_err("invalid configured rule options must fail the CLI check");
    let message = format!("{error:#}");

    assert!(message.contains(
        "invalid options for rule `postgres-no-add-column` application `strict SQL migrations`"
    ));
    assert!(message.contains("options.sqlInclude"));
    assert!(message.contains("boolean"));
    assert!(message.contains("expected a sequence"));
}

#[test]
fn effort_partner_warnings_are_non_failing_without_changing_old_warnings() {
    let mut results = check_runner::empty_results([None]);
    results
        .warnings
        .push("missing partner; falling back to max".into());
    results.analysis_warning_count = 1;
    assert!(!has_failures(&results));
    results.warnings.push("existing domain warning".into());
    assert!(has_failures(&results));
}

#[test]
fn incomplete_fact_extraction_fails_without_ordinary_findings() {
    let mut results = check_runner::empty_results([None]);
    assert!(!has_failures(&results));
    results
        .incomplete
        .push(no_mistakes::codebase::check_outcome::CheckFailure::new(
            "fact-extraction",
            "fact extraction panicked for first.ts: injected fact extraction failure",
        ));
    assert!(has_failures(&results));
}
