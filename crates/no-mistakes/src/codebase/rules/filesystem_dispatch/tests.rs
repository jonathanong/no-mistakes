use super::*;

mod fixture_config;
use fixture_config::prepared_rules;

/// Cover all dispatch branches via `run_filesystem_rules_with_files`.
/// Passing an empty file list means no rule actually does I/O on files —
/// they just enter their dispatch branch and return Ok(empty).
#[test]
fn dispatch_with_files_covers_all_rule_branches() {
    let (fixture, config_path) = prepared_rules(FILESYSTEM_RULE_IDS);
    let workflow = fixture.path().join(".github/workflows/ci.yml");
    let findings = run_filesystem_rules_with_files(
        fixture.path(),
        Some(&config_path),
        std::slice::from_ref(&workflow),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "clean minimal workflow fixture should produce no findings: {findings:?}"
    );
}

/// Cover all dispatch branches via `run_filesystem_rules`.
/// Each rule's own `check()` fn is called; with an empty/non-git directory
/// discover_files returns nothing, so no findings are emitted.
#[test]
fn dispatch_standalone_covers_all_rule_branches() {
    let (fixture, config_path) = prepared_rules(FILESYSTEM_RULE_IDS);
    let findings = run_filesystem_rules(fixture.path(), Some(&config_path)).unwrap();
    assert!(
        findings.is_empty(),
        "empty directory should produce no findings: {findings:?}"
    );
}

mod coverage;
mod prepared_inputs;
mod source_session;
mod standalone;
