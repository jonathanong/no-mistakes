use super::*;

#[test]
fn encodes_apostrophes_spaces_and_backslashes() {
    assert_eq!(
        js_string_literal("/Volumes/Jongleberry's T7/root"),
        r#""/Volumes/Jongleberry's T7/root""#
    );
    assert_eq!(js_string_literal(r"C:\temp\app"), r#""C:\\temp\\app""#);
    assert_eq!(
        replace_quoted_placeholder(
            "extends: '__ABS__'",
            "__ABS__",
            "/Volumes/Jongleberry's T7/base.js"
        ),
        r#"extends: "/Volumes/Jongleberry's T7/base.js""#
    );
}
