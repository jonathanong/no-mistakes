use ignore::WalkBuilder;
use oxc_ast::ast::{Expression, PropertyKey};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod facts;
pub mod jsx;
pub mod module_facts;

pub(crate) mod bound_comments;
pub(crate) mod builtin_receivers;

mod file_id_map;
mod file_inventory;
mod fixture_read_index;
mod fixture_reads;
#[doc(hidden)]
pub use fixture_read_index::{FixtureReadIndex, FixtureReadScope};
mod parser_diagnostic;
mod path_order;
mod path_remapper;
mod source_store;
pub(crate) use file_id_map::{FileIdMap, FileIdMapIter, FileIdMapIterMut};
pub(crate) use file_inventory::ClassifiedPath;
#[doc(hidden)]
pub use file_inventory::{FileClassification, FileId, FileInventory};
pub(crate) use parser_diagnostic::format_parse_diagnostic;
pub(crate) use path_order::{cmp_os_str_paths, sort_os_str_paths};
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
include!("comment_parser_html.rs");
include!("comment_parser_modes.rs");
include!("syntax_helpers.rs");

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod comment_parser_markdown_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod comment_parser_template_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod comment_parser_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
