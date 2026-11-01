use super::{check_in, findings_of, fixture_dir};

fn lines_of(config: &str) -> Vec<(String, u64)> {
    let root = fixture_dir("variants/shared-fragment");
    let mut found: Vec<_> = findings_of(check_in(&root, config))
        .into_iter()
        .map(|(file, line, _)| (file, line))
        .collect();
    found.sort();
    found
}

#[test]
fn statements_appending_a_shared_fragment_are_reported_at_each_caller() {
    assert_eq!(
        lines_of(".no-mistakes.yml"),
        [
            ("src/first.ts".to_string(), 5),
            ("src/second.ts".to_string(), 5),
            ("src/third.ts".to_string(), 7)
        ]
    );
}

#[test]
fn a_caller_directive_suppresses_only_that_callers_statement() {
    let found = lines_of(".no-mistakes.yml");
    assert!(found.iter().all(|(file, _)| file != "src/suppressed.ts"));
    assert!(found.iter().all(|(file, _)| file != "src/fragment.ts"));
}

#[test]
fn shared_fragment_check_json_snapshot() {
    let root = fixture_dir("variants/shared-fragment");
    crate::check_json_snapshot::assert_saved_fixture(
        "postgres_bounded_shared_fragment",
        &[("callers", root.clone(), root.join(".no-mistakes.yml"))],
    );
}
