use super::source_matches_rendered;

#[test]
fn keeps_trivia_and_rejects_a_different_token() {
    assert!(source_matches_rendered("now /*keep*/ ()", "now()"));
    assert!(source_matches_rendered("now -- keep\n()", "now()"));
    assert!(source_matches_rendered("-- keep", ""));
    assert!(source_matches_rendered(
        "now /* outer /* inner */ */ ()",
        "now()"
    ));
    assert!(source_matches_rendered("a /*c*/ / b", "a / b"));
    assert!(source_matches_rendered("now()", "now()"));
    assert!(!source_matches_rendered("integer", "INTEGER"));
    assert!(!source_matches_rendered("now", "now()"));
    assert!(!source_matches_rendered("now /*", "now()"));
    assert!(!source_matches_rendered("/*", ""));
}

#[test]
fn quoted_comment_markers_stay_literal() {
    assert!(source_matches_rendered("'--'", "'--'"));
    assert!(source_matches_rendered("'/*x*/'", "'/*x*/'"));
    assert!(source_matches_rendered(r#""a--b""#, r#""a--b""#));
    assert!(source_matches_rendered("'--' /*keep*/", "'--'"));
    assert!(source_matches_rendered("'$--'--x", "'$--'"));
    assert!(source_matches_rendered("'/*x*/' /*keep*/", "'/*x*/'"));
    assert!(source_matches_rendered("a'--'", "a'--'"));
    assert!(source_matches_rendered("'--", "'--"));
    // Two-, three-, and four-byte characters stay literal outside quotes.
    assert!(source_matches_rendered("é /*c*/ x", "é x"));
    assert!(source_matches_rendered("日$1", "日$1"));
    assert!(source_matches_rendered("😀$1", "😀$1"));
    assert!(source_matches_rendered("'é--'", "'é--'"));
    assert!(source_matches_rendered("E'--'", "E'--'"));
    assert!(source_matches_rendered(r"E'\''--'", r"E'\''"));
    // A trailing backslash must not read past the literal.
    assert!(source_matches_rendered(r"E'\", r"E'\"));
    assert!(source_matches_rendered(r"xE'--'", r"xE'--'"));
    assert!(source_matches_rendered("$$--$$", "$$--$$"));
    assert!(source_matches_rendered("$$--$$ /*keep*/", "$$--$$"));
    assert!(source_matches_rendered(
        "$tag$/*x*/$tag$",
        "$tag$/*x*/$tag$"
    ));
    // Unicode tags are delimiters. `--` inside stays literal; a later `--` does not.
    assert!(source_matches_rendered("$é$--$é$", "$é$--$é$"));
    assert!(source_matches_rendered("$é$--$é$--tail", "$é$--$é$"));
    assert!(source_matches_rendered("$tag$--$tag$", "$tag$--$tag$"));
    assert!(source_matches_rendered("$a_b$--$a_b$", "$a_b$--$a_b$"));
    assert!(source_matches_rendered("$$--", "$$--"));
    assert!(source_matches_rendered("$1", "$1"));
    assert!(source_matches_rendered("a$$b", "a$$b"));
    // Letters and `_` keep `$` in the identifier. A symbol can open a dollar quote.
    assert!(source_matches_rendered("_$1", "_$1"));
    assert!(source_matches_rendered("é$1", "é$1"));
    assert!(source_matches_rendered("+$$--$$", "+$$--$$"));
    assert!(source_matches_rendered("$$", "$$"));
    assert!(!source_matches_rendered("'--'", "'xx'"));
    assert!(!source_matches_rendered("$$--$$", "$$xx$$"));
}
