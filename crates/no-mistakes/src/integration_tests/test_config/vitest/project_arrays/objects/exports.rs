use super::super::{Ctx, ImportBinding, Options};
use super::imported_options;
use anyhow::Result;
use oxc_ast::ast::{Program, Statement};

pub(super) fn exported_star_options(
    program: &Program<'_>,
    exported: &str,
    parent: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let mut resolved = None;
    for statement in &program.body {
        let Statement::ExportAllDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() || export.exported.is_some() {
            continue;
        }
        let import = ImportBinding {
            source: export.source.value.to_string(),
            imported: exported.to_string(),
        };
        let Some(options) = imported_options(&import, parent)? else {
            continue;
        };
        if resolved.is_some() {
            return Ok(None);
        }
        resolved = Some(options);
    }
    Ok(resolved)
}
