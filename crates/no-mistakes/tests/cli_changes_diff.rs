use no_mistakes::changes::{self, ChangesDiffArgs, ChangesReport};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[cfg(unix)]
#[path = "cli_changes_diff/filesystem.rs"]
mod filesystem;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/changes-diff")
}

fn run(base: &Path, head: &Path, config: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    command
        .args(["changes", "diff", "--base-root"])
        .arg(base)
        .arg("--head-root")
        .arg(head);
    if let Some(config) = config {
        command.arg("--config").arg(config);
    }
    command.output().expect("run changes diff")
}

fn json_report(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)))
}

fn snapshot(name: &str, report: &Value) {
    insta::assert_snapshot!(name, serde_json::to_string_pretty(report).unwrap());
}

#[test]
fn changes_diff_classifies_paths_and_unions_key_presence_across_sources() {
    let root = fixture();
    let base = root.join("base");
    let head = root.join("head");
    let output = run(&base, &head, None);
    assert!(output.status.success(), "{:?}", output.stderr);
    let report = json_report(&output);
    snapshot("changes_diff_classified", &report);
    let library = changes::diff(&ChangesDiffArgs {
        base_root: base,
        head_root: head,
        config: None,
    })
    .unwrap();
    assert_eq!(
        library,
        serde_json::from_value::<ChangesReport>(report).unwrap()
    );
}

#[test]
fn changes_diff_unreadable_root_prints_an_incomplete_report() {
    let root = fixture();
    let missing = root.join("missing");
    let output = run(&missing, &root.join("head"), None);
    assert_eq!(output.status.code(), Some(1));
    let mut report = json_report(&output);
    let normalized = report["reasons"][0]
        .as_str()
        .unwrap()
        .replace(missing.to_str().unwrap(), "<root>");
    report["reasons"][0] = json!(normalized);
    snapshot("changes_diff_unreadable_root", &report);
}

#[test]
fn changes_diff_reports_a_malformed_changed_file_and_keeps_other_results() {
    let root = fixture().join("malformed");
    let output = run(&root.join("base"), &root.join("head"), None);
    assert_eq!(output.status.code(), Some(1));
    let report = json_report(&output);
    assert_eq!(report["complete"], false);
    assert_eq!(report["inputs"]["changed"], json!(["known"]));
    let reasons = report["reasons"].as_array().unwrap();
    assert_eq!(reasons.len(), 1);
    assert!(reasons[0].as_str().unwrap().starts_with(
        "cannot diff \"changed.json\" for changes category \"inputs\": could not parse head content"
    ));
}

#[test]
fn changes_diff_reports_an_unchanged_malformed_file_when_membership_is_uncertain() {
    let root = fixture().join("ambiguous");
    let output = run(&root.join("base"), &root.join("head"), None);
    assert_eq!(output.status.code(), Some(1));
    let report = json_report(&output);
    assert_eq!(report["inputs"]["changed"], json!(["known"]));
    assert_eq!(report["inputs"]["added"], json!(["unresolved"]));
    assert_eq!(report["complete"], false);
    assert!(report["reasons"][0].as_str().unwrap().starts_with(
        "cannot diff \"malformed.json\" for changes category \"inputs\": could not parse base content"
    ));
}

#[test]
fn changes_diff_applies_an_explicit_config_relative_to_head() {
    let root = fixture();
    let output = run(
        &root.join("base"),
        &root.join("head"),
        Some(Path::new("../omit-categories.yml")),
    );
    assert!(output.status.success(), "{:?}", output.stderr);
    let report = json_report(&output);
    assert_eq!(
        report["contracts"]["changed"],
        json!(["contracts/view.txt"])
    );
    assert!(report.get("fixtures").is_none());
}

