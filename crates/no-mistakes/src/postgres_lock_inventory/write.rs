use super::LockInventoryBaseline;
use anyhow::{ensure, Context, Result};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

pub(super) fn publish(
    root: &Path,
    relative: &str,
    baseline: &LockInventoryBaseline,
    existing: bool,
) -> Result<()> {
    let relative = Path::new(relative);
    let parent = root.join(relative.parent().expect("baseline path was validated"));
    let content = serde_json::to_vec_pretty(baseline)?;
    crate::invocation::commit_timeout()?;
    let canonical_parent = prepare_parent(root, &parent)?;
    let output = canonical_parent.join(relative.file_name().expect("baseline path was validated"));
    if existing {
        let metadata = output
            .symlink_metadata()
            .context("inspect lock inventory baseline before publication")?;
        ensure!(metadata.is_file(), "baselineFile must be a regular file");
    }
    let mut temporary = tempfile::NamedTempFile::new_in(&canonical_parent)?;
    temporary.write_all(&content)?;
    temporary.write_all(b"\n")?;
    crate::invocation::commit_timeout()?;
    if existing {
        temporary.persist(&output)
    } else {
        temporary.persist_noclobber(&output)
    }
    .map_err(|error| error.error)
    .context("publish lock inventory baseline")?;
    Ok(())
}

fn prepare_parent(root: &Path, parent: &Path) -> Result<PathBuf> {
    let canonical_root = root.canonicalize().context("resolve repository root")?;
    let mut ancestor = parent;
    let canonical = loop {
        if ancestor == root {
            break canonical_root.clone();
        }
        match ancestor.canonicalize() {
            Ok(canonical) => break canonical,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                ancestor = ancestor
                    .parent()
                    .expect("baseline parent is within the root");
            }
            Err(error) => return Err(error).context("resolve lock inventory baseline directory"),
        }
    };
    ensure!(
        canonical.starts_with(&canonical_root),
        "baselineFile parent must stay inside the repository"
    );
    if ancestor == parent {
        return Ok(canonical);
    }
    let directory = canonical.join(parent.strip_prefix(ancestor)?);
    fs::create_dir_all(&directory).context("create lock inventory baseline directory")?;
    let directory = directory
        .canonicalize()
        .context("resolve lock inventory baseline directory")?;
    ensure!(
        directory.starts_with(&canonical_root),
        "baselineFile parent must stay inside the repository"
    );
    Ok(directory)
}
