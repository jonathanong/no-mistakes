use super::*;

#[test]
fn dedup_occurrences_preserves_distinct_lines() {
    let mut occurrences = vec![occurrence(9), occurrence(3), occurrence(3)];

    dedup_occurrences_by_identity(&mut occurrences);

    assert_eq!(
        occurrences
            .iter()
            .map(|occurrence| occurrence.line)
            .collect::<Vec<_>>(),
        vec![3, 9]
    );
}

#[test]
fn occurrence_scope_runnable_only_for_hooks_and_tests() {
    assert!(!std::hint::black_box(TestOccurrenceScope::File).is_runnable());
    assert!(std::hint::black_box(TestOccurrenceScope::Hook).is_runnable());
    assert!(!std::hint::black_box(TestOccurrenceScope::TeardownHook).is_runnable());
    assert!(std::hint::black_box(TestOccurrenceScope::Test).is_runnable());
}

fn occurrence(line: u32) -> TestOccurrence<&'static str> {
    TestOccurrence {
        value: "selector",
        status: TestStatus::Active,
        scope: TestOccurrenceScope::Test,
        test_name: Some("visits home".to_string()),
        describe_path: Vec::new(),
        line,
    }
}