#[test]
fn changes_diff_rejects_reserved_categories_and_ambiguous_templates() {
    let root = fixture();
    for (name, message) in [
        (
            "reserved.yml",
            "changes.categories.schemaVersion: invalid changes category name \"schemaVersion\"",
        ),
        (
            "invalid-id.yml",
            "changes.categories.fixtures.id.fromPath: `fixtures/{id}/{id}.json` requires exactly one {id} placeholder",
        ),
    ] {
        let output = run(
            &root.join("base"),
            &root.join("head"),
            Some(&root.join(name)),
        );
        assert_eq!(output.status.code(), Some(2), "{name}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains(message), "{stderr}");
    }
}

#[test]
fn changes_diff_rejects_invalid_rules_even_when_the_roots_match() {
    let root = fixture();
    for (name, message) in [
        (
            "invalid-exclude.yml",
            "changes.categories.inputs.exclude[0]: contains invalid glob",
        ),
        (
            "invalid-paths.yml",
            "changes.categories.inputs.paths[0]: contains invalid glob",
        ),
        (
            "empty-glob.yml",
            "changes.categories.inputs.paths[0]: empty glob",
        ),
        (
            "missing-rule-paths.yml",
            "changes.categories.inputs.rules[0].paths must not be empty",
        ),
        (
            "category-id-without-paths.yml",
            "changes.categories.inputs.paths must not be empty",
        ),
        (
            "incomplete-key-options.yml",
            "changes.categories.inputs.id.keysOf: array and idField must be supplied together",
        ),
        (
            "conflicting-id.yml",
            "changes.categories.inputs.id: declare either fromPath or keysOf",
        ),
    ] {
        let error = changes::diff(&ChangesDiffArgs {
            base_root: root.join("base"),
            head_root: root.join("base"),
            config: Some(root.join("invalid-rules").join(name)),
        })
        .expect_err(name);
        assert!(format!("{error:#}").contains(message), "{name}: {error:#}");
    }
}

#[test]
fn changes_diff_reports_changed_paths_with_no_capture_and_preserves_valid_ids() {
    let root = fixture().join("capture-mismatch");
    let report = changes::diff(&ChangesDiffArgs {
        base_root: root.join("base"),
        head_root: root.join("head"),
        config: None,
    })
    .unwrap();
    assert!(!report.complete);
    assert_eq!(report.categories["inputs"].changed, ["matched"]);
    assert!(report.unclassified.is_empty());
    assert_eq!(report.reasons.len(), 2);
    assert!(report.reasons[0].contains("inputs/.json"));
    assert!(report.reasons[1].contains("inputs/unmatched.txt"));
}

#[test]
fn changes_diff_rejects_a_file_used_as_a_root() {
    let root = fixture();
    let report = changes::diff(&ChangesDiffArgs {
        base_root: root.join("base"),
        head_root: root.join("reserved.yml"),
        config: None,
    })
    .unwrap();
    assert!(!report.complete);
    assert!(report.categories.is_empty());
    assert!(report.unclassified.is_empty());
    assert_eq!(report.reasons.len(), 1);
    assert!(report.reasons[0].contains("not a directory"));
}

#[test]
fn changes_report_rejects_a_category_without_a_change_list() {
    let error = serde_json::from_str::<ChangesReport>(include_str!(
        "../../../test-cases/changes-diff/empty-category-report.json"
    ))
    .unwrap_err();
    assert!(error.to_string().contains("requires a change list"));
}

#[test]
fn changes_diff_scopes_excludes_per_category_so_other_categories_still_diff_keys() {
    let root = fixture().join("scoped-exclude");
    let output = run(&root.join("base"), &root.join("head"), None);
    assert!(output.status.success(), "{:?}", output.stderr);
    let report = json_report(&output);
    snapshot("changes_diff_scoped_exclude", &report);
}

#[test]
fn changes_diff_rejects_a_missing_explicit_config_when_roots_are_readable() {
    let root = fixture();
    let error = changes::diff(&ChangesDiffArgs {
        base_root: root.join("base"),
        head_root: root.join("head"),
        config: Some(PathBuf::from("missing-config.yml")),
    })
    .unwrap_err();
    assert!(
        format!("{error:#}").contains("missing-config.yml"),
        "{error:#}"
    );
}

#[test]
fn changes_diff_distinguishes_binary_bytes_from_their_lossy_text_projection() {
    let root = crate::saved_fixture::materialize_path(&fixture().join("raw-source"));
    let invalid_utf8 =
        fixture().join("../../fixtures/analysis-dataset/source-store/invalid-utf8.ts");
    std::fs::copy(invalid_utf8, root.path().join("head/raw.txt")).unwrap();
    let report = changes::diff(&ChangesDiffArgs {
        base_root: root.path().join("base"),
        head_root: root.path().join("head"),
        config: None,
    })
    .unwrap();
    assert!(report.complete, "{:?}", report.reasons);
    assert_eq!(report.categories["raw"].changed, ["raw.txt"]);
    assert!(report.unclassified.is_empty());
}
