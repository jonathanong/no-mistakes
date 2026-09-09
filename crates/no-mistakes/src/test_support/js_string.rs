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

/// Temp directory whose path contains an apostrophe and a space, matching the
/// external-drive worktree that interpolating raw paths into JS literals breaks.
pub(crate) fn quote_containing_tempdir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("Jongleberry's T7-")
        .tempdir()
        .expect("create quote-containing tempdir")
}

pub(crate) fn materialize_quote_containing_fixture(source: &std::path::Path) -> tempfile::TempDir {
    super::gitignore_fixture::materialize_saved_fixture_into(source, quote_containing_tempdir())
}

#[cfg(test)]
mod tests;
