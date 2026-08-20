pub(super) const BIND_PATTERN: &str =
    r#"\.\s*bind\s*\(\s*\(\s*(?:"(?:\\.|[^"\\\r\n])*"|'(?:\\.|[^'\\\r\n])*')\s*,\s*0\s*\)\s*\)"#;

pub(super) fn scan_lines(source: &str, bind: &regex::Regex) -> Vec<usize> {
    bind.find_iter(source)
        .map(|matched| {
            source[..matched.start()]
                .bytes()
                .filter(|&b| b == b'\n')
                .count()
                + 1
        })
        .collect()
}
