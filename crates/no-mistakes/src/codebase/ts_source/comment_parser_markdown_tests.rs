use super::has_disable_line_comment_tracking;

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
