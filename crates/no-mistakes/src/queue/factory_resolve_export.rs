use super::{FileFacts, Want};

pub(super) fn direct_export(file: &FileFacts, imported: &str, want: Want<'_>) -> Option<String> {
    match want {
        Want::Property(key) => file
            .exported_enqueue_properties
            .get(imported)?
            .get(key)
            .cloned(),
        Want::Queue => file.queue_exports.get(imported).cloned(),
        Want::Definition => None,
        Want::Either => file
            .exported_strings
            .get(imported)
            .cloned()
            .or_else(|| file.queue_exports.get(imported).cloned()),
    }
}

pub(super) fn imported_local<'a>(file: &'a FileFacts, name: &'a str, want: Want<'_>) -> &'a str {
    if matches!(want, Want::Property(_)) {
        return file
            .reexports
            .iter()
            .find(|(exported, _)| exported == name)
            .map_or(name, |(_, local)| local.as_str());
    }
    name
}
