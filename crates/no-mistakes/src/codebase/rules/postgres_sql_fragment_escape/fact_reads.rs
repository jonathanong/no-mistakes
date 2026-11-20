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

pub(super) fn specifier_at(source: &str, start: u32, end: u32) -> Option<&str> {
    let text = source.get(start as usize..end as usize)?;
    callee_specifier(text, "import").or_else(|| callee_specifier(text, "require"))
}

fn callee_specifier<'a>(text: &'a str, callee: &str) -> Option<&'a str> {
    let mut rest = text;
    while let Some(index) = rest.find(callee) {
        let after = &rest[index + callee.len()..];
        let prev = rest[..index].chars().next_back();
        let boundary = prev.is_none_or(|ch| ch != '.' && ch != '_' && !ch.is_ascii_alphanumeric());
        if boundary && after.starts_with('(') {
            return super::text::whole_quoted(&after[1..]);
        }
        rest = &rest[index + callee.len()..];
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
