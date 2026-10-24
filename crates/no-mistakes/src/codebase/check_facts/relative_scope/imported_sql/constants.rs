//! Fills a hole that names an exported string or fragment constant.

use super::program::Shared;
use crate::codebase::postgres::embedded::helpers::{split::with_origin_entry, ImportedSite};
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
    let (origin, local) = shared.find(from, &site.local, site.member.as_deref())?;
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
    // Untagged `${frag}` stringifies a statement object. Namespace fragment
    // objects can mutate after their export snapshot, even through a const binding.
    if constant.fragment && (!allow_fragment || site.member.is_some()) {
        return None;
    }
    let sql = constant.sql.clone();
    // Stamp the newly supported namespace literal before composition supplies
    // a caller-local marker; preserve existing named-import report locations.
    let sql = if site.member.is_some() {
        with_origin_entry(sql)
    } else {
        sql
    };
    Some(sql.with_source_file(&origin))
}
