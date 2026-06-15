use super::*;

#[test]
fn invalid_glob_does_not_match() {
    assert!(!glob_matches("[", "src/a.ts"));
}

#[test]
fn star_does_not_cross_slash() {
    assert!(glob_matches("src/*.ts", "src/a.ts"));
    assert!(!glob_matches("src/*.ts", "src/nested/a.ts"));
    assert!(glob_matches("src/**/*.ts", "src/nested/a.ts"));
}

#[test]
fn negation_last_match_wins() {
    let patterns = vec!["src/**".to_string(), "!src/docs/**".to_string()];
    assert!(selected_by(&patterns, "src/a.ts"));
    assert!(!selected_by(&patterns, "src/docs/x.md"));
}
