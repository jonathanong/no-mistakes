use crate::codebase::rules::run_filesystem_rules;

fn findings(name: &str) -> Vec<String> {
    let root = super::fixture(&format!("concurrency-intent/{name}"));
    run_filesystem_rules(&root, Some(&root.join(".no-mistakes.yml")))
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

#[test]
fn concurrency_intent_fixture_passes_when_locks_match_policy() {
    let found = findings("pass");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn concurrency_intent_fixture_reports_each_mismatch() {
    let first = findings("fail");
    let second = findings("fail");
    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap()
    );
    assert_eq!(
        first,
        vec![
            "concurrency cancellation mismatch: .github/workflows/ci.yml: expected conditional, got cancel-running".to_string(),
            "concurrency intent missing: .github/workflows/lint.yml".to_string(),
            "concurrency pending mismatch: .github/workflows/release.yml#publish: expected fifo, got coalesce-latest".to_string(),
            "concurrency scope mismatch: .github/workflows/deploy.yml: expected , got sha".to_string(),
        ]
    );
}
