use super::ChangedFiles;
use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

impl ChangedFiles {
    pub(crate) fn push_manual_path(&mut self, path: &Path, root: &Path) -> Result<()> {
        let (inventory_path, analysis_path) = resolve_manual_path(path, root)?;
        push_unique(&mut self.inventory_files, inventory_path);
        push_unique(&mut self.files, analysis_path.clone());
        push_unique(&mut self.authoritative_files, analysis_path.clone());
        push_unique(&mut self.manual_files, analysis_path.clone());
        if !file_is_present(&analysis_path) {
            push_unique(&mut self.deleted, analysis_path);
        }
        Ok(())
    }
}

fn push_unique(list: &mut Vec<PathBuf>, path: PathBuf) {
    if !list.contains(&path) {
        list.push(path);
    }
}

pub(super) fn normalize_unique(files: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique = HashSet::new();
    files
        .into_iter()
        .map(|path| no_mistakes::codebase::ts_resolver::normalize_path(&path))
        .filter(|path| unique.insert(path.clone()))
        .collect()
}

pub(super) fn resolve_manual_path(path: &Path, root: &Path) -> Result<(PathBuf, PathBuf)> {
    let lexical = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let lexical = no_mistakes::codebase::ts_resolver::normalize_path(&lexical);
    if !lexical.starts_with(root) {
        anyhow::bail!(
            "changed file `{}` is outside the project root `{}`",
            path.display(),
            root.display()
        )
    }
    let analysis = lexical.canonicalize().unwrap_or_else(|_| lexical.clone());
    if !analysis.starts_with(root) {
        anyhow::bail!(
            "changed file `{}` resolves outside the project root `{}`",
            path.display(),
            root.display()
        )
    }
    Ok((lexical, analysis))
}

pub(crate) fn existing_changed_files(changed: &ChangedFiles) -> Vec<PathBuf> {
    changed
        .files
        .iter()
        .filter(|f| file_is_present(f))
        .cloned()
        .collect()
}

fn file_is_present(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(_) => true,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
            ) =>
        {
            false
        }
        Err(_) => true,
    }
}
