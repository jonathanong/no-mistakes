use super::{check_in, findings_of, fixture_dir};

#[test]
fn local_append_limit_and_keyed_subject_keep_only_unbounded_controls() {
    let root = fixture_dir("issue-3115/fixture");
    assert_eq!(
        findings_of(check_in(&root, ".no-mistakes.yml")),
        [
            ("src/limit.ts".to_string(), 10, "table:accounts".to_string()),
            (
                "src/subject.ts".to_string(),
                14,
                "table:accounts".to_string()
            ),
        ]
    );
    let off = check_in(&root, "off.yml");
    assert!(off.status.success());
    let report: serde_json::Value = serde_json::from_slice(&off.stdout).unwrap();
    for field in ["rules", "advisories", "warnings"] {
        assert_eq!(report[field], serde_json::json!([]), "{report}");
    }
}

#[test]
fn local_append_limit_check_json_snapshot() {
    snapshot("limit");
}

#[test]
fn local_append_subject_check_json_snapshot() {
    snapshot("subject");
}

fn snapshot(shape: &str) {
    let root = fixture_dir("issue-3115/fixture");
    crate::check_json_snapshot::assert_saved_fixture(
        &format!("postgres_bounded_local_append_{shape}"),
        &[
            ("enabled", root.clone(), root.join(format!("{shape}.yml"))),
            ("disabled", root.clone(), root.join("off.yml")),
        ],
    );
}
