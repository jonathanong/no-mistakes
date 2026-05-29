use super::super::{
    import_bindings, shared, top_level_function_bodies, Ctx, ExprMap, ImportBinding, Options,
};
use super::expression_object;
use crate::ast;
use anyhow::Result;
use oxc_ast::ast::{Declaration, ObjectExpression, Statement};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn imported_object_options(
    import: &ImportBinding,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    imported_object_options_from(import, ctx.path, ctx)
}

fn imported_object_options_from(
    import: &ImportBinding,
    base_path: &Path,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let Some(path) = ctx.resolver.resolve(&import.source, base_path) else {
        return Ok(None);
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(None);
    }
    let result = match std::fs::read_to_string(&path) {
        Err(_) => Ok(None),
        Ok(source) => ast::with_program(&path, &source, |program, source| {
            exported_object_options(program, source, import.imported.as_str(), &path, ctx)
        })
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    result
}

fn exported_object_options(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    exported: &str,
    path: &Path,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let mut export_all_sources = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                if export.export_kind.is_type() {
                    continue;
                }
                for specifier in &export.specifiers {
                    if specifier.export_kind.is_type() || specifier.exported.name() != exported {
                        continue;
                    }
                    if let Some(reexport_source) = &export.source {
                        return imported_object_options_from(
                            &ImportBinding {
                                source: reexport_source.value.to_string(),
                                imported: specifier.local.name().to_string(),
                            },
                            path,
                            ctx,
                        );
                    }
                }
            }
            Statement::ExportAllDeclaration(export)
                if exported != "default"
                    && export.exported.is_none()
                    && !export.export_kind.is_type() =>
            {
                export_all_sources.push(export.source.value.to_string());
            }
            _ => {}
        }
    }
    let bindings = shared::top_level_object_bindings(program);
    if let Some(object) = named_export_object(program, exported, &bindings) {
        let mut local_seen = BTreeSet::new();
        let mut object_seen = BTreeSet::new();
        let mut child = Ctx {
            source,
            bindings,
            functions: top_level_function_bodies(program),
            imports: import_bindings(program),
            resolver: ctx.resolver,
            path,
            seen: ctx.seen,
            local_seen: &mut local_seen,
            object_seen: &mut object_seen,
        };
        return match super::project_object_options(object, &mut child) {
            Ok(options) => Ok(Some(options)),
            Err(_) => Ok(None),
        };
    }
    if let Some(import) = imported_reexport(program, exported) {
        return imported_object_options_from(&import, path, ctx);
    }
    let mut resolved = None;
    for source in export_all_sources {
        let options = imported_object_options_from(
            &ImportBinding {
                source,
                imported: exported.to_string(),
            },
            path,
            ctx,
        )?;
        let Some(options) = options else {
            continue;
        };
        if resolved.is_some() {
            return Ok(None);
        }
        resolved = Some(options);
    }
    Ok(resolved)
}

fn imported_reexport(program: &oxc_ast::ast::Program<'_>, exported: &str) -> Option<ImportBinding> {
    let imports = import_bindings(program);
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() || export.source.is_some() {
            continue;
        }
        for specifier in &export.specifiers {
            if specifier.export_kind.is_type() || specifier.exported.name() != exported {
                continue;
            }
            if let Some(import) = imports.get(specifier.local.name().as_str()) {
                return Some(import.clone());
            }
        }
    }
    None
}

fn named_export_object<'a>(
    program: &'a oxc_ast::ast::Program<'a>,
    exported: &str,
    bindings: &ExprMap<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    if exported == "default" {
        if let Some(object) = shared::default_export_object(program, bindings) {
            return Some(object);
        }
    }
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        let Some(Declaration::VariableDeclaration(declaration)) = &export.declaration else {
            continue;
        };
        for declarator in &declaration.declarations {
            let oxc_ast::ast::BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                continue;
            };
            if identifier.name == exported {
                return expression_object(declarator.init.as_ref()?, bindings);
            }
        }
    }
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.source.is_some() {
            continue;
        }
        for specifier in &export.specifiers {
            if specifier.export_kind.is_type() || specifier.exported.name() != exported {
                continue;
            }
            let local = specifier.local.name();
            if let Some(object) = bindings
                .get(local.as_str())
                .and_then(|expression| expression_object(expression, bindings))
            {
                return Some(object);
            }
        }
    }
    None
}
