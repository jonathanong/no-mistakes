use super::{
    check_items, extract_program, FileFacts, NoMistakesConfig, Path, PathBuf, RuleFinding,
};
use anyhow::Result;
use rayon::prelude::*;

pub(super) struct BorrowedFactItem<'a> {
    pub(super) path: &'a Path,
    pub(super) facts: &'a FileFacts,
}

impl<'a> BorrowedFactItem<'a> {
    pub(super) fn new(path: &'a Path, facts: &'a FileFacts) -> Self {
        Self { path, facts }
    }
}

struct LoadedFactItem {
    path: PathBuf,
    facts: FileFacts,
}

pub(super) fn check_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources: Vec<_> = files
        .par_iter()
        .filter_map(|path| {
            std::fs::read_to_string(path).ok().map(|source| {
                let facts = crate::ast::with_program(path, &source, |program, source| {
                    extract_program(path, source, program)
                })
                .unwrap_or_default();
                LoadedFactItem {
                    path: path.clone(),
                    facts,
                }
            })
        })
        .collect();
    check_items(
        root,
        config,
        &sources,
        |item| item.path.as_path(),
        |item| &item.facts,
        None,
    )
}
