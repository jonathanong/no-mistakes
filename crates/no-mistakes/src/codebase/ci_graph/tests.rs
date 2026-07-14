#![cfg(test)]

mod discovery_visibility;

use super::env_query::{analyze_env, EnvLocationKind, EnvScope};
use super::impact::analyze_impact;
use super::model::{PermissionLevel, PermissionSpec};
use super::parse::parse_workflow;
use super::permissions::{effective_permissions, PermissionSource};
use super::triggers::{evaluate_trigger, TriggerMatch};
use super::{discover_workflow_files, relative_slash, WorkflowSet};
use crate::config::v2::load_v2_config;
use crate::config::v2::schema::CiConfig;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/ci-graph")
            .join(name),
    )
}

fn changed(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|p| p.to_string()).collect()
}

// ── parse ────────────────────────────────────────────────────────────

#[test]
fn parses_on_string() {
    let wf = parse_workflow("on: push\njobs: {}\n", "w.yml").unwrap();
    assert!(wf.triggers.events.contains_key("push"));
    assert!(wf.triggers.events["push"].is_unconstrained());
}

#[test]
fn parses_on_list() {
    let wf = parse_workflow("on: [push, schedule]\n", "w.yml").unwrap();
    assert!(wf.triggers.events.contains_key("push"));
    assert_eq!(wf.triggers.other_events, vec!["schedule".to_string()]);
}

#[test]
fn parses_workflow_call_as_reusable() {
    let wf = parse_workflow("on:\n  workflow_call: {}\n", "w.yml").unwrap();
    assert!(wf.is_reusable);
}

#[test]
fn permission_shorthands() {
    assert_eq!(spec("permissions: read-all"), PermissionSpec::ReadAll);
    assert_eq!(spec("permissions: write-all"), PermissionSpec::WriteAll);
    assert_eq!(spec("permissions: {}"), PermissionSpec::Empty);
    assert_eq!(spec("permissions: none"), PermissionSpec::Empty);
    assert_eq!(spec("jobs: {}"), PermissionSpec::Unspecified);
    // `bogus` has an unknown level (dropped); `weird` has a non-string value
    // (skipped via the `(Some, None)` branch).
    match spec("permissions:\n  contents: read\n  bogus: sideways\n  weird: [a]") {
        PermissionSpec::Map(map) => {
            assert_eq!(map.get("contents"), Some(&PermissionLevel::Read));
            assert!(!map.contains_key("bogus"));
            assert!(!map.contains_key("weird"));
        }
        other => panic!("expected map, got {other:?}"),
    }
}

fn spec(yaml: &str) -> PermissionSpec {
    parse_workflow(yaml, "w.yml").unwrap().permissions
}

#[test]
fn both_paths_and_ignore_warns() {
    let yaml = "on:\n  push:\n    paths: [\"a/**\"]\n    paths-ignore: [\"a/b/**\"]\n";
    let wf = parse_workflow(yaml, "w.yml").unwrap();
    assert_eq!(wf.warnings.len(), 1);
    assert!(wf.triggers.events["push"].paths_ignore.is_empty());
}

#[test]
fn jobs_sorted_with_name_and_uses() {
    let yaml = "jobs:\n  b:\n    name: Bee\n  a:\n    uses: ./.github/workflows/x.yml\n";
    let wf = parse_workflow(yaml, "w.yml").unwrap();
    assert_eq!(wf.jobs[0].id, "a");
    assert_eq!(
        wf.jobs[0].uses.as_deref(),
        Some("./.github/workflows/x.yml")
    );
    assert_eq!(wf.jobs[1].name.as_deref(), Some("Bee"));
}

// ── triggers / impact ────────────────────────────────────────────────

