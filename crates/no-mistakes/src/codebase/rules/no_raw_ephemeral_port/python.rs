pub(super) const BIND_PATTERN: &str = concat!(
    r#"\.\s*bind\s*\(\s*\(\s*"#,
    r#"[^,]+"#,
    r#"\s*,\s*"#,
    r#"(?:0(?:_?0)*|0[xXoObB](?:_?0)+)"#,
    r#"(?:\s*,[^)]*)?\s*\)\s*\)"#,
);

pub(super) fn scan_lines(source: &str, bind: &regex::Regex) -> Vec<usize> {
    bind.find_iter(source)
        .filter_map(|matched| {
            if line_is_comment(source, matched.start()) {
                return None;
            }
            Some(
                source[..matched.start()]
                    .bytes()
                    .filter(|&b| b == b'\n')
                    .count()
                    + 1,
            )
        })
        .collect()
}

fn line_is_comment(source: &str, offset: usize) -> bool {
    let start = source[..offset].rfind('\n').map_or(0, |i| i + 1);
    let end = source[offset..]
        .find('\n')
        .map_or(source.len(), |i| offset + i);
    let trimmed = source[start..end].trim_start();
    trimmed.starts_with('#') || trimmed.starts_with("//")
}
