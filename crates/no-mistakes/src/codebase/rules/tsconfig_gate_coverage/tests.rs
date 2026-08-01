use super::application::resolve_gate_projects_against_tracked;
use super::workflow::{
    default_working_directory, effective_working_directory, is_repo_relative_project_path,
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

fn check(root: &Path, config: &NoMistakesConfig) -> anyhow::Result<Vec<RuleFinding>> {
    let paths = crate::codebase::ts_source::discover_files(root, &[]);
    let workflows = ParsedWorkflowSet::load(root, &config.ci);
    let sources = super::super::source_store_for_files(&paths);
    check_with_prepared(
        root,
        config,
        PreparedInputs {
            tracked_paths: &paths,
            workflows: &workflows,
            _sources: &sources,
            config_path: Some(&root.join(".no-mistakes.yml")),
        },
    )
}

#[test]
fn directory_project_arguments_cover_tracked_primary_tsconfig_in_ci_and_local_checks() {
    let root = fixture_root("pass");
    let report = findings(&root, &config(&root));
    assert!(report.is_empty(), "unexpected findings: {report:#?}");
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
fn statically_disabled_or_nonblocking_workflow_commands_do_not_cover_projects() {
    let root = fixture_root("non-enforcing-workflow");
    let report = findings(&root, &config(&root));
    assert_eq!(report.len(), 6, "{report:#?}");
    for project in [
        "disabled-job/tsconfig.json",
        "disabled-step/tsconfig.json",
        "nonblocking-job/tsconfig.json",
        "nonblocking-step/tsconfig.json",
        "expression/tsconfig.json",
        "constant-nonblocking-step/tsconfig.json",
    ] {
        assert!(report.iter().any(|finding| {
            finding.file == project && finding.message.contains("no CI typecheck registration")
        }));
    }
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
    assert!(is_repo_relative_project_path("app/tsconfig.json"));
    assert!(!is_repo_relative_project_path("../tsconfig.json"));
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
        "jobs:\n  no-steps: {}\n  incomplete:\n    steps:\n      - working-directory: ${{ matrix.dir }}\n        run: tsc --noEmit\n      - name: no command\n",
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
    assert!(ci_typechecked_projects(&workflows).is_empty());
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
        ".no-mistakes.yml",
    );

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].message.contains("non-empty reason"));
}
