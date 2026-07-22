use super::super::super::{root_spreads, ImportBinding};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrayExpression, Declaration, ExportDefaultDeclarationKind, Expression, FunctionBody, Program,
    Statement,
};
use std::collections::BTreeMap;

pub(super) fn exported_array<'a>(
    program: &'a Program<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    exported: &str,
) -> Option<&'a ArrayExpression<'a>> {
    exported_expression(program, bindings, exported)
        .and_then(|expression| match unwrap_ts_wrappers(expression) {
            Expression::ArrayExpression(array) => Some(array),
            _ => None,
        })
        .map(|array| &**array)
}

pub(super) fn reexported_imports(
    program: &Program<'_>,
    imports: &BTreeMap<String, ImportBinding>,
    exported: &str,
) -> Vec<ImportBinding> {
    let mut reexports = Vec::new();
    if let Some(import) = root_spreads::sourced_reexport(program, exported) {
        reexports.push(import);
    }
    if let Some(import) = root_spreads::imported_reexport(program, exported) {
        reexports.push(import);
    }
    if exported == "default" {
        reexports.extend(program.body.iter().filter_map(|statement| {
            let Statement::ExportDefaultDeclaration(export) = statement else {
                return None;
            };
            let Expression::Identifier(identifier) = export.declaration.as_expression()? else {
                return None;
            };
            imports.get(identifier.name.as_str()).cloned()
        }));
    }
    reexports.extend(
        root_spreads::star_barrel_sources(program).map(|source| ImportBinding {
            source: source.to_string(),
            imported: exported.to_string(),
        }),
    );
    reexports
}

pub(super) fn exported_expression<'a>(
    program: &'a Program<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    exported: &str,
) -> Option<&'a Expression<'a>> {
    if exported == "default" {
        return program.body.iter().find_map(|statement| {
            let Statement::ExportDefaultDeclaration(export) = statement else {
                return None;
            };
            let expression = export.declaration.as_expression()?;
            match expression {
                Expression::Identifier(identifier) => {
                    bindings.get(identifier.name.as_str()).copied()
                }
                _ => Some(expression),
            }
        });
    }
    program.body.iter().find_map(|statement| {
        let Statement::ExportNamedDeclaration(export) = statement else {
            return None;
        };
        if export.export_kind.is_type() || export.source.is_some() {
            return None;
        }
        if let Some(Declaration::VariableDeclaration(declaration)) = &export.declaration {
            for declarator in &declaration.declarations {
                let oxc_ast::ast::BindingPattern::BindingIdentifier(identifier) = &declarator.id
                else {
                    continue;
                };
                if identifier.name == exported {
                    return declarator.init.as_ref();
                }
            }
        }
        export.specifiers.iter().find_map(|specifier| {
            (!specifier.export_kind.is_type() && specifier.exported.name() == exported)
                .then(|| bindings.get(specifier.local.name().as_str()).copied())
                .flatten()
        })
    })
}

pub(super) fn exported_function_body<'a>(
    program: &'a Program<'a>,
    exported: &str,
) -> Option<&'a FunctionBody<'a>> {
    let functions = super::super::super::top_level_function_bodies(program);
    if exported == "default" {
        if let Some(Statement::ExportDefaultDeclaration(export)) = program
            .body
            .iter()
            .find(|statement| matches!(statement, Statement::ExportDefaultDeclaration(_)))
        {
            match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                    return function.body.as_deref();
                }
                ExportDefaultDeclarationKind::Identifier(identifier) => {
                    return functions.get(identifier.name.as_str()).copied();
                }
                _ => {}
            }
        }
    }
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() || export.source.is_some() {
            continue;
        }
        if let Some(Declaration::FunctionDeclaration(function)) = &export.declaration {
            if function
                .id
                .as_ref()
                .is_some_and(|identifier| identifier.name == exported)
            {
                return function.body.as_deref();
            }
        }
        for specifier in &export.specifiers {
            if !specifier.export_kind.is_type() && specifier.exported.name() == exported {
                return functions.get(specifier.local.name().as_str()).copied();
            }
        }
    }
    None
}
