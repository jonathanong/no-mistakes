use super::{FileFacts, Want};

pub(super) fn stable_query(want: Want<'_>) -> bool {
    matches!(
        want,
        Want::Property(_) | Want::StableString | Want::StableName
    )
}

pub(super) fn direct_export(file: &FileFacts, imported: &str, want: Want<'_>) -> Option<String> {
    if stable_query(want) && file.blocked_enqueue_exports.contains(imported) {
        return None;
    }
    match want {
        Want::Property(key) => file
            .exported_enqueue_properties
            .get(imported)?
            .get(key)
            .cloned(),
        Want::StableString => file.stable_exported_strings.get(imported).cloned(),
        Want::StableName => file
            .stable_exported_strings
            .get(imported)
            .cloned()
            .or_else(|| file.queue_exports.get(imported).cloned()),
        Want::Queue => file.queue_exports.get(imported).cloned(),
        Want::Definition => None,
        Want::Either => file
            .exported_strings
            .get(imported)
            .cloned()
            .or_else(|| file.queue_exports.get(imported).cloned()),
    }
}

pub(super) fn imported_local<'a>(
    file: &'a FileFacts,
    name: &'a str,
    want: Want<'_>,
) -> Option<&'a str> {
    if stable_query(want) {
        return file.enqueue_export_aliases.get(name).map(String::as_str);
    }
    Some(name)
}
