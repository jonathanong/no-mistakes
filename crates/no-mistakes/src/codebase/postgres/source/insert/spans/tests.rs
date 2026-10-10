use super::source_matches_rendered;

#[test]
fn keeps_trivia_and_rejects_a_different_token() {
    assert!(source_matches_rendered("now /*keep*/ ()", "now()"));
    assert!(source_matches_rendered("now -- keep\n()", "now()"));
    assert!(source_matches_rendered(
        "now /* outer /* inner */ */ ()",
        "now()"
    ));
    assert!(source_matches_rendered("a /*c*/ / b", "a / b"));
    assert!(source_matches_rendered("now()", "now()"));
    assert!(!source_matches_rendered("integer", "INTEGER"));
    assert!(!source_matches_rendered("now", "now()"));
    assert!(!source_matches_rendered("now /*", "now()"));
}
