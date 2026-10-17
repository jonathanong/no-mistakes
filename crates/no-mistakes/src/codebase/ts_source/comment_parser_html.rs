fn html_leading_comment_body(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("<!--")?;
    let end = rest.find("-->")?;
    rest[end + 3..]
        .trim()
        .is_empty()
        .then_some(rest[..end].trim())
}

fn html_directive_comment(line: &str, idx: usize) -> bool {
    let Some(rest) = line[idx..].strip_prefix("<!--") else {
        return false;
    };
    if !comment_prefix_can_start(line, idx) {
        return false;
    }
    let Some(end) = rest.find("-->") else {
        return false;
    };
    rest[..end].trim().starts_with("no-mistakes-disable-")
}

fn comment_body(line: &str, start: usize, prefix_len: usize) -> &str {
    let rest = line[start + prefix_len..].trim();
    if prefix_len == 4 {
        let end = rest.find("-->").unwrap_or(rest.len());
        return rest[..end].trim();
    }
    rest
}
