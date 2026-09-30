use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/fixture"),
    )
}

fn check(root: &Path, config: &str, format: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", format, "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(config))
        .output()
        .unwrap()
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn reports_each_dead_error_class_with_file_and_line() {
    let out = check(&fixture(), ".no-mistakes.yml", "human");
    let body = text(&out);
    assert_eq!(out.status.code(), Some(1), "{body}");
    assert_eq!(body.lines().count(), 14, "{body}");
    assert!(
        body.contains(
            "unconstructed-error-class src/errors.ts:2 exported error class `UnusedError` \
             is never constructed or subclassed in non-test source"
        ),
        "{body}"
    );
    assert!(body.contains("src/guard.ts:3 exported error class `InstanceofOnlyError`"));
    assert!(body.contains("src/hierarchy.ts:11 exported error class `Grandchild`"));
    for constructed in [
        "ConstructedError",
        "BarrelError",
        "WorkspaceError",
        "AppError",
    ] {
        assert!(!body.contains(constructed), "{constructed}: {body}");
    }
}

#[test]
fn json_output_carries_the_configured_message_and_target() {
    let out = check(&fixture(), "configs/message.yml", "json");
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert_eq!(
        report["rules"],
        serde_json::json!([{
            "file": "packages/lib/index.ts",
            "line": 5,
            "message": "Dead error class: `WorkspaceUnusedError`",
            "rule": "unconstructed-error-class",
            "target": "WorkspaceUnusedError",
        }])
    );
}

#[test]
fn test_files_option_stops_shared_helpers_from_counting_as_source() {
    let default = text(&check(&fixture(), ".no-mistakes.yml", "human"));
    let configured = text(&check(&fixture(), "configs/test-files.yml", "human"));
    assert!(!default.contains("HelperOnlyError"), "{default}");
    assert!(
        configured.contains("src/helper-only.ts:3 exported error class `HelperOnlyError`"),
        "{configured}"
    );
}

#[test]
fn invalid_options_skip_the_check_and_fail() {
    let unknown = check(&fixture(), "configs/unknown-option.yml", "human");
    assert!(!unknown.status.success());
    assert!(text(&unknown).contains("unknown field `testGlobs`"));

    let invalid = check(&fixture(), "configs/invalid-test-files.yml", "human");
    assert!(!invalid.status.success());
    assert!(text(&invalid).contains("options.testFiles contains invalid glob"));
}
