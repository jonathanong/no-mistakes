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

/// The rule shares the canonical graph with `forbidden-calls` and with the
/// consumers that walk every edge kind, so configuring it must not change what
/// any of them report.
#[test]
fn other_graph_rules_are_unchanged_when_the_rule_is_also_configured() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-calls/shared-graph-parity/fixture"),
    );
    // `danger()` and `new Boom()` go through `@fixture/lib`, which the call
    // graph resolves into the workspace package.
    let forbidden_calls = [
        "forbidden-calls src/entry.ts:5 forbidden call (application #1): \
         repository function `packages/lib/index.ts#danger`",
        "forbidden-calls src/entry.ts:6 forbidden call (application #1): \
         repository function `src/local.ts#localDanger`",
        "forbidden-calls src/entry.ts:7 forbidden call (application #1): \
         repository function `packages/lib/index.ts#Boom`",
    ];
    // Configuring the rule adds the `Child extends Parent` edge to the graph
    // both walkers read; they must report exactly what they do without it.
    let forbidden_dependencies = [
        "forbidden-dependencies src/hierarchy-entry.ts:1 src/hierarchy-entry.ts reaches \
         forbidden file 'src/parent.ts' via import. Reproduce: no-mistakes dependencies \
         'src/hierarchy-entry.ts' --filter 'src/parent.ts' --relationship all --format json",
    ];
    let reachability = [
        "required-entrypoint-reachability src/orphan.ts:1 src/orphan.ts is not \
         runtime-reachable from configured entrypoints: src/entry.ts,src/hierarchy-entry.ts",
    ];
    let without = text(&check(&root, ".no-mistakes.yml", "human"));
    let with = text(&check(&root, "with-unconstructed-error-class.yml", "human"));

    for (rule, expected) in [
        ("forbidden-calls", &forbidden_calls[..]),
        ("forbidden-dependencies", &forbidden_dependencies[..]),
        ("required-entrypoint-reachability", &reachability[..]),
    ] {
        assert_eq!(lines_of(&without, rule), expected, "{without}");
        assert_eq!(lines_of(&with, rule), expected, "{with}");
    }
    assert!(lines_of(&without, "unconstructed-error-class").is_empty());
    // The rule sees `Boom` constructed through the workspace name and `Parent`
    // subclassed by the constructed `Child`.
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
    // A rule that errors is downgraded to a `rules check skipped` warning, and
    // the check fails with exit code 1 (not the usage-error code 2). Exit code 1
    // is also the findings code, so each case pins its warning text as well.
    let unknown = check(&fixture(), "configs/unknown-option.yml", "human");
    assert_eq!(unknown.status.code(), Some(1), "{}", text(&unknown));
    assert!(text(&unknown).contains("unknown field `testGlobs`"));

    let invalid = check(&fixture(), "configs/invalid-test-files.yml", "human");
    assert_eq!(invalid.status.code(), Some(1), "{}", text(&invalid));
    assert!(text(&invalid).contains("options.testFiles contains invalid glob"));
}