#[test]
fn impact_matches_paths_and_always() {
    let set = WorkflowSet::load(&fixture("triggers"), &CiConfig::default());
    let report = analyze_impact(&set, &changed(&["src/app.ts"]));
    let paths: Vec<&str> = report.workflows.iter().map(|w| w.path.as_str()).collect();
    assert!(paths.contains(&".github/workflows/paths.yml"));
    assert!(paths.contains(&".github/workflows/list.yml"));
    assert!(paths.contains(&".github/workflows/ignore.yml"));
    // dispatch.yml and reusable.yml are not file-triggered.
    assert!(!paths.contains(&".github/workflows/dispatch.yml"));
    assert!(!paths.contains(&".github/workflows/reusable.yml"));
    // tag-only push (release) is not triggered by a branch file change.
    assert!(!paths.contains(&".github/workflows/tagonly.yml"));
    // both.yml has paths api/** which does not match.
    assert!(!paths.contains(&".github/workflows/both.yml"));
    // the both-paths warning is surfaced.
    assert!(report.warnings.iter().any(|w| w.path.ends_with("both.yml")));
    // Exercise the report's derived Clone/PartialEq/Debug.
    assert_eq!(report, report.clone());
    assert!(!format!("{report:?}").is_empty());
}

#[test]
fn impact_negation_excludes_docs() {
    let set = WorkflowSet::load(&fixture("triggers"), &CiConfig::default());
    let report = analyze_impact(&set, &changed(&["src/docs/readme.md"]));
    let paths_wf = report
        .workflows
        .iter()
        .find(|w| w.path.ends_with("paths.yml"));
    // src/docs/** is negated, so paths.yml is not matched (but list.yml always is).
    assert!(paths_wf.is_none());
    assert!(report
        .workflows
        .iter()
        .any(|w| w.path.ends_with("list.yml")));
}

#[test]
fn impact_paths_ignore_excludes_matching_file() {
    let set = WorkflowSet::load(&fixture("triggers"), &CiConfig::default());
    let report = analyze_impact(&set, &changed(&["docs/guide.md"]));
    // ignore.yml ignores docs/**, so it should not appear.
    assert!(!report
        .workflows
        .iter()
        .any(|w| w.path.ends_with("ignore.yml")));
}

#[test]
fn impact_both_paths_matches_api() {
    let set = WorkflowSet::load(&fixture("triggers"), &CiConfig::default());
    let report = analyze_impact(&set, &changed(&["api/handler.ts"]));
    let both = report
        .workflows
        .iter()
        .find(|w| w.path.ends_with("both.yml"))
        .expect("both.yml should match api/**");
    assert_eq!(both.trigger, TriggerMatch::Matched);
}

#[test]
fn evaluate_trigger_no_path_events() {
    let wf = parse_workflow("on: workflow_dispatch\n", "w.yml").unwrap();
    assert_eq!(evaluate_trigger(&wf, "a.ts").0, TriggerMatch::NoPathEvents);
}

#[test]
fn evaluate_trigger_not_matched() {
    let wf = parse_workflow("on:\n  push:\n    paths: [\"src/**\"]\n", "w.yml").unwrap();
    assert_eq!(
        evaluate_trigger(&wf, "docs/x.md").0,
        TriggerMatch::NotMatched
    );
}

// ── permissions ──────────────────────────────────────────────────────

