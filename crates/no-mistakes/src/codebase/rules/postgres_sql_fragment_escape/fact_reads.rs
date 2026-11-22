use super::load_windows::depth_zero_callee;
pub(super) use super::load_windows::window_loads_module;
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
        .is_some_and(super::load_lex::static_argument)
}

pub(super) fn specifier_at_offset(ts: &TsFileFacts, offset: u32) -> Option<&str> {
    if ts
        .namespaces
        .literal_require_resolve_offsets
        .binary_search(&offset)
        .is_ok()
    {
        return None;
    }
    let spans = &ts.namespaces.literal_import_spans;
    let index = spans.partition_point(|(start, _)| *start < offset);
    spans
        .get(index)
        .filter(|(start, _)| *start == offset)
        .map(|(_, specifier)| specifier.as_str())
}
