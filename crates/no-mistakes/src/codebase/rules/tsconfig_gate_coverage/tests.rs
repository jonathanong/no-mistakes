use super::application::resolve_gate_projects_against_tracked;
use super::workflow::{
    ci_typechecked_projects, default_working_directory, effective_working_directory,
};
use super::*;
use crate::codebase::ci_workflows::{
    ParsedWorkflowDocument, ParsedWorkflowSet, WorkflowDocumentError, WorkflowDocumentErrorKind,
};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use serde_yaml::Value;
use std::collections::BTreeMap;

mod fixture_policy;
mod no_check;
mod workflow;
mod workflow_contracts;

fn project_inputs(tracked: &BTreeSet<String>) -> ProjectSourceInputs {
    tracked
        .iter()
        .map(|project| (project.clone(), BTreeSet::from([project.clone()])))
        .collect()
}

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/tsconfig-gate-coverage")
            .join(name),
    )
}

fn config(root: &Path) -> NoMistakesConfig {
    crate::config::v2::load_v2_config(root, Some(&root.join(".no-mistakes.yml"))).unwrap()
}

fn findings(root: &Path, config: &NoMistakesConfig) -> Vec<RuleFinding> {
    check(root, config).unwrap()
}

fn add_static_runners(workflow: &mut Value) {
    workflow.as_mapping_mut().expect("workflow mapping").insert(
        Value::String("on".to_string()),
        Value::String("push".to_string()),
    );
    let jobs = workflow
        .get_mut("jobs")
        .and_then(Value::as_mapping_mut)
        .expect("workflow jobs mapping");
    for job in jobs.values_mut() {
        job.as_mapping_mut().expect("workflow job mapping").insert(
            Value::String("runs-on".to_string()),
            Value::String("ubuntu-latest".to_string()),
        );
    }
}

fn check(root: &Path, config: &NoMistakesConfig) -> anyhow::Result<Vec<RuleFinding>> {
    let paths = crate::codebase::ts_source::discover_files(root, &[]);
    let workflows = ParsedWorkflowSet::load(root, &config.ci);
    let sources = super::super::source_store_for_files(&paths);
    let workspace = crate::codebase::workspaces::load_indexed_from_source_store(root, &sources)?;
    let project_source_inputs = prepare_project_source_inputs(root, &paths, &sources, &workspace);
    check_with_prepared(
        root,
        config,
        PreparedInputs {
            tracked_paths: &paths,
            workflows: &workflows,
            project_source_inputs: &project_source_inputs,
            sources: &sources,
            config_path: Some(&root.join(".no-mistakes.yml")),
        },
    )
}

#[test]
fn unread_tsconfigs_defer_to_tsc_without_rereading_prepared_sources() {
    let source = fixture_root("no-check");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let config = config(&root);
    let paths = crate::codebase::ts_source::discover_files(&root, &[]);
    let workflows = ParsedWorkflowSet::load(&root, &config.ci);
    let sources = super::super::source_store_for_files(&paths);
    let workspace =
        crate::codebase::workspaces::load_indexed_from_source_store(&root, &sources).unwrap();
    let project_source_inputs = prepare_project_source_inputs(&root, &paths, &sources, &workspace);
    std::fs::remove_file(root.join("override/tsconfig.json")).unwrap();

    let prepared = PreparedInputs {
        tracked_paths: &paths,
        workflows: &workflows,
        project_source_inputs: &project_source_inputs,
        sources: &sources,
        config_path: Some(&root.join(".no-mistakes.yml")),
    };
    let report = check_with_prepared(&root, &config, prepared).unwrap();
    assert!(report
        .iter()
        .all(|finding| finding.file != "override/tsconfig.json"));
    let reads_after_first_check = sources.physical_read_count();

    let report = check_with_prepared(
        &root,
        &config,
        PreparedInputs {
            tracked_paths: &paths,
            workflows: &workflows,
            project_source_inputs: &project_source_inputs,
            sources: &sources,
            config_path: Some(&root.join(".no-mistakes.yml")),
        },
    )
    .unwrap();
    assert!(report
        .iter()
        .all(|finding| finding.file != "override/tsconfig.json"));
    assert_eq!(sources.physical_read_count(), reads_after_first_check);
}

