use super::frame::helper_program;
use super::Shared;
use crate::codebase::postgres::ImportedSite;
use crate::codebase::postgres::{EmbeddedSqlOptions, RecoveredSql, SiteArg};
use std::path::{Path, PathBuf};

/// Field texts of a frozen record, in entry order, with no dedup.
pub(super) fn member<R: FnMut(&str, &Path) -> Option<PathBuf>>(
    shared: &mut Shared<'_, R>,
    options: &EmbeddedSqlOptions,
    from: &Path,
    site: &ImportedSite,
) -> Option<Vec<RecoveredSql>> {
    let field = site.member.as_deref()?;
    let (origin, local) = shared.find(from, &site.local, None)?;
    if origin == from {
        return None;
    }
    let program = helper_program(shared, options, &origin, &local)?;
    let index = program.frozen_key?;
    let entries: Vec<_> = match site.args.get(index)? {
        SiteArg::Literal(literal) => vec![program
            .frozen_entries
            .iter()
            .find(|entry| entry.key == literal.text)?],
        SiteArg::Scalar => program.frozen_entries.iter().collect(),
        _ => return None,
    };
    let mut texts = Vec::with_capacity(entries.len());
    for entry in entries {
        let sql = entry
            .fields
            .iter()
            .find(|field_value| field_value.name == field)?;
        texts.push(sql.sql.clone().with_source_file(&origin));
    }
    (!texts.is_empty()).then_some(texts)
}
