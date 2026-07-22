use ignore::WalkBuilder;
use oxc_ast::ast::{Expression, PropertyKey};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod facts;
pub mod jsx;

mod file_inventory;
mod parser_diagnostic;
mod path_remapper;
mod source_store;
pub(crate) use file_inventory::ClassifiedPath;
#[doc(hidden)]
pub use file_inventory::{FileClassification, FileId, FileInventory};
pub(crate) use parser_diagnostic::format_parse_diagnostic;
pub(crate) use path_remapper::FrozenPathRemapper;
#[doc(hidden)]
pub use source_store::{JsonLoadError, SourceReadOutcome, SourceStore};

pub(crate) fn deduplicate_analysis_paths<'a>(
    paths: impl IntoIterator<Item = &'a PathBuf>,
) -> Vec<PathBuf> {
    let mut unique = BTreeMap::<PathBuf, PathBuf>::new();
    for path in paths {
        let normalized = crate::codebase::ts_resolver::normalize_path(path);
        unique
            .entry(normalized)
            .and_modify(|existing| {
                if path < existing {
                    *existing = path.clone();
                }
            })
            .or_insert_with(|| path.clone());
    }
    unique.into_values().collect()
}

include!("discovery.rs");
include!("discovery_preserve.rs");
include!("visible_snapshot.rs");
include!("disable_comments.rs");
include!("comment_parser.rs");
include!("comment_parser_modes.rs");
include!("syntax_helpers.rs");

#[cfg(test)]
mod comment_parser_tests;
#[cfg(test)]
mod tests;
