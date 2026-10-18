use super::{has_disable_comment, has_disable_line_comment_tracking};

#[test]
fn markdown_disable_line_survives_an_apostrophe() {
    let earlier = "Don't pin here.\nPIN-102 <!-- no-mistakes-disable-line my-rule -->";
    let same_line = "It's PIN-102 <!-- no-mistakes-disable-line my-rule -->";
    for (source, line) in [(earlier, 2), (same_line, 1)] {
        assert!(has_disable_line_comment_tracking(
            source, line, "my-rule", false
        ));
        assert!(!has_disable_line_comment_tracking(
            source, line, "my-rule", true
        ));
    }
}

#[test]
fn markdown_disable_next_line_survives_an_apostrophe() {
    let earlier = "Don't pin here.\n<!-- no-mistakes-disable-next-line my-rule -->\nPIN-102";
    let same_line = "<!-- no-mistakes-disable-next-line my-rule: don't -->\nPIN-102";
    assert!(has_disable_comment(earlier, 3, "my-rule"));
    assert!(has_disable_comment(same_line, 2, "my-rule"));
    assert!(!has_disable_comment(earlier, 2, "my-rule"));
}

#[test]
fn comment_body_distinguishes_html_and_line_comments() {
    let html = "<!-- no-mistakes-disable-line my-rule --> tail";
    assert_eq!(
        super::comment_body(html, 0, super::CommentKind::Html),
        "no-mistakes-disable-line my-rule"
    );
    assert_eq!(
        super::comment_body(
            "<!-- no-mistakes-disable-line my-rule",
            0,
            super::CommentKind::Html
        ),
        "no-mistakes-disable-line my-rule"
    );
    let line = "code // no-mistakes-disable-line my-rule --> stays";
    let start = line.find("//").unwrap();
    assert_eq!(
        super::comment_body(line, start, super::CommentKind::line("//")),
        "no-mistakes-disable-line my-rule --> stays"
    );
}

#[test]
fn html_prefixes_that_are_not_comments_are_not_directives() {
    // An unclosed opener is not a leading comment, and a 1-byte `<` is not HTML.
    assert!(super::html_leading_comment_body("<!-- unclosed").is_none());
    assert!(!super::html_directive_comment("<div>", 0));
    assert!(super::matching_disable_directive_tracking("code", None, "my-rule", true).is_none());
}
