use super::*;

fn task(warning: Option<String>) -> CheckTask<Vec<&'static str>> {
    CheckTask {
        findings: vec!["retained finding"],
        react_suppression_targets: Vec::new(),
        pre_suppressed: Vec::new(),
        suppression_sources: Vec::new(),
        advisories: Vec::new(),
        warning,
        duration: Duration::from_millis(7),
    }
}

#[test]
fn successful_tasks_keep_findings_and_warning_failure_classification() {
    for warning in [None, Some("one input could not be read".to_string())] {
        let session = AnalysisSession::disabled();
        let retained = retain(&session, "filesystem", Ok(task(warning.clone())));
        assert_eq!(retained.findings, ["retained finding"]);
        assert_eq!(retained.duration, Duration::from_millis(7));
        assert_eq!(retained.warning, warning);
        let expected = warning
            .map(|message| CheckFailure::new("filesystem", message))
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(session.check_failures(), expected);
    }
}

#[test]
fn failed_task_retains_error_chain_without_inventing_findings_or_timing() {
    let session = AnalysisSession::disabled();
    let error = anyhow::anyhow!("source unavailable").context("building graph");
    let retained = retain::<Vec<&str>>(&session, "rules", Err(error));
    assert!(retained.findings.is_empty());
    assert!(retained.warning.is_none());
    assert!(retained.advisories.is_empty());
    assert_eq!(retained.duration, Duration::ZERO);
    assert_eq!(
        session.check_failures(),
        [CheckFailure::new(
            "rules",
            "building graph: source unavailable".to_string(),
        )]
    );
}
