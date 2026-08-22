use super::*;

#[test]
fn read_request_source_without_cache_reads_and_errors() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cfg.ts");
    std::fs::write(&path, "export {}").unwrap();
    assert!(read_request_source(&path).unwrap().contains("export"));
    assert!(read_request_source(&dir.path().join("missing.ts")).is_err());
}

#[test]
fn read_request_source_with_session_surfaces_read_failures() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.ts");
    let session = crate::codebase::analysis_session::AnalysisSession::new(None);
    let error = read_request_source_with_session(&session, &missing).unwrap_err();
    assert!(error.to_string().contains("reading"));
}
