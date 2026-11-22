use crate::codebase::ts_source::facts::TsFileFacts;

pub(super) fn export_site(ts: &TsFileFacts, local: &str) -> Option<u32> {
    ts.namespaces
        .export_name_offsets
        .iter()
        .find(|(name, _)| name == local)
        .map(|(_, offset)| *offset)
}

pub(super) fn equals_member(ts: &TsFileFacts, name: &str) -> Option<(String, String)> {
    ts.namespaces
        .import_equals_members
        .iter()
        .find(|(local, _, _)| local == name)
        .map(|(_, object, member)| (object.clone(), member.clone()))
}

pub(super) fn equals_alias(ts: &TsFileFacts, name: &str) -> Option<String> {
    ts.namespaces
        .import_equals_aliases
        .iter()
        .find(|(local, _)| local == name)
        .map(|(_, target)| target.clone())
}

pub(super) fn require_specifier<'a>(ts: &'a TsFileFacts, name: &str) -> Option<&'a str> {
    ts.namespaces
        .import_equals_requires
        .iter()
        .find(|(local, _)| local == name)
        .map(|(_, specifier)| specifier.as_str())
}

pub(super) fn depth_zero_module_specifier(
    ts: &TsFileFacts,
    window: &str,
    window_start: u32,
) -> Option<String> {
    if let Some((index, _)) = depth_zero_callee(ts, window, window_start) {
        let offset = window_start.checked_add(u32::try_from(index).ok()?)?;
        return specifier_at_offset(ts, offset).map(str::to_string);
    }
    window
        .is_empty()
        .then(|| specifier_at_offset(ts, window_start).map(str::to_string))
        .flatten()
}

pub(super) fn unrecorded_static_specifier(
    ts: &TsFileFacts,
    window: &str,
    window_start: u32,
) -> bool {
    let Some((index, rest)) = depth_zero_callee(ts, window, window_start) else {
        return false;
    };
    let offset = window_start.saturating_add(u32::try_from(index).unwrap_or(0));
    if specifier_at_offset(ts, offset).is_some() {
        return false;
    }
    rest.strip_prefix("import")
        .or_else(|| rest.strip_prefix("require"))
        .and_then(|after| after.trim_start().strip_prefix('('))
        .is_some_and(|after| super::text::whole_quoted(after).is_some())
}

fn depth_zero_callee<'a>(
    ts: &TsFileFacts,
    window: &'a str,
    window_start: u32,
) -> Option<(usize, &'a str)> {
    let spans = &ts.namespaces.literal_import_spans;
    let first = spans.partition_point(|(offset, _)| *offset < window_start);
    let mut recorded = spans[first..].iter().peekable();
    let mut depth = 0i32;
    for (index, ch) in window.char_indices() {
        if ch == '(' {
            depth += 1;
            continue;
        }
        if ch == ')' && depth > 0 {
            depth -= 1;
            continue;
        }
        if depth != 0 {
            continue;
        }
        let rest = &window[index..];
        let offset = window_start as u64 + index as u64;
        while recorded
            .peek()
            .is_some_and(|(start, _)| u64::from(*start) < offset)
        {
            recorded.next();
        }
        let has_fact = recorded
            .peek()
            .is_some_and(|(start, _)| u64::from(*start) == offset);
        if !has_fact && !callee_at_start(rest, "import") && !callee_at_start(rest, "require") {
            continue;
        }
        let prev = window[..index].chars().next_back();
        if prev.is_some_and(|prev| prev == '.' || prev == '_' || prev.is_ascii_alphanumeric()) {
            continue;
        }
        return Some((index, rest));
    }
    None
}

pub(super) fn specifier_at_offset(ts: &TsFileFacts, offset: u32) -> Option<&str> {
    let spans = &ts.namespaces.literal_import_spans;
    let index = spans.partition_point(|(start, _)| *start < offset);
    spans
        .get(index)
        .filter(|(start, _)| *start == offset)
        .map(|(_, specifier)| specifier.as_str())
}

pub(super) fn window_loads_module(ts: &TsFileFacts, window: &str, start: u32) -> bool {
    let rhs = window.split_once('=').map(|(_, rhs)| rhs).unwrap_or(window);
    let call = rhs.trim_start();
    let call = call
        .strip_prefix("await")
        .filter(|rest| rest.starts_with(char::is_whitespace))
        .map(str::trim_start)
        .unwrap_or(call);
    let offset = start.saturating_add(u32::try_from(window.len() - call.len()).unwrap_or(u32::MAX));
    let recorded = specifier_at_offset(ts, offset).is_some();
    direct_module_load(call.trim_end().trim_end_matches(';'), recorded)
}

pub(super) fn direct_module_load(text: &str, recorded: bool) -> bool {
    let rest = if recorded {
        text.find('(').map(|index| &text[index..])
    } else {
        text.strip_prefix("import")
            .or_else(|| text.strip_prefix("require"))
    };
    let Some(rest) = rest.and_then(|rest| rest.trim_start().strip_prefix('(')) else {
        return false;
    };
    close_call(rest).is_some_and(|after| after.trim().is_empty())
}

fn close_call(text: &str) -> Option<&str> {
    let mut depth = 1i32;
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        match ch {
            '/' if chars.next_if(|(_, next)| *next == '/').is_some() => {
                while chars.next_if(|(_, next)| *next != '\n').is_some() {}
            }
            '/' if chars.next_if(|(_, next)| *next == '*').is_some() => loop {
                match chars.next() {
                    Some((_, '*')) if chars.next_if(|(_, next)| *next == '/').is_some() => break,
                    Some(_) => {}
                    None => return None,
                }
            },
            '"' | '\'' | '`' => {
                while let Some((_, inner)) = chars.next() {
                    if inner == '\\' {
                        chars.next();
                        continue;
                    }
                    if inner == ch {
                        break;
                    }
                }
            }
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return text.get(index + ch.len_utf8()..);
                }
            }
            _ => {}
        }
    }
    None
}

fn callee_at_start(text: &str, callee: &str) -> bool {
    text.strip_prefix(callee)
        .is_some_and(|rest| rest.trim_start().starts_with('('))
}
