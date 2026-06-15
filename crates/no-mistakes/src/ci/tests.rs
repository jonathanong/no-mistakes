use super::render::{render_env, render_impact};
use super::*;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/ci-graph")
            .join(name),
    )
}

fn impact(dir: &str, file: &str) -> CiImpactReport {
    impact_report(&fixture(dir), None, &[PathBuf::from(file)]).unwrap()
}

#[test]
fn render_impact_all_formats() {
    let report = impact("triggers", "src/app.ts");
    assert!(render_impact(&report, Format::Json)
        .unwrap()
        .contains("\"workflows\""));
    assert!(render_impact(&report, Format::Yml)
        .unwrap()
        .contains("workflows:"));
    assert!(render_impact(&report, Format::Paths)
        .unwrap()
        .contains(".github/workflows/paths.yml"));
    assert!(render_impact(&report, Format::Md)
        .unwrap()
        .contains("- .github"));
    let human = render_impact(&report, Format::Human).unwrap();
    assert!(human.contains("(always)"));
    assert!(human.contains("warning:"));
}

#[test]
fn render_impact_empty() {
    // custom-dir only has a paths-filtered workflow (no always-trigger),
    // so a non-matching file yields an empty report.
    let report = impact("custom-dir", "README.md");
    assert!(render_impact(&report, Format::Human)
        .unwrap()
        .contains("No workflows triggered"));
}

#[test]
fn render_impact_permissions_branches() {
    // perms.yml exercises an explicit map, write/none levels, and empty.
    let report = impact("permissions", "any.ts");
    let human = render_impact(&report, Format::Human).unwrap();
    assert!(human.contains("packages:write"));
    assert!(human.contains("(none)"));
    assert!(human.contains("contents:none"));
    // default.yml renders the assumed-default marker.
    assert!(human.contains("assumed default"));
}

#[test]
fn render_env_all_formats() {
    let report = env_report(&fixture("env"), None, "CIGRAPH_STEP_VAR").unwrap();
    assert!(render_env(&report, Format::Json)
        .unwrap()
        .contains("\"variable\""));
    assert!(render_env(&report, Format::Yml)
        .unwrap()
        .contains("variable:"));
    assert!(render_env(&report, Format::Paths)
        .unwrap()
        .contains(".github/workflows/env.yml"));
    assert!(render_env(&report, Format::Md)
        .unwrap()
        .contains("definition @ step"));
    assert!(render_env(&report, Format::Human)
        .unwrap()
        .contains("reference @ step"));
}

#[test]
fn render_env_empty() {
    let report = env_report(&fixture("env"), None, "MISSING").unwrap();
    assert!(render_env(&report, Format::Human)
        .unwrap()
        .contains("not found"));
}

#[test]
fn render_env_covers_all_scopes_and_warnings() {
    use crate::codebase::ci_graph::env_query::{CiEnvFile, CiEnvLocation};
    use crate::codebase::ci_graph::model::CiWarning;
    let report = CiEnvReport {
        variable: "V".to_string(),
        files: vec![CiEnvFile {
            path: "w.yml".to_string(),
            locations: vec![
                CiEnvLocation {
                    kind: EnvLocationKind::Definition,
                    scope: EnvScope::Workflow,
                    job: None,
                    value: Some("x".to_string()),
                },
                CiEnvLocation {
                    kind: EnvLocationKind::Reference,
                    scope: EnvScope::Job,
                    job: Some("build".to_string()),
                    value: None,
                },
            ],
        }],
        warnings: vec![CiWarning {
            path: "w.yml".to_string(),
            message: "note".to_string(),
        }],
    };
    let human = render_env(&report, Format::Human).unwrap();
    assert!(human.contains("definition @ workflow = x"));
    assert!(human.contains("reference @ job job=build"));
    assert!(human.contains("warning: w.yml: note"));
}

#[test]
fn absolute_changed_file_is_relativized() {
    let abs = fixture("triggers").join("src/app.ts");
    let report = impact_report(&fixture("triggers"), None, &[abs]).unwrap();
    assert_eq!(report.changed_files, vec!["src/app.ts".to_string()]);
}

#[test]
fn run_dispatches_both_subcommands() {
    run(CiArgs {
        command: CiCommand::Impact(CiImpactArgs {
            files: vec![PathBuf::from("src/app.ts")],
            root: fixture("triggers"),
            config: None,
            format: Some(Format::Json),
            json: false,
        }),
    })
    .unwrap();
    run(CiArgs {
        command: CiCommand::Env(CiEnvArgs {
            var: "CIGRAPH_WORKFLOW_VAR".to_string(),
            root: fixture("env"),
            config: None,
            format: None,
            json: true,
        }),
    })
    .unwrap();
}
