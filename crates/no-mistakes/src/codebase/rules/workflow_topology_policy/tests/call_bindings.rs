use crate::codebase::ci_workflows::{
    ParsedWorkflowDocument, ParsedWorkflowSet, PreparedWorkflowTopologyInputs,
    WorkflowDocumentError, WorkflowDocumentErrorKind,
};
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use std::path::Path;
use std::sync::Arc;

fn run(options: &str) -> AnalysisDiagnostics {
    let root = super::fixture("call-bindings");
    super::super::prepare::diagnostics_with_files(&root, &super::config(options), false).unwrap()
}

fn run_in_memory(options: &str, docs: &[(&str, Result<&str, &str>)]) -> AnalysisDiagnostics {
    let parsed = ParsedWorkflowSet {
        documents: docs
            .iter()
            .map(|(path, body)| ParsedWorkflowDocument {
                path: (*path).to_string(),
                value: match body {
                    Ok(yaml) => serde_yaml::from_str(yaml).map_err(|err| WorkflowDocumentError {
                        kind: WorkflowDocumentErrorKind::Parse,
                        message: err.to_string(),
                    }),
                    Err(msg) => Err(WorkflowDocumentError {
                        kind: WorkflowDocumentErrorKind::Parse,
                        message: (*msg).to_string(),
                    }),
                },
                source: body.ok().map(Arc::from),
            })
            .collect(),
    };
    let inputs = PreparedWorkflowTopologyInputs::new(
        Path::new("."),
        &Default::default(),
        Arc::new(parsed),
        ParsedWorkflowSet {
            documents: Vec::new(),
        },
    );
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    super::super::diagnostics_prepared(
        Path::new("."),
        &super::config(options),
        None,
        &sources,
        &inputs,
        None,
        false,
    )
    .unwrap()
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
    let error = crate::codebase::rules::validate_options::validate_rule_options(
        &root,
        Path::new(".no-mistakes.yml"),
        &cfg,
        &|_| None,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("requireExplicitWorkflowCallBindings"), "{error}");
}

#[test]
fn reports_malformed_non_mapping_non_scalar_and_case_colliding_shapes() {
    let callee_bad_shapes = r#"
on:
  workflow_call:
    inputs: bad
    secrets: [TOKEN]
jobs:
  run:
    runs-on: ubuntu-latest
    steps:
      - run: echo ok
"#;
    let callee_collisions = r#"
on:
  workflow_call:
    inputs:
      Env:
        required: true
      env:
        required: true
    secrets:
      Token:
        required: true
      token:
        required: true
jobs:
  run:
    runs-on: ubuntu-latest
    steps:
      - run: echo ok
"#;
    let callee_normal = r#"
on:
  workflow_call:
    inputs:
      env:
        required: true
    secrets:
      TOKEN:
        required: true
jobs:
  run:
    runs-on: ubuntu-latest
    steps:
      - run: echo ok
"#;
    let caller = r#"
on: push
jobs:
  bad_callee:
    uses: ./.github/workflows/bad-shapes.yml
  colliding_callee:
    uses: ./.github/workflows/collisions.yml
  bad_caller_values:
    uses: ./.github/workflows/normal.yml
    with:
      env:
    secrets:
      TOKEN: [1]
  duplicate_caller_keys:
    uses: ./.github/workflows/normal.yml
    with: ${{ inputs.all }}
    secrets:
      TOKEN: a
      token: b
"#;
    let output = run_in_memory(
        "requireExplicitWorkflowCallBindings: true",
        &[
            (".github/workflows/malformed.yml", Err("invalid yaml")),
            (".github/workflows/bad-shapes.yml", Ok(callee_bad_shapes)),
            (".github/workflows/collisions.yml", Ok(callee_collisions)),
            (".github/workflows/normal.yml", Ok(callee_normal)),
            (".github/workflows/caller.yml", Ok(caller)),
        ],
    );
    assert!(output
        .findings
        .iter()
        .any(|f| f.message.contains("leaves required input `env` empty")));
    let adv: Vec<&str> = output.advisories.iter().map(|f| f.message.as_str()).collect();
    assert!(adv.iter().any(|m| m.contains("malformed.yml")), "{adv:#?}");
    assert!(
        adv.iter()
            .any(|m| m.contains("sets `on.workflow_call.secrets`")),
        "{adv:#?}"
    );
    assert!(
        adv.iter()
            .any(|m| m.contains("sets `on.workflow_call.inputs`")),
        "{adv:#?}"
    );
    assert!(
        adv.iter().any(|m| m.contains("declares case-colliding")),
        "{adv:#?}"
    );
    assert!(
        adv.iter().any(|m| m.contains("sets `secrets.TOKEN` to `a non-mapping value`")),
        "{adv:#?}"
    );
    assert!(
        adv.iter().any(|m| m.contains("sets `with` to `${{ inputs.all }}`")),
        "{adv:#?}"
    );
    assert!(
        adv.iter()
            .any(|m| m.contains("duplicate case-colliding `secrets` keys")),
        "{adv:#?}"
    );
}

