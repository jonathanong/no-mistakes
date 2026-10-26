use super::{
    check_items, extract_program, CheckContext, FileFacts, NoMistakesConfig, Path, RuleFinding,
};
use anyhow::Result;
use rayon::prelude::*;
use std::path::PathBuf;

pub(super) struct BorrowedFactItem<'a> {
    pub(super) path: &'a Path,
    pub(super) facts: &'a FileFacts,
    pub(super) error: Option<&'a str>,
}

impl<'a> BorrowedFactItem<'a> {
    pub(super) fn new(path: &'a Path, facts: &'a FileFacts, error: Option<&'a str>) -> Self {
        Self { path, facts, error }
    }
}

struct LoadedFactItem {
    path: PathBuf,
    facts: FileFacts,
    error: Option<String>,
}

pub(super) fn check_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::super::source_store_for_files(files);
    let items: Vec<_> = files
        .par_iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .map(|path| {
            let (facts, error) = match sources.read_path(path) {
                Ok(source) => match crate::ast::with_program(path, &source, |program, source| {
                    extract_program(path, source, program)
                }) {
                    Ok(facts) => (facts, None),
                    Err(error) => (FileFacts::default(), Some(error.to_string())),
                },
                Err(error) => (FileFacts::default(), Some(error.to_string())),
            };
            LoadedFactItem {
                path: path.clone(),
                facts,
                error,
            }
        })
        .collect();
    check_items(
        CheckContext {
            root,
            config,
            sources: &sources,
            inferred_roots: None,
            defer_suppression: false,
        },
        &items,
        |item| item.path.as_path(),
        |item| (&item.facts, item.error.as_deref()),
    )
    .map(|output| output.findings)
}
