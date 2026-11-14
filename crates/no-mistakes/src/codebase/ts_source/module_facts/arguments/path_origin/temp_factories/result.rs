/// Call result of a Node temporary factory.
/// Distinct from the module-family sync flag: awaiting `mkdtempSync` is still a
/// string, and callback `fs.mkdtemp` does not return a directory.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::codebase::ts_source::module_facts::arguments::path_origin) enum TempReturn {
    Directory,
    Promise,
    Callback,
}

pub(super) fn named(imported: &str, sync: bool) -> Option<TempReturn> {
    match (imported, sync) {
        ("mkdtempSync", true) => Some(TempReturn::Directory),
        ("mkdtemp", true) => Some(TempReturn::Callback),
        ("mkdtemp", false) => Some(TempReturn::Promise),
        _ => None,
    }
}

pub(super) fn node_fs(specifier: &str) -> bool {
    matches!(
        specifier,
        "node:fs" | "fs" | "node:fs/promises" | "fs/promises"
    )
}

pub(super) fn factory(name: &str, specifier: &str) -> bool {
    node_fs(specifier) && named(name, matches!(specifier, "fs" | "node:fs")).is_some()
}
