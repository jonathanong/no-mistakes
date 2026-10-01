use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/fixture"),
    )
}

fn parse_failure_fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/parse-failure/fixture"),
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
    assert_eq!(body.lines().count(), 51, "{body}");
    assert!(
        body.contains(
            "unconstructed-error-class src/errors.ts:2 exported error class `UnusedError` \
             is never constructed or subclassed in non-test source"
        ),
        "{body}"
    );
    assert!(body.contains("src/guard.ts:3 exported error class `InstanceofOnlyError`"));
    assert!(body.contains("src/hierarchy.ts:11 exported error class `Grandchild`"));
    // A dead namespace member is reported by its qualified name.
    assert!(
        body.contains(
            "unconstructed-error-class src/namespaced.ts:14 exported error class \
             `Errors.DeadNamespacedError` is never constructed or subclassed in non-test source"
        ),
        "{body}"
    );
    assert!(body.contains("src/namespace-lib.ts:31 exported error class `Lib.Deep.DeepDead`"));
    assert!(
        body.contains("src/namespaced.ts:57 exported error class `Dotted.Path.DeadDottedError`")
    );
    // A shadowing parameter, a sourced export clause, and an erased
    // `import type x = require()` each leave a dead class reported.
    assert!(body.contains("src/namespace-shadowed.ts:4 exported error class `Shadowed.ShadowDead`"));
    assert!(body.contains("exported error class `Unseen.RemoteDead`"));
    assert!(body.contains("exported error class `TypeOnly.TypeOnlyError`"));
    // A namespace named only in an erased type, a same-named constant, and a
    // static guard on a class are no use of the namespace.
    assert!(body.contains("exported error class `Queried.QueriedDead`"));
    assert!(body.contains("exported error class `Collide.Inner.CollideDead`"));
    assert!(body.contains("exported error class `Guarded.GuardedDead`"));
    assert!(body.contains("exported error class `Kept.KeptDead`"));
    for silent in [
        // A namespace member built through a reference the graph resolves.
        "Errors.TopicError",
        "Errors.Built",
        "Lib.Used",
        "Lib.Deep.DeepUsed",
        "CollideBase",
        "CollideChild",
        "CollideGrand",
        // A member its namespace does not export.
        "Hidden",
        // A namespace subclass still credits its base.
        "NsBase",
        "Errors.Base",
        // A namespace that escapes through an alias, a computed access, a
        // default export, or a dynamic import is never reported.
        "AliasedDead",
        "ComputedDead",
        "DefaultedDead",
        "GapDead",
        "DynamicDead",
        "ViaAliasDead",
        "ViaComputedDead",
        "Standard.Built",
        // `declare` classes, in a `.ts` file or an ambient module block.
        "DeclaredError",
        "ModuleBlockError",
        "AmbientNamespaceError",
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

/// The rule concludes that no non-test file constructs a class, so a file that
/// fails to parse could hold the construction. `src/broken.ts` holds the only
/// `new Used()`; reporting `Used` would be a false finding, so the rule stops.
#[test]
fn a_broken_non_test_file_stops_the_rule_instead_of_reporting() {
    let root = parse_failure_fixture();
    let prefix = "rules check skipped: unconstructed-error-class: \
                  cannot prove error classes unconstructed:";
    for (config, named) in [
        // `src/__tests__/broken.ts` is also broken but is a test, and
        // `src/broken.d.ts` is a declaration file, so neither is counted:
        // `other-broken.ts` and `test-helpers/` remain.
        (
            ".no-mistakes.yml",
            "`src/broken.ts` (and 2 other files) failed to parse",
        ),
        (
            "configs/test-helpers.yml",
            "`src/broken.ts` (and 1 other file) failed to parse",
        ),
        (
            "configs/one-broken-source.yml",
            "`src/broken.ts` failed to parse",
        ),
    ] {
        let out = check(&root, config, "human");
        let body = text(&out);
        assert_eq!(out.status.code(), Some(1), "{config}: {body}");
        assert!(
            body.contains(&format!("{prefix} {named}: ")),
            "{config}: {body}"
        );
        assert!(
            lines_of(&body, "unconstructed-error-class").is_empty(),
            "{body}"
        );
        assert!(!body.contains("`Used`"), "{config}: {body}");
    }
}

/// Test files never count as construction, so a broken one cannot hide a
/// construction and does not stop the rule. `src/__tests__/` is a test by
/// default; `testFiles` classifies the other broken files. `src/broken.d.ts`
/// is broken too and is no test: a declaration file holds no construction, so
/// it never stops the rule either.
#[test]
fn broken_test_files_do_not_stop_the_rule() {
    let out = check(
        &parse_failure_fixture(),
        "configs/all-broken-are-tests.yml",
        "human",
    );
    let body = text(&out);
    assert_eq!(out.status.code(), Some(1), "{body}");
    assert!(!body.contains("failed to parse"), "{body}");
    assert_eq!(
        lines_of(&body, "unconstructed-error-class"),
        [
            "unconstructed-error-class src/used.ts:2 exported error class `Used` \
             is never constructed or subclassed in non-test source"
        ],
        "{body}"
    );
}
