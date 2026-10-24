//! Keep supplemental lookup conservative without changing ordinary graph paths.
use crate::codebase::ts_resolver::normalize_path;
use std::path::{Component, Path, PathBuf};

pub(super) fn lookup_importer(
    specifier: &str,
    importer: &Path,
    consumer_root: &Path,
) -> Option<PathBuf> {
    std::path::absolute(importer)
        .ok()
        .zip(std::path::absolute(consumer_root).ok())
        .and_then(|(importer, root)| {
            lookup_absolute(specifier, &comparable(&importer), &comparable(&root))
        })
}

fn lookup_absolute(specifier: &str, importer: &Path, consumer_root: &Path) -> Option<PathBuf> {
    // A whole project/common-root alias is harmless. Only aliases below that
    // shared boundary can change which package's dependency scope is selected.
    let (common, real_common) = importer
        .ancestors()
        .find(|path| consumer_root.starts_with(path))
        .and_then(|common| {
            // Validate the fixed root without collapsing its logical aliases.
            std::fs::canonicalize(consumer_root)
                .ok()
                .and_then(|_| std::fs::canonicalize(common).ok())
                .map(|real| (common, real))
        })?;
    // common is an importer ancestor, so its remaining components are the suffix.
    let suffix: PathBuf = importer
        .components()
        .skip(common.components().count())
        .collect();
    let mapped = normalize_path(&real_common.join(suffix));
    let actual = std::fs::canonicalize(importer).ok()?;
    if mapped == actual {
        return Some(mapped);
    }

    let relative = specifier.starts_with("./") || specifier.starts_with("../");
    if !relative
        || Path::new(specifier)
            .components()
            .any(|part| part == Component::ParentDir)
    {
        return None;
    }
    // ./children of a directory symlink retain the same physical parent. A
    // declaration-file symlink relocating that parent cannot make that proof.
    mapped
        .parent()
        .and_then(|parent| std::fs::canonicalize(parent).ok())
        .filter(|parent| Some(parent.as_path()) == actual.parent())
        .map(|_| mapped)
}

#[cfg(not(windows))]
fn comparable(path: &Path) -> PathBuf {
    path.to_path_buf()
}

#[cfg(windows)]
fn comparable(path: &Path) -> PathBuf {
    use std::path::Prefix;
    let mut components = path.components();
    let Some(Component::Prefix(prefix)) = components.next() else {
        return path.to_path_buf();
    };
    let mut output = match prefix.kind() {
        Prefix::VerbatimDisk(drive) => PathBuf::from(format!("{}:", char::from(drive))),
        Prefix::VerbatimUNC(server, share) => {
            let mut unc = PathBuf::from(r"\\");
            unc.push(server);
            unc.push(share);
            unc
        }
        _ => return path.to_path_buf(),
    };
    output.extend(components.map(|component| component.as_os_str()));
    output
}
