use super::*;

#[test]
fn call_first_string_arguments_allow_line_suppression_at_the_dynamic_call() {
    let root = call_literal_fixture_root("suppressed-non-literal");
    let files = vec![root.join("schedules.mts"), root.join("registry.mts")];

    let findings = crate::codebase::rules::filesystem_dispatch::run_filesystem_rules_with_config(
        &root,
        &call_literal_config("ai_agents.upsertJobScheduler"),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn call_first_string_arguments_reject_incomplete_prepared_facts() {
    let root = call_literal_fixture_root("valid");
    let files = vec![root.join("schedules.mts"), root.join("registry.mts")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    // This intentionally supplies a fact entry without call-site coverage.
    // An empty call_sites vector is not evidence that the configured target is
    // absent when the shared fact plan is sparse.
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        vec![root.join("schedules.mts")],
        crate::codebase::check_facts::CheckFactPlan::default(),
    );
    let findings = check_with_files_sources_and_facts(
        &root,
        &call_literal_config("ai_agents.upsertJobScheduler"),
        &files,
        &sources,
        Some(&facts),
    )
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("prepared TypeScript facts that do not cover call-site facts"));
}
