//! Fills a hole that names an exported immutable string constant.

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
    // A const export fixes the binding, not the builder object's contents.
    // Another importer can mutate it after this per-file export snapshot.
    if constant.fragment {
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