#[test]
fn directory_project_arguments_resolve_only_against_tracked_tsconfigs() {
    let tracked = BTreeSet::from([
        "app/tsconfig.json".to_string(),
        "app.json/tsconfig.json".to_string(),
        "tsconfig.json".to_string(),
    ]);
    let gate_projects = BTreeSet::from([
        "app".to_string(),
        "app.json".to_string(),
        ".".to_string(),
        "missing".to_string(),
    ]);

    assert_eq!(
        resolve_gate_projects_against_tracked(&gate_projects, &tracked),
        BTreeSet::from([
            "app/tsconfig.json".to_string(),
            "app.json/tsconfig.json".to_string(),
            "missing".to_string(),
            "tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn reports_each_missing_gate_on_the_uncovered_project() {
    let root = fixture_root("missing-ci");
    let report = findings(&root, &config(&root));
    assert_eq!(report.len(), 1, "{report:#?}");
    assert_eq!(report[0].file, "tools/tsconfig.tools.json");
    assert_eq!(report[0].line, 1);
    assert!(report[0].message.contains("no CI typecheck registration"));
}

#[test]
fn append_style_or_non_always_commands_do_not_count_as_local_gates() {
    let root = fixture_root("missing-local");
    let report = findings(&root, &config(&root));
    assert_eq!(report.len(), 1, "{report:#?}");
    assert_eq!(report[0].file, "app/tsconfig.json");
    assert!(report[0]
        .message
        .contains("no local typecheck registration"));
}

#[test]
fn workflow_defaults_step_directories_and_shell_cwds_are_resolved() {
    let root = fixture_root("working-directories");
    let report = findings(&root, &config(&root));
    assert!(report.is_empty(), "unexpected findings: {report:#?}");
}

#[test]
fn non_enforcing_or_non_runnable_workflow_commands_do_not_cover_projects() {
    let root = fixture_root("non-enforcing-workflow");
    let report = findings(&root, &config(&root));
    assert_eq!(report.len(), 11, "{report:#?}");
    for project in [
        "disabled-job/tsconfig.json",
        "disabled-step/tsconfig.json",
        "nonblocking-job/tsconfig.json",
        "nonblocking-step/tsconfig.json",
        "expression/tsconfig.json",
        "constant-nonblocking-step/tsconfig.json",
        "failure-mode-mutated/tsconfig.json",
        "non-posix-shell/tsconfig.json",
        "missing-runner/tsconfig.json",
        "dynamic-runner/tsconfig.json",
        "implicit-windows-shell/tsconfig.json",
    ] {
        assert!(report.iter().any(|finding| {
            finding.file == project && finding.message.contains("no CI typecheck registration")
        }));
    }
    // Unresolved expressions fail open as enforcing; the rule does not guess
    // whether a dynamic condition will disable the gate at runtime.
    assert!(report
        .iter()
        .all(|finding| finding.file != "dynamic-expression/tsconfig.json"));
}

#[test]
fn validates_allowlist_reasons_staleness_and_normalized_collisions() {
    let root = fixture_root("allowlist-errors");
    let report = findings(&root, &config(&root));
    assert!(report.iter().any(|finding| {
        finding.file == ".no-mistakes.yml" && finding.message.contains("non-empty reason")
    }));
    assert!(report
        .iter()
        .any(|finding| finding.message.contains("stale allowProjects entry")));
    assert!(report
        .iter()
        .any(|finding| finding.message.contains("static repository-relative")));
    assert!(report
        .iter()
        .any(|finding| finding.message.contains("is not a tsconfig path")));
    assert!(report
        .iter()
        .any(|finding| finding.message.contains("normalize to the same path")));
    assert!(report.iter().any(|finding| {
        finding.file == "allowed/tsconfig.json"
            && finding.message.contains("no CI typecheck registration")
    }));
}

#[test]
fn reasoned_allowlist_entry_exempts_an_auxiliary_project_from_both_gates() {
    let root = fixture_root("allowlist-pass");
    let report = findings(&root, &config(&root));
    assert!(report.is_empty(), "unexpected findings: {report:#?}");
}

#[test]
fn ignores_node_modules_and_reports_malformed_workflows_once() {
    let root = fixture_root("malformed-workflow");
    let report = findings(&root, &config(&root));
    assert_eq!(
        report
            .iter()
            .filter(|finding| finding.file == ".github/workflows/bad.yml")
            .count(),
        1,
        "{report:#?}"
    );
    assert!(report
        .iter()
        .all(|finding| !finding.file.contains("node_modules")));
}

#[test]
fn normal_rule_filtering_keeps_the_prepared_api_scope_aware() {
    let root = fixture_root("missing-ci");
    let mut config = config(&root);
    config.rules = vec![RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        include: vec!["app/**".to_string()],
        ..Default::default()
    }];
    let report = findings(&root, &config);
    assert!(report.is_empty(), "unexpected findings: {report:#?}");
}

#[test]
fn invalid_rule_filter_is_returned_without_partial_coverage_findings() {
    let root = fixture_root("missing-ci");
    let mut config = config(&root);
    config.rules = vec![RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        include: vec!["[".to_string()],
        ..Default::default()
    }];
    assert!(check(&root, &config).is_err());
}

#[test]
fn tsconfig_inventory_keeps_only_tracked_compiler_configs() {
    let root = Path::new("/repo");
    let paths = vec![
        root.join("tsconfig.json"),
        root.join("tools/tsconfig.build.json"),
        root.join("node_modules/library/tsconfig.json"),
        root.join("tsconfig"),
    ];
    assert_eq!(
        tracked_tsconfigs(root, &paths),
        BTreeSet::from([
            "tools/tsconfig.build.json".to_string(),
            "tsconfig.json".to_string()
        ])
    );
    assert!(is_tsconfig_path("nested/tsconfig.extra.json"));
    assert!(!is_tsconfig_path("tsconfig."));
    assert!(!is_tsconfig_path("nested/tsconfig.json.bak"));
}

#[test]
fn pure_helpers_keep_config_and_workflow_boundaries_static() {
    let configured: Value =
        serde_yaml::from_str("defaults:\n  run:\n    working-directory: packages/app\n").unwrap();
    let dynamic: Value =
        serde_yaml::from_str("defaults:\n  run:\n    working-directory: ${{ matrix.package }}\n")
            .unwrap();
    assert_eq!(default_working_directory(&configured), Some("packages/app"));
    assert_eq!(
        effective_working_directory(&configured, Some(".".into())),
        Some("packages/app".into())
    );
    assert_eq!(
        effective_working_directory(&dynamic, Some(".".into())),
        None
    );
    assert_eq!(
        effective_working_directory(&Value::Null, Some("fallback".into())),
        Some("fallback".into())
    );
    assert_eq!(config_file(Path::new("/repo"), None), ".no-mistakes.yml");
    assert_eq!(
        config_file(
            Path::new("/repo"),
            Some(Path::new("/repo/config/no-mistakes.yml"))
        ),
        "config/no-mistakes.yml"
    );
}

#[test]
fn workflow_load_errors_are_rendered_for_both_failure_kinds() {
    let workflows = ParsedWorkflowSet {
        documents: vec![
            ParsedWorkflowDocument {
                path: ".github/workflows/read.yml".into(),
                value: Err(WorkflowDocumentError {
                    kind: WorkflowDocumentErrorKind::Read,
                    message: "permission denied".into(),
                }),
            },
            ParsedWorkflowDocument {
                path: ".github/workflows/parse.yml".into(),
                value: Err(WorkflowDocumentError {
                    kind: WorkflowDocumentErrorKind::Parse,
                    message: "invalid YAML".into(),
                }),
            },
        ],
    };
    let findings = workflow_load_findings(&workflows);
    assert_eq!(findings.len(), 2);
    assert!(findings[0]
        .message
        .contains("could not parse workflow YAML"));
    assert!(findings[1].message.contains("could not read workflow file"));
}

#[test]
fn ci_scanner_skips_workflow_shapes_without_static_runnable_steps() {
    let incomplete: Value = serde_yaml::from_str(
        "on: push\njobs:\n  no-steps:\n    runs-on: ubuntu-latest\n  incomplete:\n    runs-on: ubuntu-latest\n    steps:\n      - working-directory: ${{ matrix.dir }}\n        run: tsc --noEmit\n      - name: no command\n",
    )
    .unwrap();
    let workflows = ParsedWorkflowSet {
        documents: vec![
            ParsedWorkflowDocument {
                path: ".github/workflows/no-jobs.yml".into(),
                value: Ok(Value::Null),
            },
            ParsedWorkflowDocument {
                path: ".github/workflows/incomplete.yml".into(),
                value: Ok(incomplete),
            },
        ],
    };
    assert!(ci_typechecked_projects(&workflows, &BTreeSet::new(), &BTreeMap::new()).is_empty());
}

#[test]
fn ci_scanner_honors_static_posix_shell_overrides_and_defaults() {
    let mut workflow: Value = serde_yaml::from_str(
        "defaults:\n  run:\n    shell: python\njobs:\n  workflow-default-python:\n    steps:\n      - run: tsc --noEmit --project workflow-default-python/tsconfig.json\n  job-default-bash-template:\n    defaults:\n      run:\n        shell: 'bash --noprofile --norc -eo pipefail {0}'\n    steps:\n      - run: tsc --noEmit --project job-default-bash-template/tsconfig.json\n  step-override-sh-template:\n    steps:\n      - shell: 'sh -e {0}'\n        run: tsc --noEmit --project step-override-sh-template/tsconfig.json\n  unsupported-template:\n    defaults:\n      run:\n        shell: bash\n    steps:\n      - shell: 'bash -c {0}'\n        run: tsc --noEmit --project unsupported-template/tsconfig.json\n  dynamic-shell:\n    steps:\n      - shell: ${{ matrix.shell }}\n        run: tsc --noEmit --project dynamic-shell/tsconfig.json\n",
    )
    .unwrap();
    add_static_runners(&mut workflow);
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: ".github/workflows/shells.yml".into(),
            value: Ok(workflow),
        }],
    };

    let expected = BTreeSet::from([
        "job-default-bash-template/tsconfig.json".to_string(),
        "step-override-sh-template/tsconfig.json".to_string(),
    ]);
    assert_eq!(
        ci_typechecked_projects(&workflows, &expected, &project_inputs(&expected)),
        expected
    );
}