#[test]
fn supports_repo_unanalyzable_custom_message_and_scalar_workflow_call() {
    let root = super::fixture("call-bindings");
    let mut cfg = super::config("requireExplicitWorkflowCallBindings: true");
    cfg.unanalyzable = Some(crate::config::v2::Unanalyzable::Report);
    let reported = super::super::prepare::diagnostics_with_files(&root, &cfg, false).unwrap();
    assert_eq!(reported.findings.len(), 11);
    assert!(reported.advisories.is_empty());

    cfg.rules[0].message = Some("Bind workflow_call inputs and secrets explicitly.".to_owned());
    let output = super::super::prepare::diagnostics_with_files(&root, &cfg, false).unwrap();
    // Three findings on missing_all (line 13) coalesce when replaced by one custom message.
    assert_eq!(output.findings.len(), 9);
    assert!(output.advisories.is_empty());
    assert!(output
        .findings
        .iter()
        .all(|f| f.message == "Bind workflow_call inputs and secrets explicitly."));

    let scalar_callee = "on: workflow_call\njobs:\n  ok:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo ok\n";
    let typed_callee = r#"
on:
  workflow_call:
    inputs:
      count:
        required: true
        type: number
      dry_run:
        required: true
        type: boolean
    secrets:
jobs:
  ok:
    runs-on: ubuntu-latest
    steps:
      - run: echo ok
"#;
    let caller = r#"
on: push
x-call: &remote_call
  uses: owner/repo/.github/workflows/ci.yml@v1
jobs:
  call_scalar:
    uses: ./.github/workflows/scalar.yml
    with:
    secrets:
  call_typed:
    uses: ./.github/workflows/typed.yml
    with:
      count: 3
      dry_run: true
  call_with_inherit:
    uses: ./.github/workflows/typed.yml
    with: inherit
  call_non_scalar_with:
    uses: ./.github/workflows/typed.yml
    with:
      count: { nested: 1 }
      dry_run: true
  call_dup_with:
    uses: ./.github/workflows/typed.yml
    with:
      count: 1
      COUNT: 2
      dry_run: true
  call_merged:
    <<: *remote_call
  call_remote_inherit:
    uses: owner/repo/.github/workflows/ci.yml@v1
    secrets: inherit
  call_empty_contract:
    uses: ./.github/workflows/empty-contract.yml
"#;
    let empty_contract = "on:\n  workflow_call:\njobs:\n  ok:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo ok\n";
    let extra = run_in_memory(
        "requireExplicitWorkflowCallBindings: true",
        &[
            (".github/workflows/scalar.yml", Ok(scalar_callee)),
            (".github/workflows/typed.yml", Ok(typed_callee)),
            (".github/workflows/empty-contract.yml", Ok(empty_contract)),
            (".github/workflows/caller.yml", Ok(caller)),
        ],
    );
    assert_eq!(extra.findings.len(), 1, "{:#?}", extra.findings);
    assert!(extra.findings[0].message.contains("forbidden secrets: inherit"));
    let extra_adv: Vec<&str> = extra.advisories.iter().map(|f| f.message.as_str()).collect();
    assert!(
        extra_adv.iter().any(|m| m.contains("sets `with` to `inherit`")),
        "{extra_adv:#?}"
    );
    assert!(
        extra_adv
            .iter()
            .any(|m| m.contains("sets `with.count` to `a non-mapping value`")),
        "{extra_adv:#?}"
    );
    assert!(
        extra_adv
            .iter()
            .any(|m| m.contains("duplicate case-colliding `with` keys")),
        "{extra_adv:#?}"
    );
    assert!(
        extra_adv.iter().any(|m| m.contains("remote workflow")),
        "{extra_adv:#?}"
    );

    let mut locations = super::super::locations::Locations::default();
    let fallback_finding = super::super::call_bindings_map::job_field_finding(
        ".github/workflows/caller-violations.yml#missing_all",
        "missing_all",
        "secrets",
        "missing secrets".to_owned(),
    );
    let violation_file = root.join(".github/workflows/caller-violations.yml");
    let violation_sources = crate::codebase::rules::source_store_for_files(&[violation_file]);
    let located = locations.locate(&root, &violation_sources, vec![fallback_finding]);
    assert_eq!(located.len(), 1);
    assert!(located[0].line > 1, "expected job fallback line, got {}", located[0].line);
}
