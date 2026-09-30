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
    assert_eq!(body.lines().count(), 16, "{body}");
    assert!(
        body.contains(
            "unconstructed-error-class src/errors.ts:2 exported error class `UnusedError` \
             is never constructed or subclassed in non-test source"
        ),
        "{body}"
    );
    assert!(body.contains("src/guard.ts:3 exported error class `InstanceofOnlyError`"));
    assert!(body.contains("src/hierarchy.ts:11 exported error class `Grandchild`"));
    // A class nested in a namespace is reported by its own name.
    assert!(body.contains("src/namespaced.ts:14 exported error class `DeadNamespacedError`"));
    for silent in [
        // `new this()` in the factory of a class nested in a namespace builds it.
        "TopicError",
        "ConstructedError",
        "BarrelError",
        "WorkspaceError",
        "AppError",
        // Declaration files describe code outside the analyzed source.
        "AmbientClientError",
        "AmbientModuleError",
        "AmbientCommonError",
        // Each suppression directive form keeps its class out of the report.
        "SuppressedError",
        "LineSuppressedError",
        "FileSuppressedError",
    ] {
        assert!(!body.contains(silent), "{silent}: {body}");
    }
}

fn lines_of(body: &str, rule: &str) -> Vec<String> {
    let prefix = format!("{rule} ");
    body.lines()
        .filter(|line| line.starts_with(&prefix))
        .map(String::from)
        .collect()
}

/// The rule follows `@scope/package` names into the workspace; no other rule's
/// resolution does, so configuring it must not change their output.
#[test]
fn forbidden_calls_output_is_unchanged_when_the_rule_is_also_configured() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-calls/workspace-package/fixture"),
    );
    // `danger()` and `new Boom()` go through `@fixture/lib`, which call sites
    // never resolve into the workspace, so only the module-export target and
    // the relative repository call are reported.
    let expected = [
        "forbidden-calls src/entry.ts:5 forbidden call (application #1): \
         module export `@fixture/lib#danger`",
        "forbidden-calls src/entry.ts:6 forbidden call (application #1): \
         repository function `src/local.ts#localDanger`",
    ];
    let without = text(&check(&root, ".no-mistakes.yml", "human"));
    let with = text(&check(&root, "with-unconstructed-error-class.yml", "human"));

    assert_eq!(lines_of(&without, "forbidden-calls"), expected, "{without}");
    assert_eq!(lines_of(&with, "forbidden-calls"), expected, "{with}");
    // The rule sees `Boom` constructed through the workspace name.
    assert_eq!(
        lines_of(&with, "unconstructed-error-class"),
        [
            "unconstructed-error-class packages/lib/index.ts:5 exported error class `Unused` \
          is never constructed or subclassed in non-test source"
        ],
        "{with}"
    );
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
fn invalid_options_skip_the_check_and_fail() {
    let unknown = check(&fixture(), "configs/unknown-option.yml", "human");
    assert!(!unknown.status.success());
    assert!(text(&unknown).contains("unknown field `testGlobs`"));

    let invalid = check(&fixture(), "configs/invalid-test-files.yml", "human");
    assert!(!invalid.status.success());
    assert!(text(&invalid).contains("options.testFiles contains invalid glob"));
}
