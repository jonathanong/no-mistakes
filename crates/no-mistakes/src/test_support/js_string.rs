/// Encode `value` as a JavaScript/JSON string literal, including quotes.
pub(crate) fn js_string_literal(value: impl AsRef<str>) -> String {
    serde_json::to_string(value.as_ref()).expect("serializing a string never fails")
}

/// Replace a single-quoted `'placeholder'` in generated source with an
/// encoded JavaScript string literal for `value`.
pub(crate) fn replace_quoted_placeholder(
    source: &str,
    placeholder: &str,
    value: impl AsRef<str>,
) -> String {
    source.replace(
        &format!("'{placeholder}'"),
        &js_string_literal(value.as_ref()),
    )
}

#[cfg(test)]
mod tests {
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
}
