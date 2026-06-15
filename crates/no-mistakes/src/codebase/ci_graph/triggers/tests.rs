use super::*;

fn compiled(patterns: &[&str]) -> Vec<CompiledGlob> {
    compile_list(&patterns.iter().map(|p| p.to_string()).collect::<Vec<_>>())
}

#[test]
fn invalid_glob_is_dropped() {
    assert!(compiled(&["["]).is_empty());
    assert!(!selected_by(&compiled(&["["]), "src/a.ts"));
}

#[test]
fn star_does_not_cross_slash() {
    assert!(selected_by(&compiled(&["src/*.ts"]), "src/a.ts"));
    assert!(!selected_by(&compiled(&["src/*.ts"]), "src/nested/a.ts"));
    assert!(selected_by(&compiled(&["src/**/*.ts"]), "src/nested/a.ts"));
}

#[test]
fn negation_last_match_wins() {
    let globs = compiled(&["src/**", "!src/docs/**"]);
    assert!(selected_by(&globs, "src/a.ts"));
    assert!(!selected_by(&globs, "src/docs/x.md"));
    // Only the positive pattern is reported as the matching filter.
    assert_eq!(
        matching_patterns(&globs, "src/a.ts"),
        vec!["src/**".to_string()]
    );
}
