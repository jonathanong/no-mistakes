use super::super::findings;
use super::*;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{ResolvedCallSite, ResolvedCallTarget};

#[test]
fn terminal_selectors_match_typed_parameter_member_calls() {
    let findings = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ function: { file: src/selectors.mts, symbol: timeoutCall } }]\n      unknownCalls: ignore\n      targets: [{ terminal: waitForTimeout }]\n",
    )
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(
        findings[0].message.contains("terminal `waitForTimeout`"),
        "{findings:#?}"
    );
    assert_eq!(findings[0].import.as_deref(), Some("page.waitForTimeout"));
}

#[test]
fn terminal_selectors_use_unresolved_member_source_spelling() {
    let root = coverage_root();
    let app = application(
        "terminal",
        "roots: [{ file: src/selectors.mts }]\ntargets: [{ terminal: waitForTimeout }]",
    );
    let opts =
        options("roots: [{ file: src/selectors.mts }]\ntargets: [{ terminal: waitForTimeout }]");
    let site = ResolvedCallSite {
        file: root.join("src/selectors.mts"),
        caller: Some("timeoutCall".to_string()),
        caller_id: None,
        line: 24,
        offset: 0,
        invocation: InvocationKind::Call,
        source_callee: "page.waitForTimeout".to_string(),
        target: ResolvedCallTarget::Unknown,
    };

    assert_eq!(
        findings::finding_for_site(&root, &app, "terminal, application #1", &opts, &site)
            .expect("typed parameter member call matches terminal selector")
            .target
            .as_deref(),
        Some("terminal `waitForTimeout`")
    );

    let bare = ResolvedCallSite {
        source_callee: "waitForTimeout".to_string(),
        ..site
    };
    assert!(
        findings::finding_for_site(&root, &app, "terminal, application #1", &opts, &bare).is_none(),
        "bare unknown identifiers must not match a terminal selector"
    );
}
