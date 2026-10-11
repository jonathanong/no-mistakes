use super::{Options, Path};

pub(super) fn normalize_inherited_root(inherited: &mut Options, path: &Path) {
    let Some(root) = inherited.root.as_deref() else {
        return;
    };
    let root = Path::new(root);
    if !root.is_absolute() {
        inherited.root = Some(
            crate::codebase::ts_resolver::normalize_path(
                &path.parent().unwrap_or(Path::new(".")).join(root),
            )
            .to_string_lossy()
            .into_owned(),
        );
    }
}

pub(super) fn combine_excludes(
    inherited: Option<Vec<String>>,
    local: Option<Vec<String>>,
) -> Option<Vec<String>> {
    let mut excludes = inherited.unwrap_or_default();
    excludes.extend(local.unwrap_or_default());
    (!excludes.is_empty()).then_some(excludes)
}
