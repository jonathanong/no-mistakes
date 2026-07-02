#[derive(Debug, PartialEq, Eq)]
pub(super) struct InlineLink {
    pub(super) text: String,
    pub(super) href: String,
    pub(super) offset: usize,
}

pub(super) fn inline_links_outside_code(source: &str) -> Vec<InlineLink> {
    let fenced = strip_fenced_code(source);
    let bytes = fenced.as_bytes();
    let mut links = Vec::new();
    let mut index = 0usize;
    let mut in_inline_code = false;
    while index < bytes.len() {
        match bytes[index] {
            b'`' => {
                in_inline_code = !in_inline_code;
                index += 1;
            }
            b'[' if !in_inline_code && (index == 0 || bytes[index - 1] != b'!') => {
                if let Some((link, next)) = parse_inline_link(&fenced, index) {
                    links.push(link);
                    index = next;
                } else {
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }
    links
}

pub(super) fn parse_inline_link(source: &str, start: usize) -> Option<(InlineLink, usize)> {
    let bytes = source.as_bytes();
    let text_end = find_byte(bytes, start + 1, b']')?;
    if bytes.get(text_end + 1) != Some(&b'(') {
        return None;
    }
    let href_start = text_end + 2;
    let href_end = find_byte(bytes, href_start, b')')?;
    Some((
        InlineLink {
            text: source[start + 1..text_end].to_string(),
            href: source[href_start..href_end].to_string(),
            offset: start,
        },
        href_end + 1,
    ))
}

fn find_byte(bytes: &[u8], start: usize, target: u8) -> Option<usize> {
    bytes
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, byte)| (*byte == target).then_some(index))
}

fn strip_fenced_code(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_fence: Option<&str> = None;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start_matches([' ', '\t']);
        let indent = line.len() - trimmed.len();
        let marker = if indent <= 3 && trimmed.starts_with("```") {
            Some("```")
        } else if indent <= 3 && trimmed.starts_with("~~~") {
            Some("~~~")
        } else {
            None
        };
        match (in_fence, marker) {
            (None, Some(marker)) => {
                in_fence = Some(marker);
                out.push_str(&"\n".repeat(line.matches('\n').count()));
            }
            (Some(active), Some(marker)) if active == marker => {
                in_fence = None;
                out.push_str(&"\n".repeat(line.matches('\n').count()));
            }
            (Some(_), _) => {
                out.push_str(&"\n".repeat(line.matches('\n').count()));
            }
            (None, _) => out.push_str(line),
        }
    }
    out
}
