use regex::bytes::Regex;
use std::sync::LazyLock;

static BARE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?-u)^/(?:\\[^\r\n]|[^\s\\/])",
        r"(?:(?:\\[^\r\n]|[^\\/\r\n])*(?:\\[^\r\n]|[^\s\\/]))?/",
    ))
    .expect("Swift single-line bare regex")
});

pub(super) fn end(source: &[u8], start: usize) -> Option<usize> {
    if source[start] != b'/'
        || !matches!(
            source[..start].last(),
            None | Some(
                b' ' | b'\t' | b'\n' | b'\r' | b'=' | b'(' | b'[' | b'{' | b',' | b':' | b';'
            )
        )
    {
        return None;
    }
    BARE_REGEX
        .find(&source[start..])
        .map(|matched| start + matched.end())
}
