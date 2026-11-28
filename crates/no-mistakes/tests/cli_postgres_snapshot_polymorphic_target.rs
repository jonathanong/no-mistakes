use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::process::Command;

const RULE: &str = "postgres-polymorphic-target";

#[test]
fn postgres_polymorphic_target_check_json_snapshot() {
    let cases = [
        ("fail", "fail", "catalog.yml"),
        ("pass", "pass", "catalog.yml"),
        ("allow", "fail", "allow.yml"),
        ("stale_allow", "fail", "stale-allow.yml"),
        ("custom_suffix", "fail", "custom-suffix.yml"),
        ("custom_message", "fail", "custom-message.yml"),
        ("severity_advisory", "fail", "severity-advisory.yml"),
        ("project", "fail", "project.yml"),
        ("excluded", "fail", "excluded.yml"),
        ("off", "fail", "off.yml"),
        ("missing_catalog", "missing-catalog", "catalog.yml"),
        ("file_disable", "file-disable", "catalog.yml"),
        ("line_disable", "line-disable", "catalog.yml"),
        ("next_line_disable", "next-line-disable", "catalog.yml"),
        ("missing_path", "fail", "missing-path.yml"),
        ("empty_type_suffix", "fail", "empty-type-suffix.yml"),
        ("empty_id_suffix", "fail", "empty-id-suffix.yml"),
        ("same_suffix", "fail", "same-suffix.yml"),
        ("bad_reason", "fail", "bad-reason.yml"),
        ("invalid_allow", "fail", "invalid-allow.yml"),
        ("duplicate_allow", "fail", "duplicate-allow.yml"),
        ("unknown_option", "fail", "unknown-option.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(RULE, "fixture", fixture),
            rules_path(RULE, "configs", config),
        )
    });
    assert_saved_fixture("postgres_polymorphic_target", &cases);
}

#[test]
fn postgres_polymorphic_target_separates_reported_and_safe_pairs() {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(rules_path(RULE, "fixture", "fail"))
        .arg("--config")
        .arg(rules_path(RULE, "configs", "catalog.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 6, "{findings:#?}");
    let messages: Vec<_> = findings
        .iter()
        .map(|finding| finding["message"].as_str().unwrap())
        .collect();
    for expected in [
        "table:comments: columns target_type and target_id",
        "table:bookmarks: columns item_type and item_id",
        "table:notes: columns source_type and source_id",
        "table:audit_entries: columns subject_type and subject_id",
        "table:audit_entries: columns actor_type and actor_id",
        "table:events_parent: columns ref_type and ref_id",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing finding `{expected}`: {messages:#?}"
        );
    }
    for unexpected in [
        "owner_id",
        "table:attachments",
        "table:reminders",
        "table:events_leaf",
        "table:custom_links",
    ] {
        assert!(
            messages.iter().all(|message| !message.contains(unexpected)),
            "unexpected finding `{unexpected}`: {messages:#?}"
        );
    }
}
