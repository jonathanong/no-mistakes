use std::path::PathBuf;
use std::process::Command;

const STATIC: &[&str] = &[
    "dot",
    "bracket",
    "reads",
    "static-shadowed",
    "barrel",
    "member-rhs",
    "optional",
    "for-in-source",
    "for-of-source",
    "property-key",
    "default-read",
];
const UNKNOWN: &[&str] = &[
    "passed",
    "returned",
    "spread",
    "rest",
    "computed",
    "reexport",
    "reassigned",
    "mixed",
    "alias",
    "array-spread",
    "shadowed",
    "key-reference",
    "template",
    "numeric",
    "member-write",
    "member-update",
    "member-delete",
    "local-type",
    "namespace-reexport",
    "for-in-target",
    "for-of-target",
    "property-target",
    "default-target",
    "array-target",
    "rest-target",
    "unused-namespace",
    "default-export",
    "destructured",
];

fn root(case: &str) -> PathBuf {
    crate::check_json_snapshot::rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        &format!("static-namespaces/{case}"),
    )
}

fn report(case: &str) -> serde_json::Value {
    let root = root(case);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .args(["--format", "json"])
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn namespace_members_consume_exactly_the_static_exports() {
    for case in STATIC.iter().chain(["named-control"].iter()) {
        let report = report(case);
        assert!(
            report["advisories"].as_array().unwrap().is_empty(),
            "{case}: {report}"
        );
        let targets: Vec<_> = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["target"].as_str().unwrap())
            .collect();
        assert_eq!(targets, ["testOnly"], "{case}: {report}");
    }
    let report = report("unused-control");
    let targets: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["target"].as_str().unwrap())
        .collect();
    assert_eq!(targets, ["work"]);
}

#[test]
fn escaping_namespaces_keep_every_export_uncertain() {
    for case in UNKNOWN {
        let report = report(case);
        assert!(
            report["rules"].as_array().unwrap().is_empty(),
            "{case}: {report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            2,
            "{case}: {report}"
        );
    }
    let report = report("missing");
    assert!(report["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["message"]
            .as_str()
            .unwrap()
            .contains("unresolved symbol `missing`")));
}

#[test]
fn static_namespaces_check_json_snapshot() {
    let cases: Vec<_> = STATIC
        .iter()
        .chain(UNKNOWN)
        .chain(["named-control", "unused-control", "missing"].iter())
        .map(|case| (*case, root(case), root(case).join(".no-mistakes.yml")))
        .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "production_exports_static_namespaces",
        &cases,
    );
    let off: Vec<_> = cases
        .iter()
        .map(|(case, root, _)| (*case, root.clone(), root.join("off.yml")))
        .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "production_exports_static_namespaces_off",
        &off,
    );
}
