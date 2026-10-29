use std::path::PathBuf;
use std::process::Command;

const RULE: &str = "production-exports-only-used-by-tests";

fn root(case: &str) -> PathBuf {
    crate::check_json_snapshot::rules_path(RULE, "fixture", &format!("importer-gaps/{case}"))
}

fn report(case: &str) -> serde_json::Value {
    let root = root(case);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

/// Importer files carrying an advisory, in report order.
fn advised(case: &str) -> Vec<String> {
    report(case)["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["file"].as_str().unwrap().replace('\\', "/"))
        .collect()
}

fn targets(case: &str) -> Vec<String> {
    report(case)["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["target"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn accepted_shapes_keep_the_definite_finding_and_drop_the_advisory() {
    for case in [
        "ambient-modules",
        "asset-imports",
        "picked-type-imports",
        "mock-type-assertions",
        "commonjs-exports",
    ] {
        assert!(advised(case).is_empty(), "{case}: {}", report(case));
    }
    for case in ["ambient-modules", "asset-imports", "commonjs-exports"] {
        assert_eq!(targets(case), ["value"], "{case}");
    }
}

#[test]
fn created_require_is_an_ordinary_require_selector() {
    // For this rule `node:` specifiers are external, and the literal
    // repository specifier is an ordinary unknown require selector attached
    // to the module it names.
    let report = report("create-require");
    let advisories = report["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 1, "{report}");
    let message = advisories[0]["message"].as_str().unwrap();
    assert!(message.contains("tests/loader.test.mts:6"), "{report}");
    assert!(message.contains("`../src/api.mjs`"), "{report}");
    assert!(!message.contains("Shadowed"), "{report}");
}

#[test]
fn near_misses_keep_their_advisories() {
    for (case, files) in [
        (
            "ambient-modules-near-miss",
            vec![
                "src/body-export-all.d.ts",
                "src/body-export-from.d.ts",
                // The import-equals form is also its own unsupported form.
                "src/body-import-equals.d.ts",
                "src/body-import-equals.d.ts",
                "src/body-import-type.d.ts",
                "src/body-import.d.ts",
                "src/legacy.d.ts",
                "src/named.d.ts",
            ],
        ),
        (
            "asset-imports-near-miss",
            vec![
                "src/code.mts",
                "src/directory.mts",
                "src/missing.mts",
                "src/query.mts",
            ],
        ),
        (
            "picked-type-imports-near-miss",
            vec![
                "tests/absent-key.test.mts",
                "tests/bare.test.mts",
                "tests/computed-key.test.mts",
                "tests/local-pick.test.mts",
                "tests/mixed-union.test.mts",
                "tests/numeric-key.test.mts",
                "tests/omit.test.mts",
                "tests/other-generic.test.mts",
                // The local namespace declaration is its own unsupported form.
                "tests/qualified-pick.test.mts",
                "tests/qualified-pick.test.mts",
            ],
        ),
        (
            "mock-type-assertions-near-miss",
            vec![
                "tests/callback.test.mts",
                "tests/local-vi.test.mts",
                "tests/member-callee.test.mts",
                "tests/not-a-factory.test.mts",
                "tests/not-a-mock.test.mts",
                "tests/other-module.test.mts",
                "tests/plain-assertion.test.mts",
                "tests/wrong-callee.test.mts",
            ],
        ),
        (
            "commonjs-exports-near-miss",
            vec![
                "src/alias.cjs",
                "src/chained.cjs",
                "src/compound.cjs",
                "src/computed.cjs",
                "src/mixed.cjs",
                "src/mixed.cjs",
                "src/mutated.cjs",
                "src/nested.cjs",
                "src/optional.cjs",
                "src/rhs.cjs",
                "src/twice.cjs",
                "src/twice.cjs",
                "src/whole.cjs",
            ],
        ),
        (
            "create-require-near-miss",
            vec![
                "tests/alias.test.mts",
                "tests/alias.test.mts",
                "tests/computed.test.mts",
                "tests/computed.test.mts",
                "tests/destructured.test.mts",
                "tests/escape-alias.test.mts",
                "tests/escape-argument.test.mts",
                "tests/escape-call.test.mts",
                "tests/escape-comma.test.mts",
                "tests/escape-export-const.test.mts",
                "tests/escape-export-default.test.mts",
                "tests/escape-map.test.mts",
                "tests/no-base.test.mts",
                "tests/other-base.test.mts",
                "tests/reassigned.test.mts",
                "tests/unknown-source.test.mts",
            ],
        ),
    ] {
        assert_eq!(advised(case), files, "{case}: {}", report(case));
    }
}

#[test]
fn importer_gaps_check_json_snapshot() {
    let cases: Vec<_> = [
        "ambient-modules",
        "ambient-modules-near-miss",
        "asset-imports",
        "asset-imports-near-miss",
        "picked-type-imports",
        "picked-type-imports-near-miss",
        "mock-type-assertions",
        "mock-type-assertions-near-miss",
        "commonjs-exports",
        "commonjs-exports-near-miss",
        "create-require",
        "create-require-near-miss",
    ]
    .into_iter()
    .map(|case| (case, root(case), root(case).join(".no-mistakes.yml")))
    .collect();
    crate::check_json_snapshot::assert_saved_fixture("production_exports_importer_gaps", &cases);
}