#[test]
fn permission_resolution_across_jobs() {
    let set = WorkflowSet::load(&fixture("permissions"), &CiConfig::default());
    let workflow = set
        .workflows
        .iter()
        .find(|w| w.path.ends_with("perms.yml"))
        .unwrap();

    let job = |id: &str| workflow.jobs.iter().find(|j| j.id == id).unwrap();

    let inherit = effective_permissions(workflow, job("inherit"));
    assert_eq!(inherit.source, PermissionSource::Workflow);
    assert_eq!(inherit.scopes.get("issues"), Some(&PermissionLevel::Write));

    let override_job = effective_permissions(workflow, job("override"));
    assert_eq!(override_job.source, PermissionSource::Job);
    assert_eq!(
        override_job.scopes.get("packages"),
        Some(&PermissionLevel::Write)
    );
    assert!(!override_job.scopes.contains_key("contents"));

    let readall = effective_permissions(workflow, job("readall"));
    assert_eq!(readall.scopes.get("contents"), Some(&PermissionLevel::Read));
    // id-token is write-only, so read-all omits it rather than reporting `read`.
    assert!(!readall.scopes.contains_key("id-token"));
    // read-only scopes are reported as read.
    assert_eq!(readall.scopes.get("models"), Some(&PermissionLevel::Read));

    let writeall = effective_permissions(workflow, job("writeall"));
    // read-write scopes become write; the write-only scope is granted write.
    assert_eq!(
        writeall.scopes.get("contents"),
        Some(&PermissionLevel::Write)
    );
    assert_eq!(
        writeall.scopes.get("id-token"),
        Some(&PermissionLevel::Write)
    );
    // write-all caps read-only scopes at read (GitHub never grants them write).
    assert_eq!(writeall.scopes.get("models"), Some(&PermissionLevel::Read));
    assert_eq!(
        writeall.scopes.get("vulnerability-alerts"),
        Some(&PermissionLevel::Read)
    );

    let empty = effective_permissions(workflow, job("empty"));
    // `permissions: {}` still carries the non-configurable metadata: read.
    assert_eq!(empty.scopes.get("metadata"), Some(&PermissionLevel::Read));
    assert_eq!(empty.scopes.len(), 1);
}

#[test]
fn permission_assumed_default() {
    let set = WorkflowSet::load(&fixture("permissions"), &CiConfig::default());
    let default_wf = set
        .workflows
        .iter()
        .find(|w| w.path.ends_with("default.yml"))
        .unwrap();
    let resolved = effective_permissions(default_wf, &default_wf.jobs[0]);
    assert_eq!(resolved.source, PermissionSource::Default);
    assert!(resolved.assumed_default);
    assert_eq!(
        resolved.scopes.get("contents"),
        Some(&PermissionLevel::Read)
    );
}

// ── env ──────────────────────────────────────────────────────────────

#[test]
fn env_workflow_definition_and_reference() {
    let report = analyze_env(
        &fixture("env"),
        &CiConfig::default(),
        "CIGRAPH_WORKFLOW_VAR",
    );
    let file = &report.files[0];
    assert!(file
        .locations
        .iter()
        .any(|l| l.kind == EnvLocationKind::Definition && l.scope == EnvScope::Workflow));
    assert!(file
        .locations
        .iter()
        .any(|l| l.kind == EnvLocationKind::Reference));
}

#[test]
fn env_step_scope_definition_and_reference() {
    let report = analyze_env(&fixture("env"), &CiConfig::default(), "CIGRAPH_STEP_VAR");
    let file = &report.files[0];
    assert!(file
        .locations
        .iter()
        .any(|l| l.kind == EnvLocationKind::Definition && l.scope == EnvScope::Step));
    // The var is referenced in two distinct steps (a `run:` and a `with:`); both
    // occurrences are preserved rather than deduped into one.
    let refs = file
        .locations
        .iter()
        .filter(|l| l.kind == EnvLocationKind::Reference && l.scope == EnvScope::Step)
        .count();
    assert_eq!(refs, 2);
}

#[test]
fn env_numeric_value_coerced() {
    let report = analyze_env(&fixture("env"), &CiConfig::default(), "CIGRAPH_NUMERIC");
    let def = report.files[0]
        .locations
        .iter()
        .find(|l| l.kind == EnvLocationKind::Definition)
        .unwrap();
    assert_eq!(def.value.as_deref(), Some("42"));
}

#[test]
fn env_unknown_variable_is_empty() {
    let report = analyze_env(&fixture("env"), &CiConfig::default(), "NOPE");
    assert!(report.files.is_empty());
}

#[test]
fn env_non_scalar_value_has_no_value() {
    let report = analyze_env(&fixture("env"), &CiConfig::default(), "CIGRAPH_LIST");
    let def = report.files[0]
        .locations
        .iter()
        .find(|l| l.kind == EnvLocationKind::Definition)
        .unwrap();
    assert!(def.value.is_none());
    // Exercise the report's derived Clone/PartialEq/Debug.
    assert_eq!(report, report.clone());
    assert!(!format!("{report:?}").is_empty());
}

