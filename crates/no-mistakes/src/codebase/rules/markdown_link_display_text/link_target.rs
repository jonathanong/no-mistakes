use super::*;

pub(super) fn finding_for_link(
    file: &str,
    source: &str,
    link: InlineLink,
    extensions: &[&str],
) -> Option<RuleFinding> {
    let text = markdown_unescape(link.text).replace('`', "");
    if !looks_like_md_filename(&text, extensions) || is_non_local_href(&link.href) {
        return None;
    }
    let basename = href_basename(&link.href)?;
    if basename == text {
        return None;
    }
    Some(RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: byte_offset_to_line(source, link.offset) as usize,
        message: format!(
            "{file}: link text \"{text}\" does not match target basename \"{basename}\""
        ),
        import: Some(text),
        target: Some(basename),
    })
}

fn looks_like_md_filename(text: &str, extensions: &[&str]) -> bool {
    extensions.iter().any(|extension| text.ends_with(extension))
        && !text.is_empty()
        && !text
            .chars()
            .any(|ch| ch == '/' || ch == '\\' || ch.is_whitespace())
}

pub(super) fn href_basename(href: &str) -> Option<String> {
    let bare = href_destination(href);
    let before_fragment = bare.split('#').next().unwrap_or_default();
    let before_query = before_fragment.split('?').next().unwrap_or_default();
    if before_query.ends_with('/') {
        return None;
    }
    before_query
        .rsplit('/')
        .next()
        .filter(|basename| !basename.is_empty())
        .map(percent_decode)
        .map(markdown_unescape)
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if let Some(decoded) = decode_hex_byte(bytes.get(index + 1), bytes.get(index + 2)) {
                out.push(decoded);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| value.to_string())
}

fn markdown_unescape(value: String) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                if next.is_ascii_punctuation() {
                    out.push(next);
                } else {
                    out.push(ch);
                    out.push(next);
                }
            } else {
                out.push(ch);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn decode_hex_byte(high: Option<&u8>, low: Option<&u8>) -> Option<u8> {
    Some(hex_value(*high?)? * 16 + hex_value(*low?)?)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn is_non_local_href(href: &str) -> bool {
    let bare = href_destination(href);
    bare.starts_with('#') || bare.starts_with("//") || has_url_scheme(bare)
}

fn has_url_scheme(value: &str) -> bool {
    let Some(colon) = value.find(':') else {
        return false;
    };
    let scheme = &value[..colon];
    !scheme.is_empty()
        && scheme
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
        && scheme
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic())
}

pub(super) fn href_destination(value: &str) -> &str {
    let trimmed = value.trim();
    if let Some(rest) = trimmed.strip_prefix('<') {
        if let Some(end) = rest.find('>') {
            &rest[..end]
        } else {
            trimmed
        }
    } else {
        trimmed.split_ascii_whitespace().next().unwrap_or_default()
    }
}
