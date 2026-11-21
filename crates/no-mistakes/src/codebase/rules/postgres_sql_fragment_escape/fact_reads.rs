use super::text::export_offset;
use crate::codebase::ts_source::facts::TsFileFacts;

pub(super) fn export_site(ts: &TsFileFacts, source: &str, local: &str) -> Option<u32> {
    ts.namespaces
        .export_name_offsets
        .iter()
        .find(|(name, _)| name == local)
        .map(|(_, offset)| *offset)
        .or_else(|| export_offset(source, local))
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
    if let Some((index, rest)) = depth_zero_callee(window) {
        let offset = window_start.checked_add(u32::try_from(index).ok()?)?;
        if let Some(spec) = specifier_at_offset(ts, offset) {
            return Some(spec.to_string());
        }
        let after = rest
            .strip_prefix("import(")
            .or_else(|| rest.strip_prefix("require("))?;
        return super::text::whole_quoted(after).map(str::to_string);
    }
    window
        .is_empty()
        .then(|| specifier_at_offset(ts, window_start).map(str::to_string))
        .flatten()
}

fn depth_zero_callee(window: &str) -> Option<(usize, &str)> {
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
        if !callee_at_start(rest, "import") && !callee_at_start(rest, "require") {
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

pub(super) fn specifier_between(ts: &TsFileFacts, start: u32, end: u32) -> Option<&str> {
    ts.namespaces
        .literal_import_spans
        .iter()
        .find(|(offset, _)| *offset >= start && *offset < end)
        .map(|(_, specifier)| specifier.as_str())
}

pub(super) fn specifier_at_offset(ts: &TsFileFacts, offset: u32) -> Option<&str> {
    ts.namespaces
        .literal_import_spans
        .iter()
        .find(|(start, _)| *start == offset)
        .map(|(_, specifier)| specifier.as_str())
}

pub(super) fn window_loads_module(window: &str) -> bool {
    let rhs = window.split_once('=').map(|(_, rhs)| rhs).unwrap_or(window);
    direct_module_load(rhs.trim().trim_end_matches(';'))
}

pub(super) fn direct_module_load(text: &str) -> bool {
    let text = text.trim_start();
    let text = text
        .strip_prefix("await")
        .map(str::trim_start)
        .unwrap_or(text);
    callee_at_start(text, "import") || callee_at_start(text, "require")
}

fn callee_at_start(text: &str, callee: &str) -> bool {
    text.strip_prefix(callee)
        .is_some_and(|rest| rest.starts_with('('))
}
