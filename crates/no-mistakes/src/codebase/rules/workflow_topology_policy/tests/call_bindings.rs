use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;

fn run(options: &str) -> AnalysisDiagnostics {
    let root = super::fixture("call-bindings");
    super::super::prepare::diagnostics_with_files(&root, &super::config(options), false).unwrap()
}

#[test]
fn requires_all_declared_secrets_and_required_inputs_and_advises_on_unanalyzable() {
    let output = run("requireExplicitWorkflowCallBindings: true");
    assert!(output
        .findings
        .iter()
        .all(|f| f.file.ends_with("caller-violations.yml")));
    let messages: Vec<&str> = output.findings.iter().map(|f| f.message.as_str()).collect();
    assert_eq!(messages.len(), 7, "{messages:#?}");
    assert!(messages
        .iter()
        .any(|m| m.contains("forbidden secrets: inherit")));
    assert!(messages
        .iter()
        .any(|m| m.contains("inherit_only") && m.contains("required input `target_env`")));
    assert!(messages
        .iter()
        .any(|m| m.contains("missing_optional_secret") && m.contains("declared secret `AUDIT_TOKEN`")));
    assert!(messages
        .iter()
        .any(|m| m.contains("missing_all") && m.contains("declared secret `DEPLOY_KEY`")));
    assert!(messages
        .iter()
        .any(|m| m.contains("missing_all") && m.contains("declared secret `AUDIT_TOKEN`")));
    assert!(messages
        .iter()
        .any(|m| m.contains("missing_all") && m.contains("required input `target_env`")));
    assert!(messages
        .iter()
        .any(|m| m.contains("empty_secret") && m.contains("leaves declared secret `DEPLOY_KEY` empty")));

    assert!(output
        .advisories
        .iter()
        .all(|f| f.file.ends_with("caller-unanalyzable.yml")));
    let adv_messages: Vec<&str> = output
        .advisories
        .iter()
        .map(|f| f.message.as_str())
        .collect();
    assert_eq!(adv_messages.len(), 4, "{adv_messages:#?}");
    assert!(adv_messages.iter().any(|m| m.contains("remote workflow")));
    assert!(adv_messages.iter().any(|m| m.contains("does-not-exist.yml")));
    assert!(adv_messages
        .iter()
        .any(|m| m.contains("does not declare `on.workflow_call`")));
    assert!(adv_messages
        .iter()
        .any(|m| m.contains("sets `secrets` to `${{ inputs.secrets_map }}`")));
}

#[test]
fn combining_with_forbid_secrets_inherit_does_not_duplicate_inherit_finding() {
    let single = run("requireExplicitWorkflowCallBindings: true");
    let both = run("forbidSecretsInherit: true\nrequireExplicitWorkflowCallBindings: true");
    assert_eq!(single.findings, both.findings);
    assert_eq!(single.advisories, both.advisories);
}

#[test]
fn unanalyzable_policy_routes_to_report_or_ignore() {
    let reported = run("requireExplicitWorkflowCallBindings: true\nunanalyzable: report");
    assert_eq!(reported.findings.len(), 11);
    assert!(reported.advisories.is_empty());

    let ignored = run("requireExplicitWorkflowCallBindings: true\nunanalyzable: ignore");
    assert_eq!(ignored.findings.len(), 7);
    assert!(ignored.advisories.is_empty());
}

#[test]
fn false_and_omitted_report_nothing() {
    let off = run("requireExplicitWorkflowCallBindings: false");
    assert!(off.findings.is_empty());
    assert!(off.advisories.is_empty());

    let omitted = run("{}");
    assert!(omitted.findings.is_empty());
    assert!(omitted.advisories.is_empty());
}

#[test]
fn invalid_boolean_is_rejected_at_config_validation() {
    let root = super::fixture("call-bindings");
    let cfg = super::config("requireExplicitWorkflowCallBindings: \"yes\"");
    let mut errors = Vec::new();
    super::super::collect_config_errors(&root, None, &cfg, &mut errors);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("requireExplicitWorkflowCallBindings"));
}
