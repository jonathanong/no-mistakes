//! Local `const` arrays published with `export { name }`.

use super::super::include_config::includes_in_array;
use super::{missing, SourceList};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, ModuleExportName, Program, Statement,
    VariableDeclaration, VariableDeclarationKind,
};
use oxc_span::GetSpan;

pub(super) fn extract(program: &Program<'_>, source: &str, export_name: &str) -> SourceList {
    if let Some(parsed) = direct_const_export(program, source, export_name) {
        return parsed;
    }
    let Some(local) = named_export_local(program, export_name) else {
        return missing(export_name);
    };
    let Some(init) = const_initializer(program, local) else {
        return missing(export_name);
    };
    array_export(source, export_name, init)
}

fn direct_const_export(
    program: &Program<'_>,
    source: &str,
    export_name: &str,
) -> Option<SourceList> {
    for statement in &program.body {
        let Statement::ExportDeclaration(export) = statement else {
            continue;
        };
        let Declaration::VariableDeclaration(declaration) = &export.declaration else {
            continue;
        };
        if let Some(parsed) = const_export(declaration, source, export_name) {
            return Some(parsed);
        }
    }
    None
}

fn const_export(
    declaration: &VariableDeclaration<'_>,
    source: &str,
    export_name: &str,
) -> Option<SourceList> {
    if declaration.kind != VariableDeclarationKind::Const {
        return None;
    }
    for declarator in &declaration.declarations {
        if binding_name(&declarator.id) != Some(export_name) {
            continue;
        }
        let Some(init) = declarator.init.as_ref() else {
            return Some(missing(export_name));
        };
        return Some(array_export(source, export_name, init));
    }
    None
}

fn named_export_local<'a>(program: &'a Program<'a>, export_name: &str) -> Option<&'a str> {
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() {
            continue;
        }
        for specifier in &export.specifiers {
            if specifier.export_kind.is_type() {
                continue;
            }
            if crate::ast::module_export_name(&specifier.exported) != export_name {
                continue;
            }
            // oxc rewrites a local export name to an identifier reference.
            let ModuleExportName::IdentifierReference(identifier) = &specifier.local else {
                continue;
            };
            return Some(identifier.name.as_str());
        }
    }
    None
}

fn const_initializer<'a>(program: &'a Program<'a>, name: &str) -> Option<&'a Expression<'a>> {
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                if let Some(init) = const_binding(declaration, name) {
                    return Some(init);
                }
            }
            Statement::ExportDeclaration(export) => {
                let Declaration::VariableDeclaration(declaration) = &export.declaration else {
                    continue;
                };
                if let Some(init) = const_binding(declaration, name) {
                    return Some(init);
                }
            }
            _ => {}
        }
    }
    None
}

fn const_binding<'a>(
    declaration: &'a VariableDeclaration<'a>,
    name: &str,
) -> Option<&'a Expression<'a>> {
    if declaration.kind != VariableDeclarationKind::Const {
        return None;
    }
    for declarator in &declaration.declarations {
        if binding_name(&declarator.id) == Some(name) {
            return declarator.init.as_ref();
        }
    }
    None
}

fn array_export(source: &str, export_name: &str, init: &Expression<'_>) -> SourceList {
    let Expression::ArrayExpression(array) = unwrap_ts_wrappers(init) else {
        return SourceList::Unanalyzable {
            line: line_of(source, init.span().start),
            reason: format!("export {export_name} is not a static array of strings"),
        };
    };
    match includes_in_array(source, array.as_ref()) {
        Ok(entries) => SourceList::Includes(entries),
        Err((line, reason)) => SourceList::Unanalyzable { line, reason },
    }
}

fn binding_name<'a>(binding: &'a BindingPattern<'a>) -> Option<&'a str> {
    match binding {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn line_of(source: &str, start: u32) -> usize {
    crate::codebase::ts_source::byte_offset_to_line(source, start as usize) as usize
}
