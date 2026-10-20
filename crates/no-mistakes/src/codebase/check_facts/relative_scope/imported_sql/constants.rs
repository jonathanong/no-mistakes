//! Fills a hole that names an exported string or fragment constant.

use super::program::Shared;
use crate::codebase::postgres::embedded::helpers::ImportedSite;
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::postgres::EmbeddedSqlOptions;
use std::path::{Path, PathBuf};

pub(super) fn recovered<R: FnMut(&str, &Path) -> Option<PathBuf>>(
    shared: &mut Shared<'_, R>,
    options: &EmbeddedSqlOptions,
    from: &Path,
    site: &ImportedSite,
    allow_fragment: bool,
) -> Option<RecoveredSql> {
    let (origin, local) = shared.find(from, &site.local)?;
    // The name was not an import of another module. A same-file const is
    // inlined during the walk, so this hole has nothing to fill.
    if origin == from {
        return None;
    }
    let (_, embedded) = shared
        .files
        .get(&origin)?
        .embedded_sql
        .iter()
        .find(|(known, _)| known == options)?;
    let constant = embedded.exported_constants.get(&local)?;
    // Untagged `${frag}` stringifies a statement object. Only `.append` copies it.
    if constant.fragment && !allow_fragment {
        return None;
    }
    Some(constant.sql.clone().with_source_file(&origin))
}
