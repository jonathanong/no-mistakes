use ignore::WalkBuilder;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod facts;
pub use no_mistakes_ts_facts::jsx;
pub mod module_facts;

pub(crate) use no_mistakes_ts_analysis::bound_comments;
pub(crate) mod builtin_receivers;

mod file_id_map;
mod file_inventory;
mod fixture_read_index;
#[doc(hidden)]
pub use fixture_read_index::{FixtureReadIndex, FixtureReadScope};
pub(crate) use no_mistakes_ts_facts::fixture_reads;
mod path_order;
mod path_remapper;
mod source_store;
pub(crate) use file_id_map::{FileIdMap, FileIdMapIter, FileIdMapIterMut};
pub(crate) use file_inventory::ClassifiedPath;
#[doc(hidden)]
pub use file_inventory::{FileClassification, FileId, FileInventory};
pub use no_mistakes_ts_facts::syntax_helpers::{
    is_test_file, starts_with_use_client, static_property_key_name, unwrap_ts_wrappers,
};
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

use no_mistakes_ts_analysis::comment_parser::{
    comment_body, is_word_char, leading_comment_text, line_comment_start, CommentKind,
    LineCommentScanState,
};

include!("discovery.rs");
include!("discovery_preserve.rs");
include!("visible_snapshot.rs");
include!("disable_comments.rs");

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod comment_parser_markdown_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod comment_parser_template_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod comment_parser_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