#[test]
fn ci_scanner_rejects_an_empty_shell_setting() {
    let mut workflow: Value = serde_yaml::from_str(
        "jobs:\n  empty-shell:\n    steps:\n      - shell: ''\n        run: tsc --noEmit --project app/tsconfig.json\n",
    )
    .unwrap();
    add_static_runners(&mut workflow);
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: ".github/workflows/empty-shell.yml".into(),
            value: Ok(workflow),
        }],
    };
    let tracked = BTreeSet::from(["app/tsconfig.json".to_string()]);
    assert!(ci_typechecked_projects(&workflows, &tracked, &project_inputs(&tracked)).is_empty());
}

#[test]
fn ci_scanner_accepts_only_execution_preserving_shell_template_flags() {
    let mut workflow: Value = serde_yaml::from_str(
        "jobs:\n  bare-bash:\n    steps:\n      - shell: bash\n        run: tsc --noEmit --project bare-bash/tsconfig.json\n  bash-flags:\n    steps:\n      - shell: 'bash -eu -o pipefail {0}'\n        run: tsc --noEmit --project bash-flags/tsconfig.json\n  sh-flags:\n    steps:\n      - shell: 'sh -ux {0}'\n        run: tsc --noEmit --project sh-flags/tsconfig.json\n  syntax-check-only:\n    steps:\n      - shell: 'bash -n {0}'\n        run: tsc --noEmit --project syntax-check-only/tsconfig.json\n  version-only:\n    steps:\n      - shell: 'bash --version {0}'\n        run: tsc --noEmit --project version-only/tsconfig.json\n  shell-without-script-template:\n    steps:\n      - shell: 'bash -e'\n        run: tsc --noEmit --project shell-without-script-template/tsconfig.json\n  sh-pipefail:\n    steps:\n      - shell: 'sh -o pipefail {0}'\n        run: tsc --noEmit --project sh-pipefail/tsconfig.json\n  empty-short-flag:\n    steps:\n      - shell: 'bash - {0}'\n        run: tsc --noEmit --project empty-short-flag/tsconfig.json\n  bare-template-word:\n    steps:\n      - shell: 'bash pipefail {0}'\n        run: tsc --noEmit --project bare-template-word/tsconfig.json\n",
    )
    .unwrap();
    add_static_runners(&mut workflow);
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: ".github/workflows/template-flags.yml".into(),
            value: Ok(workflow),
        }],
    };

    let expected = BTreeSet::from([
        "bare-bash/tsconfig.json".to_string(),
        "bash-flags/tsconfig.json".to_string(),
        "sh-flags/tsconfig.json".to_string(),
    ]);
    assert_eq!(
        ci_typechecked_projects(&workflows, &expected, &project_inputs(&expected)),
        expected
    );
}

