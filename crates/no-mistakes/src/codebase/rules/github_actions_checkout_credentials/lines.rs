pub(super) fn checkout_use_lines(source: &str) -> Vec<usize> {
    let lines: Vec<&str> = source.lines().collect();
    let mut frames: Vec<(usize, Kind)> = Vec::new();
    let mut found = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if let Some(&(indent, Kind::Block)) = frames.last() {
            if block_continues(lines[index], indent) {
                index += 1;
                continue;
            }
            frames.pop();
            continue;
        }
        if let Some(parsed) = parse_line(lines[index]) {
            while frames
                .last()
                .is_some_and(|(indent, _)| *indent >= parsed.scope)
            {
                frames.pop();
            }
            if parsed.is_item && parent(&frames) == Some(Kind::Steps) {
                frames.push((parsed.scope, Kind::Step));
            }
            if let Some(key) = parsed.key.as_deref() {
                note_uses(key, &parsed, &frames, &mut found, index + 1);
                push_frame(&parsed, key, &mut frames);
            }
        }
        index += 1;
    }
    found
}

pub(super) fn is_checkout_uses(uses: &str) -> bool {
    let uses = strip_comment(uses).trim();
    let uses = unquote(uses).trim();
    let name = match uses.split_once('@') {
        Some((name, _)) => name,
        None => uses,
    };
    name.trim_end_matches('/') == "actions/checkout"
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Jobs,
    Job,
    Steps,
    Step,
    Block,
}

struct Parsed {
    scope: usize,
    key_indent: usize,
    is_item: bool,
    key: Option<String>,
    value: String,
    block: bool,
    nested: bool,
}

fn note_uses(
    key: &str,
    parsed: &Parsed,
    frames: &[(usize, Kind)],
    found: &mut Vec<usize>,
    line: usize,
) {
    if key == "uses" && parent(frames) == Some(Kind::Step) && is_checkout_uses(&parsed.value) {
        found.push(line);
    }
}

fn push_frame(parsed: &Parsed, key: &str, frames: &mut Vec<(usize, Kind)>) {
    if parsed.block {
        frames.push((parsed.key_indent, Kind::Block));
        return;
    }
    if !parsed.nested {
        return;
    }
    let kind = match (key, parent(frames)) {
        ("jobs", None) => Some(Kind::Jobs),
        (_, Some(Kind::Jobs)) => Some(Kind::Job),
        ("steps", Some(Kind::Job)) => Some(Kind::Steps),
        _ => None,
    };
    if let Some(kind) = kind {
        frames.push((parsed.key_indent, kind));
    }
}

fn parent(frames: &[(usize, Kind)]) -> Option<Kind> {
    frames.last().map(|(_, kind)| *kind)
}

fn block_continues(line: &str, key_indent: usize) -> bool {
    line.trim().is_empty() || leading(line) > key_indent
}

fn parse_line(line: &str) -> Option<Parsed> {
    let indent = leading(line);
    let trimmed = line[indent..].trim_end();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let (is_item, scope, body, key_indent) = if let Some(rest) = trimmed.strip_prefix("- ") {
        let body = rest.trim_start();
        let key_indent = indent + 2 + rest.len() - body.len();
        (true, indent, body, key_indent)
    } else if trimmed == "-" {
        return Some(bare(indent, indent, true));
    } else {
        (false, indent, trimmed, indent)
    };
    if body.starts_with('#') {
        return Some(bare(scope, key_indent, is_item));
    }
    let (key, raw) = body.split_once(':')?;
    let key = clean_key(key)?;
    let raw = strip_comment(raw).trim();
    let block = is_block(raw);
    let nested = !block && raw.is_empty();
    Some(Parsed {
        scope,
        key_indent,
        is_item,
        key: Some(key),
        value: if block || nested {
            String::new()
        } else {
            raw.to_string()
        },
        block,
        nested,
    })
}

fn bare(scope: usize, key_indent: usize, is_item: bool) -> Parsed {
    Parsed {
        scope,
        key_indent,
        is_item,
        key: None,
        value: String::new(),
        block: false,
        nested: false,
    }
}

fn clean_key(key: &str) -> Option<String> {
    let key = key.trim().trim_matches(|ch| ch == '"' || ch == '\'');
    if key.is_empty() || key.contains([' ', '{', '}', '[', ']', ',', '#']) {
        None
    } else {
        Some(key.to_string())
    }
}

fn is_block(value: &str) -> bool {
    matches!(value.chars().next(), Some('|' | '>'))
}

fn leading(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}

pub(super) fn strip_comment(value: &str) -> &str {
    let mut quote = None;
    for (index, ch) in value.char_indices() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }
        if ch == '#' {
            return &value[..index];
        }
    }
    value
}

fn unquote(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}
