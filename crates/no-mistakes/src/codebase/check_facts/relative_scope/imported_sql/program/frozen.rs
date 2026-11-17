use super::frame::helper_program;
use super::{Filled, Shared};
use crate::codebase::postgres::ImportedSite;
use crate::codebase::postgres::{EmbeddedSqlOptions, RecoveredSql, SiteArg, SiteKind};
use std::path::{Path, PathBuf};

/// Expand a proven frozen identifier or a scalar record's parameterized member.
pub(super) fn member<R: FnMut(&str, &Path) -> Option<PathBuf>>(
    shared: &mut Shared<'_, R>,
    options: &EmbeddedSqlOptions,
    from: &Path,
    site: &ImportedSite,
) -> Option<Filled> {
    let field = site.member.as_deref()?;
    let (origin, local) = shared.find(from, &site.local, None)?;
    if origin == from {
        return None;
    }
    let program = helper_program(shared, options, &origin, &local)?;
    let Some(index) = program.frozen_key else {
        return (site.kind == SiteKind::ParameterizedMember
            && program.scalar_member(field, &site.args))
        .then(|| {
            let sql = crate::codebase::postgres::embedded::internal_bind(1);
            Filled {
                texts: vec![RecoveredSql {
                    origins: vec![0; sql.len()],
                    sql,
                    line: site.line,
                    positions: Vec::new(),
                    invocations: Default::default(),
                }],
                contributes_sql: false,
            }
        });
    };
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
    (!texts.is_empty()).then_some(Filled {
        texts,
        contributes_sql: true,
    })
}
