//! Match a live enumeration root against the configured runtime-asset roots.
use super::config::RuntimeAsset;
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathOrigin};
use std::path::Path;

fn parts(path: &Path) -> Vec<String> {
    path.to_string_lossy()
        .split(['/', '\\'])
        .filter(|part| !part.is_empty() && *part != ".")
        .map(str::to_owned)
        .collect()
}

/// The declared asset root that wholly contains the enumerated path, if any.
///
/// A path with a non-literal component, or one that climbs out of the
/// repository, is never matched: the declaration names a tree, not a guess.
pub(super) fn declared<'o>(
    origin: &PathOrigin,
    root: &Path,
    file: &Path,
    assets: &'o [RuntimeAsset],
) -> Option<&'o RuntimeAsset> {
    if origin.dynamic || origin.env.is_some() || origin.param.is_some() {
        return None;
    }
    let mut path = match origin.anchor {
        PathAnchor::Module => parts(file.parent()?.strip_prefix(root).ok()?),
        PathAnchor::Cwd | PathAnchor::Relative | PathAnchor::Absolute => Vec::new(),
        _ => return None,
    };
    let segments = if origin.anchor == PathAnchor::Absolute {
        origin.segments.strip_prefix(parts(root).as_slice())?
    } else {
        &origin.segments
    };
    for segment in segments {
        if segment == ".." {
            path.pop()?;
        } else {
            path.push(segment.clone());
        }
    }
    assets
        .iter()
        .find(|asset| path.starts_with(&parts(Path::new(&asset.root))))
}
