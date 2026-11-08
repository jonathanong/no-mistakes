use std::process::Command;

const STATIC: &[&str] = &[
    "loader-read",
    "then-bracket",
    "all-hole",
    "loader",
    "loader-multi",
    "then-call",
    "then-block",
    "then-default",
    "all",
    "all-pair",
];
const UNKNOWN: &[&str] = &[
    "loader-reference",
    "loader-optional-member",
    "loader-unselected",
    "all-direct-call",
    "all-receiver-call",
    "all-detached-await",
    "loader-alias",
    "loader-spread",
    "loader-duplicate",
    "loader-optional",
    "loader-args",
    "loader-let",
    "loader-unused",
    "loader-result-write",
    "loader-result-mutation",
    "loader-result-this",
    "loader-params",
    "loader-method",
    "loader-computed-key",
    "loader-async",
    "loader-no-await",
    "loader-callback",
    "loader-pattern",
    "loader-array",
    "loader-nested",
    "loader-block",
    "then-alias",
    "then-reassign",
    "then-delete",
    "then-return",
    "then-unused",
    "then-call-escape",
    "all-short",
    "all-namespace",
    "all-outer-rest",
    "all-stored",
    "all-no-await",
    "all-other",
    "all-extra-arg",
    "all-optional",
    "all-literal",
    "loader-escape",
    "loader-computed",
    "loader-write",
    "loader-mixed",
    "loader-export",
    "loader-this",
    "then-escape",
    "then-this",
    "then-write",
    "then-shadow",
    "then-computed",
    "all-rest",
    "all-computed",
    "all-spread",
    "all-shadow",
];

fn root(case: &str) -> std::path::PathBuf {
    crate::check_json_snapshot::rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        &format!("dynamic-selectors/{case}"),
    )
}

#[test]
fn dynamic_selectors_prove_members_and_preserve_escapes() {
    for case in STATIC.iter().chain(UNKNOWN).chain(["named-control"].iter()) {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(root(case))
            .args(["--format", "json"])
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        if UNKNOWN.contains(case) {
            assert!(
                report["rules"].as_array().unwrap().is_empty(),
                "{case}: {report}"
            );
            assert_eq!(
                report["advisories"].as_array().unwrap().len(),
                3,
                "{case}: {report}"
            );
        } else {
            assert!(
                report["advisories"].as_array().unwrap().is_empty(),
                "{case}: {report}"
            );
            let mut targets: Vec<_> = report["rules"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["target"].as_str().unwrap())
                .collect();
            targets.sort_unstable();
            let expected = if case.starts_with("loader") {
                vec!["testOnly", "work"]
            } else if *case == "named-control" {
                vec!["testOnly"]
            } else {
                vec!["main", "testOnly"]
            };
            assert_eq!(targets, expected, "{case}: {report}");
        }
    }
}

#[test]
fn dynamic_selectors_check_json_snapshot() {
    for (snapshot, config) in [
        ("production_exports_dynamic_selectors", ".no-mistakes.yml"),
        ("production_exports_dynamic_selectors_off", "off.yml"),
    ] {
        let cases: Vec<_> = STATIC
            .iter()
            .chain(UNKNOWN)
            .chain(["named-control"].iter())
            .map(|case| (*case, root(case), root(case).join(config)))
            .collect();
        crate::check_json_snapshot::assert_saved_fixture(snapshot, &cases);
    }
}
