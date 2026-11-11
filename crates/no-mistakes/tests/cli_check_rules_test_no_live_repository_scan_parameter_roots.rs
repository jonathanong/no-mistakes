use std::path::PathBuf;

const RULE: &str = "test-no-live-repository-scan";
const CASES: [&str; 28] = [
    "derived-join",
    "helper-return",
    "defaulted-param",
    "join-call",
    "options-map",
    "omitted-default",
    "nested-member",
    "outside-cjs",
    "imported-cjs",
    "setup-spread",
    "sibling-const",
    "import-alias",
    "reaching-alias",
    "recursive-helper",
    "path-only-cjs",
    "path-and-unread",
    "many-blockers",
    "unmatched-config",
    "unresolved-setups",
    "barrel-reexport",
    "captured-root",
    "captured-root-caller",
    "captured-root-owned-caller",
    "deep-unreadable",
    "wildcard-export",
    "readable-package",
    "meta-main",
    "caller-argument",
];

fn root(name: &str) -> PathBuf {
    crate::check_json_snapshot::rules_path(RULE, "parameter-roots", name)
}

#[test]
fn parameter_roots_check_json_snapshot() {
    let cases: Vec<_> = CASES
        .into_iter()
        .map(|name| {
            let root = root(name);
            (name, root.clone(), root.join(".no-mistakes.yml"))
        })
        .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_parameter_roots",
        &cases,
    );
}

#[test]
fn caller_references_check_json_snapshot() {
    let cases: Vec<_> = [
        "star-owned",
        "star-live",
        "named-owned",
        "named-live",
        "erased-owned",
        "erased-live",
        "dynamic-owned",
        "selected-root",
        "env-selected-escape",
        "env-selected-sibling-owned",
        "dynamic-reexport-owned",
        "dynamic-reexport-live",
        "void-owned",
        "void-live",
        "dynamic-live",
        "guarded-owned",
        "guarded-bare-owned",
        "guarded-bare-live",
        "void-import-owned",
        "void-import-live",
        "guarded-live",
        "constant-owned",
        "missing-export",
        "receiver-value",
        "constant-live",
        "namespace-escape",
        "namespace-computed",
        "promise-escape",
        "stored-promise-owned",
        "stored-promise-live",
        "stored-promise-escape",
        "exported-namespace",
        "receiver-call",
        "runtime-reference",
        "void-live-call",
        "object-reference",
        "namespace-reexport",
    ]
    .into_iter()
    .map(|name| {
        let root = crate::check_json_snapshot::rules_path(RULE, "caller-references", name);
        (name, root.clone(), root.join(".no-mistakes.yml"))
    })
    .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_caller_references",
        &cases,
    );
}

#[test]
fn caller_reference_owned_roots_clear_and_recall_controls_report() {
    let base = crate::check_json_snapshot::rules_path(RULE, "caller-references", "");
    for entry in std::fs::read_dir(base).unwrap() {
        let root = entry.unwrap().path();
        let name = root.file_name().unwrap().to_str().unwrap();
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--format", "json", "--root"])
            .arg(&root)
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let count = report["rules"].as_array().unwrap().len()
            + report["advisories"].as_array().unwrap().len();
        if name.ends_with("-owned") {
            assert_eq!(count, 0, "{name}: {report}");
        } else {
            assert!(count > 0, "{name}: {report}");
        }
    }
}

const DYNAMIC_ORIGINS: [(&str, usize); 10] = [
    ("production-only", 0),
    ("test-reachable", 1),
    ("production-live", 1),
    ("production-unconditional", 0),
    ("production-opaque", 0),
    ("test-opaque", 1),
    ("test-dynamic-escape", 1),
    ("setup-dynamic-escape", 1),
    ("global-setup-dynamic-escape", 1),
    ("config-dynamic-escape", 1),
];

#[test]
fn dynamic_origins_check_json_snapshot() {
    let cases: Vec<_> = DYNAMIC_ORIGINS
        .into_iter()
        .map(|(name, _)| {
            let root = crate::check_json_snapshot::rules_path(RULE, "dynamic-origins", name);
            (name, root.clone(), root.join(".no-mistakes.yml"))
        })
        .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_dynamic_origins",
        &cases,
    );
}

#[test]
fn only_test_originating_dynamic_importers_block_owned_callers() {
    for (name, expected) in DYNAMIC_ORIGINS {
        let root = crate::check_json_snapshot::rules_path(RULE, "dynamic-origins", name);
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--format", "json", "--root"])
            .arg(root)
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let count = report["rules"].as_array().unwrap().len()
            + report["advisories"].as_array().unwrap().len();
        assert_eq!(count, expected, "{name}: {report}");
    }
}
