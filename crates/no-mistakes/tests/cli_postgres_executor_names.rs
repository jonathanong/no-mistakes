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
    for config in [
        "warning-only",
        "project-override",
        "offset",
        "namespace",
        "wrapped",
        "sequence",
    ] {
        let (output, report) = report(config);
        assert!(output.status.success(), "{config}: {report}");
        let warnings = report["warnings"].to_string();
        assert!(warnings.contains("executorNames"), "{config}: {report}");
        assert!(warnings.contains("read"), "{config}: {report}");
        let file = match config {
            "namespace" | "wrapped" | "sequence" => config,
            _ => "annotated",
        };
        assert!(
            warnings.contains(&format!("{file}.ts:2")),
            "{config}: {report}"
        );
    }
}

#[test]
fn executor_names_do_not_warn_for_unused_or_out_of_scope_defaults() {
    for config in [
        "null-import",
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
fn excluded_executors_are_removed_after_selection_without_omission_warnings() {
    for (config, findings) in [
        ("exclude-override", 2),
        ("exclude-defaults", 2),
        ("exclude-any-module", 2),
        ("exclude-all", 0),
        ("exclude-all-scoped", 2),
        ("exclude-extras", 3),
        ("project-exclude", 0),
    ] {
        let (output, report) = report(config);
        assert_eq!(output.status.success(), findings == 0, "{config}: {report}");
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            findings,
            "{report}"
        );
        assert_eq!(
            report["warnings"],
            serde_json::json!([]),
            "{config}: {report}"
        );
        assert!(
            !report["rules"].to_string().contains("SELECT 2"),
            "{report}"
        );
    }
}

#[test]
fn unused_exclusions_warn_without_failing_checks() {
    for config in ["exclude-unused", "exclude-unimported"] {
        let (output, report) = report(config);
        assert!(output.status.success(), "{config}: {report}");
        let warnings = report["warnings"].to_string();
        assert!(warnings.contains("excludeExecutorNames"), "{report}");
        assert!(warnings.contains("read"), "{report}");
    }
    let (_, report) = report("exclude-invalid");
    let warnings = report["warnings"].to_string();
    assert!(warnings.contains("excludeExecutorNames"), "{report}");
    assert!(warnings.contains("readPool"), "{report}");
}

#[test]
fn postgres_executor_names_check_json_snapshot() {
    let root = root();
    let cases = [
        "exclude-override",
        "exclude-defaults",
        "exclude-any-module",
        "exclude-all",
        "exclude-all-scoped",
        "exclude-extras",
        "exclude-invalid",
        "exclude-unused",
        "exclude-unimported",
        "project-exclude",
        "override",
        "extension",
        "warning-only",
        "null-import",
        "never-imported",
        "shadowed",
        "type-only",
        "unrelated",
        "unused",
        "excluded",
        "project-override",
        "project-extension",
        "offset",
        "namespace",
        "wrapped",
        "sequence",
    ]
    .map(|name| (name, root.clone(), root.join(format!("{name}.yml"))));
    crate::check_json_snapshot::assert_saved_fixture("postgres_executor_names", &cases);
}
