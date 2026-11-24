use super::{diff_json, plan, selected_files, summary};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/tests-plan-changes-attributes")
        .join(name)
        .join("fixture")
}

fn saved_report(fixture: &Path, name: &str) -> Value {
    let path = fixture.join("reports").join(name);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn changed_fixture_attributes_select_only_the_readers_of_changed_fields() {
    let fixture = fixture("narrow");
    let head = fixture.join("head");
    let diff = diff_json(&fixture.join("base"), &head);
    assert_eq!(diff, saved_report(&fixture, "size.json"));
    let args = [
        "--changes",
        "../reports/size.json",
        "--changed-file",
        "fixtures/profile.avatar.json",
    ];
    let report = plan(&head, "1", &args);
    assert_eq!(report, plan(&head, "2", &args));
    assert_eq!(report["fallback_triggered"], false);
    assert_eq!(selected_files(&report), ["tests/size.test.ts"]);
    insta::assert_snapshot!(
        "fixture_attribute_routing",
        serde_json::to_string_pretty(&json!({ "diff": diff, "plan": summary(&report) })).unwrap()
    );
}

#[test]
fn fixture_attributes_join_source_impact_using_prepared_graph_facts() {
    let head = fixture("narrow").join("head");
    let args = [
        "--changes",
        "../reports/size.json",
        "--changed-file",
        "src/label.ts",
    ];
    let report = plan(&head, "1", &args);
    assert_eq!(report["fallback_triggered"], false);
    assert_eq!(
        selected_files(&report),
        ["tests/size.test.ts", "tests/unrelated.test.ts"]
    );
}

#[test]
fn changed_fixture_attributes_overlap_ancestors_and_descendants_of_static_reads() {
    let head = fixture("narrow").join("head");
    let args = ["--changes", "../reports/encoding-and-alt.json"];
    let report = plan(&head, "1", &args);
    assert_eq!(report["fallback_triggered"], false);
    assert_eq!(
        selected_files(&report),
        ["tests/alt.test.ts", "tests/encoding.test.ts"]
    );

    let report = plan(&head, "1", &["--changes", "../reports/unused-field.json"]);
    assert_eq!(report["fallback_triggered"], false);
    assert!(selected_files(&report).is_empty());
}

#[test]
fn fixture_attributes_preserve_whole_fixture_routing_when_diff_or_read_is_opaque() {
    let fixture = fixture("fallback");
    let head = fixture.join("head");
    let diff = diff_json(&fixture.join("base"), &head);
    assert_eq!(diff, saved_report(&fixture, "changed.json"));
    let args = ["--changes", "../reports/changed.json"];
    let report = plan(&head, "1", &args);
    assert_eq!(report, plan(&head, "2", &args));
    assert_eq!(report["fallback_triggered"], false);
    assert_eq!(
        selected_files(&report),
        [
            "tests/invalid.test.ts",
            "tests/iteration.test.ts",
            "tests/raw.test.ts",
            "tests/replaced.test.ts",
            "tests/spread.test.ts",
            "tests/unknown-call.test.ts",
        ]
    );
    insta::assert_snapshot!(
        "fixture_attribute_fallback_routing",
        serde_json::to_string_pretty(&json!({ "diff": diff, "plan": summary(&report) })).unwrap()
    );
}

#[test]
fn fixture_attributes_preserve_legacy_and_unmatched_identifier_routing() {
    let head = fixture("narrow").join("head");
    let report = plan(&head, "1", &["--changes", "../reports/legacy.json"]);
    assert_eq!(report["fallback_triggered"], false);
    assert_eq!(
        selected_files(&report),
        [
            "tests/alt.test.ts",
            "tests/encoding.test.ts",
            "tests/size.test.ts",
        ]
    );

    let args = [
        "--changes",
        "../reports/unmatched.json",
        "--limit-files",
        "1",
    ];
    let report = plan(&head, "1", &args);
    assert_eq!(report["fallback_triggered"], true);
    assert_eq!(report["warnings"][0]["type"], "changes-fallback");
    assert!(report["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("Unmatched IDs"));
    assert_eq!(
        selected_files(&report),
        [
            "tests/alt.test.ts",
            "tests/encoding.test.ts",
            "tests/size.test.ts",
            "tests/unrelated.test.ts",
        ]
    );
}
