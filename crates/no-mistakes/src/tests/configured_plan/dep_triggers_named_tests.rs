use super::{compile_ordered_patterns, matches_ordered};

#[test]
fn ordered_patterns_trim_before_detecting_negation() {
    let patterns =
        compile_ordered_patterns(&["src/**".to_string(), " !./src/generated/**".to_string()])
            .unwrap();
    assert!(matches_ordered(&patterns, "src/keep.ts"));
    assert!(!matches_ordered(&patterns, "src/generated/a.ts"));
}
