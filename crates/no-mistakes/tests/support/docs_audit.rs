use super::{read, repo_root};

#[test]
fn audit_docs_preserve_observation_and_provenance_limits() {
    let body = read(&repo_root().join("docs/cli/tests-audit.md"));
    for marker in [
        "per-test-file",
        "caller-supplied metadata",
        "does not prove completeness",
        "source_digest",
        "scope_digest",
        "trace_complete",
        "testsPlan",
        "coverage-final.json",
    ] {
        assert!(body.contains(marker), "audit docs missing {marker}");
    }
}
