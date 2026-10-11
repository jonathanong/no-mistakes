use no_mistakes::codebase::rules::RuleFinding;
use std::cmp::Ordering;
use std::path::PathBuf;

fn baseline(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rust-api/rule-finding")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn public_rule_finding_keeps_its_six_field_literal_json_and_value_traits() {
    // This integration target compiles as a downstream library consumer.
    let finding = RuleFinding {
        rule: "postgres-no-offset".into(),
        file: "src/query.ts".into(),
        line: 7,
        message: "use cursor pagination".into(),
        import: None,
        target: Some("offset".into()),
    };
    let expected: serde_json::Value = serde_json::from_str(&baseline("expected.json")).unwrap();
    assert_eq!(serde_json::to_value(&finding).unwrap(), expected);
    assert_eq!(
        format!("{finding:?}"),
        baseline("expected-debug.txt").trim()
    );
    let same = finding.clone();
    assert_eq!(finding, same);
    assert_eq!(finding.cmp(&same), Ordering::Equal);
    assert_eq!(finding.partial_cmp(&same), Some(Ordering::Equal));
    let mut later = same;
    later.line += 1;
    assert_eq!(finding.cmp(&later), Ordering::Less);
    assert_ne!(finding, later);
}
