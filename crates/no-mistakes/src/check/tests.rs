use super::*;

#[test]
fn timing_metadata_omits_unknown_check_runner_labels() {
    assert_eq!(
        timing_metadata("queues"),
        Some((
            "analysis.queues",
            no_mistakes::diagnostics::TimingKind::Parallel,
        ))
    );
    assert_eq!(timing_metadata("not-a-check-timing"), None);
}

#[test]
fn cli_check_surfaces_precise_invalid_rule_option_type() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/invalid-rule-options");
    let error = run(CheckArgs {
        root,
        config: None,
        tsconfig: None,
        format: Format::Human,
        json: false,
        include_suppressed: false,
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