#[test]
fn paths_accept_single_string() {
    let wf = parse_workflow("on:\n  push:\n    paths: src/**\n", "w.yml").unwrap();
    assert_eq!(wf.triggers.events["push"].paths, vec!["src/**".to_string()]);
}

#[test]
fn permission_spec_unsupported_shape_is_unspecified() {
    assert_eq!(spec("permissions:\n  - a\n"), PermissionSpec::Unspecified);
}

// ── discovery / config / warnings ────────────────────────────────────

#[test]
fn custom_workflow_dir_from_config() {
    let root = fixture("custom-dir");
    let config = load_v2_config(&root, None).unwrap();
    assert_eq!(config.ci.workflow_dirs, vec!["ci-pipelines".to_string()]);
    let set = WorkflowSet::load(&root, &config.ci);
    assert_eq!(set.workflows.len(), 1);
    let report = analyze_impact(&set, &changed(&["lib/x.ts"]));
    assert_eq!(report.workflows.len(), 1);
}

#[test]
fn malformed_workflow_produces_warning() {
    let set = WorkflowSet::load(&fixture("malformed"), &CiConfig::default());
    assert!(set.workflows.is_empty());
    assert_eq!(set.warnings.len(), 1);
    // env analysis surfaces the same parse warning.
    let report = analyze_env(&fixture("malformed"), &CiConfig::default(), "X");
    assert_eq!(report.warnings.len(), 1);
}

#[test]
fn unreadable_workflow_produces_read_warning() {
    // The fixture has a directory named `broken.yml`; reading it fails as I/O.
    let set = WorkflowSet::load(&fixture("unreadable"), &CiConfig::default());
    assert!(set.workflows.is_empty());
    assert!(set
        .warnings
        .iter()
        .any(|w| w.message.contains("could not read")));
    let report = analyze_env(&fixture("unreadable"), &CiConfig::default(), "X");
    assert!(report
        .warnings
        .iter()
        .any(|w| w.message.contains("could not read")));
}

#[test]
fn anchors_and_aliases_resolve() {
    let set = WorkflowSet::load(&fixture("anchor"), &CiConfig::default());
    let wf = &set.workflows[0];
    assert!(wf.triggers.events["push"]
        .paths
        .contains(&"pkg/**".to_string()));
    let resolved = effective_permissions(wf, &wf.jobs[0]);
    assert_eq!(
        resolved.scopes.get("contents"),
        Some(&PermissionLevel::Read)
    );
}

#[test]
fn discover_skips_missing_dirs() {
    let files = discover_workflow_files(Path::new("/nonexistent-xyz"), &CiConfig::default());
    assert!(files.is_empty());
}

#[test]
fn discovery_uses_git_visibility_and_keeps_tracked_ignored_workflows() {
    let dir = crate::test_support::materialize_gitignore_fixture("auto-discovery");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    crate::test_support::git_add_force(dir.path(), &[".github/workflows/tracked-ignored.yml"]);

    let files: Vec<String> = discover_workflow_files(dir.path(), &CiConfig::default())
        .iter()
        .map(|path| relative_slash(dir.path(), path))
        .collect();

    assert!(files.contains(&".github/workflows/visible.yml".to_string()));
    assert!(files.contains(&".github/workflows/tracked-ignored.yml".to_string()));
    assert!(files.contains(&".github/workflows/broken.yml".to_string()));
    assert!(!files.contains(&".github/workflows/ignored.yml".to_string()));
    assert!(analyze_env(dir.path(), &CiConfig::default(), "IGNORED_ENV")
        .files
        .is_empty());
    assert!(
        !analyze_env(dir.path(), &CiConfig::default(), "TRACKED_IGNORED_ENV")
            .files
            .is_empty()
    );
}

#[test]
fn relative_slash_outside_root_returns_input() {
    let rel = relative_slash(Path::new("/a/b"), Path::new("/c/d.yml"));
    assert_eq!(rel, "/c/d.yml");
}
