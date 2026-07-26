/// Extract link targets from Markdown source using pulldown-cmark.
///
/// Returns raw link URL strings. Callers are responsible for filtering
/// external URLs and resolving relative paths to absolute file paths.
pub fn extract_links(source: &str) -> Vec<String> {
    use pulldown_cmark::{Event, Options, Parser, Tag};

    let mut links = Vec::new();

    for event in Parser::new_ext(source, Options::all()) {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                links.push(dest_url.into_string());
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                links.push(dest_url.into_string());
            }
            _ => {}
        }
    }

    links
}

/// Returns true if the URL is an external link that should not be resolved
/// to a local file path.
pub fn is_external(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("mailto:")
        || url.starts_with("//")
        || url.starts_with('#')
        || url.starts_with('?')
}

/// Decode a repository-local URL path while rejecting encoded separators.
pub fn decode_local_path(url: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(url.len());
    let raw = url.as_bytes();
    let mut index = 0;
    while index < raw.len() {
        if raw[index] != b'%' {
            bytes.push(raw[index]);
            index += 1;
            continue;
        }
        let high = hex(*raw.get(index + 1)?)?;
        let low = hex(*raw.get(index + 2)?)?;
        let decoded = (high << 4) | low;
        if matches!(decoded, b'/' | b'\\') {
            return None;
        }
        bytes.push(decoded);
        index += 3;
    }
    String::from_utf8(bytes).ok()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
