use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/postgres-executor-names")
        .canonicalize()
        .unwrap()
}

fn check(config: &str) -> Output {
    let root = root();
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(format!("{config}.yml")))
        .args(["--format", "json"])
        .output()
        .unwrap()
}

fn report(config: &str) -> (Output, Value) {
    let output = check(config);
    let report = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "{config}: {error}: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output, report)
}

#[test]
fn executor_names_override_warns_and_extension_restores_all_default_sites() {
    for (config, findings, warning) in [("override", 1, true), ("extension", 4, false)] {
        let (output, report) = report(config);
        assert_eq!(output.status.code(), Some(1), "{config}: {report}");
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            findings,
            "{report}"
        );
        let warnings = report["warnings"].to_string();
        assert_eq!(warnings.contains("executorNames"), warning, "{report}");
        if warning {
            for omitted in ["query", "read", "write"] {
                assert!(warnings.contains(omitted), "{report}");
            }
            assert!(warnings.contains("extraExecutorNames"), "{report}");
            assert!(warnings.contains("sites.ts:"), "{report}");
        }
    }
}

#[test]
fn executor_names_warnings_do_not_fail_checks_and_follow_project_inheritance() {
    for config in ["warning-only", "project-override", "offset"] {
        let (output, report) = report(config);
        assert!(output.status.success(), "{config}: {report}");
        let warnings = report["warnings"].to_string();
        assert!(warnings.contains("executorNames"), "{config}: {report}");
        assert!(warnings.contains("read"), "{config}: {report}");
        assert!(warnings.contains("annotated.ts:2"), "{config}: {report}");
    }
}

#[test]
fn executor_names_do_not_warn_for_unused_or_out_of_scope_defaults() {
    for config in [
        "never-imported",
        "shadowed",
        "type-only",
        "unrelated",
        "unused",
        "excluded",
        "project-extension",
    ] {
        let (output, report) = report(config);
        assert!(output.status.success(), "{config}: {report}");
        assert_eq!(
            report["warnings"],
            serde_json::json!([]),
            "{config}: {report}"
        );
    }
}

#[test]
fn executor_names_and_extra_executor_names_are_mutually_exclusive() {
    for config in ["conflict", "project-conflict"] {
        let output = check(config);
        assert!(!output.status.success(), "{config}");
        let diagnostic = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            diagnostic.contains("executorNames"),
            "{config}: {diagnostic}"
        );
        assert!(
            diagnostic.contains("extraExecutorNames"),
            "{config}: {diagnostic}"
        );
    }
}

#[test]
fn postgres_executor_names_check_json_snapshot() {
    let root = root();
    let cases = [
        "override",
        "extension",
        "warning-only",
        "never-imported",
        "shadowed",
        "type-only",
        "unrelated",
        "unused",
        "excluded",
        "project-override",
        "project-extension",
        "offset",
    ]
    .map(|name| (name, root.clone(), root.join(format!("{name}.yml"))));
    crate::check_json_snapshot::assert_saved_fixture("postgres_executor_names", &cases);
}