#[test]
fn ci_scanner_requires_static_runners_and_shell_failure_propagation() {
    let workflow: Value = serde_yaml::from_str(
        "on: push\njobs:\n  implicit-shell:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project implicit-shell/tsconfig.json; echo later\n  builtin-bash:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: bash\n        run: tsc --noEmit --project builtin-bash/tsconfig.json; echo later\n  custom-final-typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: echo first; tsc --noEmit --project custom-final-typecheck/tsconfig.json\n  custom-masked-typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash {0}'\n        run: tsc --noEmit --project custom-masked-typecheck/tsconfig.json; echo later\n  custom-errexit:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'bash -e {0}'\n        run: tsc --noEmit --project custom-errexit/tsconfig.json; echo later\n  custom-errexit-option:\n    runs-on: ubuntu-latest\n    steps:\n      - shell: 'sh -o errexit {0}'\n        run: tsc --noEmit --project custom-errexit-option/tsconfig.json; echo later\n  missing-runner:\n    steps:\n      - run: tsc --noEmit --project missing-runner/tsconfig.json\n  dynamic-runner:\n    runs-on: ${{ matrix.os }}\n    steps:\n      - run: tsc --noEmit --project dynamic-runner/tsconfig.json\n  bare-self-hosted:\n    runs-on: self-hosted\n    steps:\n      - run: tsc --noEmit --project bare-self-hosted/tsconfig.json\n  label-array-runner:\n    runs-on: [self-hosted, linux]\n    steps:\n      - run: tsc --noEmit --project label-array-runner/tsconfig.json\n  dynamic-label-array-runner:\n    runs-on: [self-hosted, '${{ matrix.os }}']\n    steps:\n      - run: tsc --noEmit --project dynamic-label-array-runner/tsconfig.json\n  implicit-windows:\n    runs-on: Windows-2025\n    steps:\n      - run: tsc --noEmit --project implicit-windows/tsconfig.json\n  implicit-self-hosted-windows:\n    runs-on: [self-hosted, windows]\n    steps:\n      - run: tsc --noEmit --project implicit-self-hosted-windows/tsconfig.json\n  explicit-bash-windows:\n    runs-on: windows-latest\n    steps:\n      - shell: bash\n        run: tsc --noEmit --project explicit-bash-windows/tsconfig.json\n",
    )
    .unwrap();
    let workflows = ParsedWorkflowSet {
        documents: vec![ParsedWorkflowDocument {
            path: ".github/workflows/runners-and-shells.yml".into(),
            value: Ok(workflow),
        }],
    };

    let expected = BTreeSet::from([
        "builtin-bash/tsconfig.json".to_string(),
        "custom-errexit-option/tsconfig.json".to_string(),
        "custom-errexit/tsconfig.json".to_string(),
        "custom-final-typecheck/tsconfig.json".to_string(),
        "explicit-bash-windows/tsconfig.json".to_string(),
        "implicit-shell/tsconfig.json".to_string(),
        "label-array-runner/tsconfig.json".to_string(),
    ]);
    assert_eq!(
        ci_typechecked_projects(&workflows, &expected, &project_inputs(&expected)),
        expected
    );
}

