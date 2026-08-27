use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub(super) fn item_values<'a>(
    items: Option<&'a serde_json::Value>,
    item_name: &str,
) -> Vec<&'a serde_json::Value> {
    let Some(items) = items else {
        return Vec::new();
    };
    items
        .get(item_name)
        .and_then(|value| value.as_array())
        .map(|values| values.iter().collect())
        .unwrap_or_default()
}

pub(super) fn default_compile_files(
    all_files: &[PathBuf],
    project_dir: &Path,
) -> BTreeSet<PathBuf> {
    all_files
        .iter()
        .filter(|path| path.starts_with(project_dir))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("cs"))
        .filter(|path| {
            !path
                .strip_prefix(project_dir)
                .expect("project descendant should have a relative path")
                .components()
                .any(|component| {
                    component.as_os_str() == OsStr::new("bin")
                        || component.as_os_str() == OsStr::new("obj")
                })
        })
        .cloned()
        .collect()
}
