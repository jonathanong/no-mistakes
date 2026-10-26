use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::SourceStore;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub(super) fn is_declaration(path: &Path) -> bool {
    path.to_str().is_some_and(|name| {
        name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
    })
}

pub(super) fn declaration(path: &Path, sources: &SourceStore) -> Result<Option<PathBuf>, ()> {
    let path = normalize_path(path);
    if is_declaration(&path) {
        return unique(vec![path], sources);
    }
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("js") => unique(vec![path.with_extension("d.ts")], sources),
        Some("mjs") => unique(vec![path.with_extension("d.mts")], sources),
        Some("cjs") => unique(vec![path.with_extension("d.cts")], sources),
        Some(_) => Ok(None),
        None => {
            let stem = path.to_str().ok_or(())?;
            let files = [".d.ts", ".d.mts", ".d.cts"]
                .map(|suffix| PathBuf::from(format!("{stem}{suffix}")));
            if let Some(found) = unique(files.to_vec(), sources)? {
                return Ok(Some(found));
            }
            unique(
                ["index.d.ts", "index.d.mts", "index.d.cts"]
                    .map(|name| path.join(name))
                    .to_vec(),
                sources,
            )
        }
    }
}

fn unique(paths: Vec<PathBuf>, sources: &SourceStore) -> Result<Option<PathBuf>, ()> {
    let mut found = None;
    for path in paths {
        match sources.read_path(&path) {
            Ok(_) if found.is_some() => return Err(()),
            Ok(_) => found = Some(path),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(_) => return Err(()),
        }
    }
    Ok(found)
}