#[test]
fn application_scan_combines_allowlist_and_missing_gate_findings() {
    let tracked = BTreeSet::from(["app/tsconfig.json".to_string()]);
    let options = Options {
        allow_projects: BTreeMap::from([
            ("missing/tsconfig.json".to_string(), "obsolete".to_string()),
            ("app/tsconfig.json".to_string(), "".to_string()),
        ]),
    };
    let findings = scan_application(
        &options,
        &tracked,
        &tracked,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::new(),
        ".no-mistakes.yml",
    );
    assert_eq!(findings.len(), 4, "{findings:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.target.as_deref() == Some("missing/tsconfig.json")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("no CI typecheck registration")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("no local typecheck registration")));
}

#[test]
fn blank_allowlist_reasons_do_not_claim_normalized_paths() {
    let tracked = BTreeSet::from(["app/tsconfig.json".to_string()]);
    let options = Options {
        allow_projects: BTreeMap::from([
            ("./app/tsconfig.json".to_string(), "".to_string()),
            (
                "app/tsconfig.json".to_string(),
                "reasoned exemption".to_string(),
            ),
        ]),
    };

    let findings = scan_application(
        &options,
        &tracked,
        &tracked,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::new(),
        ".no-mistakes.yml",
    );

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].message.contains("non-empty reason"));
}
