use std::path::Path;

pub(super) fn build_path(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|&&part| !part.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("/")
}

/// Preserve the directory/wildcard spelling, allowing an extension on files.
/// Exact aliases may also name the directory's index file.
pub(super) fn target_matches(expected: &str, target: &str) -> bool {
    if target == expected {
        return true;
    }
    let Some(suffix) = target.strip_prefix(expected) else {
        return false;
    };
    let extension = suffix.strip_prefix('.').or_else(|| {
        (!expected.contains('*'))
            .then(|| suffix.strip_prefix("/index."))
            .flatten()
    });
    extension
        .is_some_and(|extension| !extension.is_empty() && !extension.contains(['/', '\\', '*']))
        && (!expected.contains('*') || expected.ends_with('*'))
}

pub(super) fn target_exists(root: &Path, expected: &str, target: &str) -> bool {
    if let Some((directory, _)) = target.split_once('*') {
        return root.join(directory.trim_end_matches('/')).is_dir();
    }
    let path = root.join(target);
    // A bare directory may itself contain a dot; only an added file suffix
    // changes the mapping from the existing directory form to a file form.
    if target == expected {
        path.is_dir()
    } else {
        path.is_file()
    }
}
