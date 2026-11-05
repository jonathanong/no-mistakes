use super::*;
use crate::codebase::analysis_session::AnalysisSession;

#[test]
fn prerequisite_failure_preserves_legacy_error_and_partial_consumer() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/partial-failures/fixture/src/broken.ts");
    assert!(file.exists());
    let invoke = |session: Option<&AnalysisSession>| accept(session, &file, "invalid source");
    let session = AnalysisSession::disabled();
    for error in [
        invoke(None).unwrap_err(),
        invoke(Some(&session)).unwrap_err(),
    ] {
        assert!(error
            .to_string()
            .contains("failed to parse integration file"));
        assert!(error.to_string().contains("invalid source"));
        assert!(error.to_string().contains(&file.display().to_string()));
    }
    assert!(session.check_failures().is_empty());
    session.enable_partial_checks();
    assert!(!invoke(Some(&session)).unwrap());
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].consumer, "integration-test-no-mocks");
    assert_eq!(failures[0].file.as_ref(), Some(&file));
    assert!(failures[0].message.contains("invalid source"));
}
