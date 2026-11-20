pub(super) fn quoted(text: &str) -> Option<&str> {
    let quote = text.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &text[quote.len_utf8()..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

pub(super) fn whole_quoted(text: &str) -> Option<&str> {
    let text = text.trim();
    let spec = quoted(text)?;
    let used = spec.len().checked_add(2)?;
    let rest = text.get(used..)?.trim_start();
    if rest.is_empty() || rest.starts_with(')') || rest.starts_with(',') || rest.starts_with('.') {
        Some(spec)
    } else {
        None
    }
}

pub(super) fn declarator_text(rest: &str) -> &str {
    let mut depth = 0i32;
    for (index, ch) in rest.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth > 0 => depth -= 1,
            ',' | ';' | '\n' if depth == 0 => return &rest[..index],
            _ => {}
        }
    }
    rest
}

pub(super) fn export_alias_local<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let mut rest = source;
    let token = "export {";
    while let Some(index) = rest.find(token) {
        let after = &rest[index + token.len()..];
        let Some(end) = after.find('}') else {
            break;
        };
        if let Some(local) = alias_local(&after[..end], name) {
            return Some(local);
        }
        rest = &after[end..];
    }
    None
}

pub(super) fn import_specifier<'a>(source: &'a str, local: &str) -> Option<&'a str> {
    for line in source.lines() {
        let line = line.trim();
        if !line.starts_with("import ") || !line.contains(local) {
            continue;
        }
        let from = line.split(" from ").nth(1)?;
        return quoted(from.trim().trim_end_matches(';'));
    }
    None
}

pub(super) fn import_equals_member<'a>(source: &'a str, name: &str) -> Option<(&'a str, &'a str)> {
    let marker = format!("import {name} = ");
    let mut rest = source;
    while let Some(index) = rest.find(&marker) {
        let after = rest[index + marker.len()..].trim_start();
        let head = after.split([';', '\n']).next().unwrap_or(after).trim();
        if let Some((object, member)) = head.split_once('.') {
            let object = object.trim();
            let member = member.trim();
            if is_simple(object) && is_simple(member) {
                return Some((object, member));
            }
        }
        rest = &rest[index + marker.len()..];
    }
    None
}

pub(super) fn import_alias<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("import {name} = ");
    let mut rest = source;
    while let Some(index) = rest.find(&marker) {
        let after = rest[index + marker.len()..].trim_start();
        let head = after.split([';', '\n']).next().unwrap_or(after).trim();
        if is_simple(head) {
            return Some(head);
        }
        rest = &rest[index + marker.len()..];
    }
    None
}

fn is_simple(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(ch) if ch == '_' || ch.is_ascii_alphabetic() => {
            chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        }
        _ => false,
    }
}

pub(super) fn export_offset(source: &str, local: &str) -> Option<u32> {
    let mut rest = source;
    let mut base = 0u32;
    while let Some(index) = rest.find("export") {
        let boundary = rest[..index]
            .chars()
            .next_back()
            .is_none_or(|ch| ch != '_' && !ch.is_ascii_alphanumeric());
        let advance = index + "export".len();
        if boundary {
            let decl = export_declaration(&rest[index..]);
            if !decl.contains("import ") && export_mentions_local(decl, local) {
                return Some(base + u32::try_from(index).ok()?);
            }
        }
        base = base.checked_add(u32::try_from(advance).ok()?)?;
        rest = rest.get(advance..)?;
    }
    None
}

fn export_declaration(text: &str) -> &str {
    let mut depth = 0i32;
    for (index, ch) in text.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    let tail = &text[index + 1..];
                    if let Some(semi) = tail.find(';') {
                        if !tail[..semi].contains('\n') {
                            return &text[..=index + 1 + semi];
                        }
                    }
                    return &text[..=index];
                }
            }
            '\n' | ';' if depth == 0 => return &text[..=index],
            _ => {}
        }
    }
    text
}

fn export_mentions_local(line: &str, local: &str) -> bool {
    let body = line
        .trim()
        .trim_start_matches("export")
        .trim()
        .trim_start_matches('{')
        .trim_end_matches(';')
        .trim_end_matches('}');
    body.split(',').any(|part| {
        let part = part.trim().trim_end_matches(';').trim();
        let name = part
            .split_once(" as ")
            .map(|(name, _)| name.trim())
            .unwrap_or(part);
        name == local
    })
}

pub(super) fn import_call_specifier(text: &str) -> Option<&str> {
    let head = text
        .split_once(".then")
        .map(|(head, _)| head)
        .unwrap_or(text);
    let after = head
        .split("import(")
        .nth(1)
        .or_else(|| head.split("require(").nth(1))?;
    whole_quoted(after)
}

fn alias_local<'a>(body: &'a str, name: &str) -> Option<&'a str> {
    let body = body
        .trim()
        .trim_start_matches("export")
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}');
    for part in body.split(',') {
        let part = part.trim().trim_end_matches(';').trim();
        if let Some((local, exported)) = part.split_once(" as ") {
            if exported.trim() == name {
                return Some(local.trim());
            }
        } else if part == name {
            return Some(part);
        }
    }
    None
}
