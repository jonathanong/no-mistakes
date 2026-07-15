use super::{NextJsProjectLookup, RULE_ID};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path};
use crate::codebase::unique_exports::SourceFile;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(crate) fn collect_source_files(root: &Path, files: &[PathBuf]) -> Result<Vec<SourceFile>> {
    let visible_files = crate::codebase::ts_source::discover_visible_paths(root);
    let nextjs_projects = NextJsProjectLookup::new(root, files, &visible_files);
    files
        .par_iter()
        .map(|path| {
            let source = std::fs::read_to_string(path)
                .with_context(|| format!("reading source file {}", path.display()))?;
            let is_tsx = matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("tsx" | "jsx")
            );
            let disabled = has_disable_file_comment(&source, RULE_ID);
            let symbols = if disabled {
                Default::default()
            } else {
                crate::codebase::ts_symbols::extract_symbols_at_path(path, &source, is_tsx)
                    .with_context(|| format!("extracting symbols from {}", path.display()))?
            };
            Ok(SourceFile {
                path: normalize_path(path),
                rel: relative_slash_path(root, path),
                disabled,
                is_nextjs_project: nextjs_projects.contains_file(path),
                source,
                symbols: symbols.into(),
            })
        })
        .collect()
}

pub(crate) fn file_is_in_nextjs_project(root: &Path, path: &Path) -> bool {
    let visible_files = crate::codebase::ts_source::discover_visible_paths(root);
    NextJsProjectLookup::new(root, &[path.to_path_buf()], &visible_files).contains_file(path)
}
