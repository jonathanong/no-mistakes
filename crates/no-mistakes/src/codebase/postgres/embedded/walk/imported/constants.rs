//! Same-file string consts are inlined. Imported ones reuse the helper hole.

use super::super::const_literals::LiteralObjects;
use super::super::ScopeVisitor;
use crate::codebase::postgres::embedded::helpers::{sentinel, ExportedConstant, ImportedSite};
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::postgres::embedded::EmbeddedSqlKind;
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{
    Declaration, ExportNamedDeclaration, Expression, IdentifierReference, ImportOrExportKind,
    Program, Statement, VariableDeclarationKind,
};

#[path = "default_export.rs"]
mod default_export;
#[path = "names.rs"]
mod names;

use names::{binding_name, local_name};

pub(in crate::codebase::postgres::embedded::walk) fn note_fragment_const(
    visitor: &mut ScopeVisitor<'_>,
    init: &Expression<'_>,
    scope_index: usize,
    name: &str,
) {
    if !matches!(
        unwrap_ts_wrappers(init),
        Expression::TaggedTemplateExpression(_)
    ) {
        return;
    }
    if let Some(names) = visitor.literal_objects.fragment_consts.get_mut(scope_index) {
        names.insert(name.to_string());
    }
}

pub(in crate::codebase::postgres::embedded::walk) fn identifier_piece(
    visitor: &ScopeVisitor<'_>,
    ident: &IdentifierReference<'_>,
) -> Option<String> {
    let name = ident.name.as_str();
    if visitor.literal_objects.is_text_const(&visitor.scopes, name) {
        let binding = visitor.lookup(name)?;
        if binding.kind != EmbeddedSqlKind::ImmutableLocal {
            return None;
        }
        return binding.sql;
    }
    imported_hole(visitor, name, ident.span.start, false)
}

pub(in crate::codebase::postgres::embedded::walk) fn appended_identifier(
    visitor: &ScopeVisitor<'_>,
    ident: &IdentifierReference<'_>,
) -> Option<String> {
    let name = ident.name.as_str();
    if let Some(binding) = visitor.lookup(name) {
        return known_binding_sql(binding.kind, binding.sql);
    }
    imported_hole(visitor, name, ident.span.start, true)
}

pub(in crate::codebase::postgres::embedded::walk) fn snapshot_exports(
    visitor: &mut ScopeVisitor<'_>,
    program: &Program<'_>,
) {
    for statement in &program.body {
        match statement {
            Statement::ExportDeclaration(export) => {
                let Declaration::VariableDeclaration(declaration) = &export.declaration else {
                    continue;
                };
                if declaration.kind != VariableDeclarationKind::Const {
                    continue;
                }
                for declarator in &declaration.declarations {
                    record_export(visitor, binding_name(&declarator.id));
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                record_named(visitor, export);
            }
            Statement::ExportDefaultDeclaration(export) => {
                default_export::record(visitor, export);
            }
            _ => {}
        }
    }
}

fn record_named(visitor: &mut ScopeVisitor<'_>, export: &ExportNamedDeclaration<'_>) {
    if export.export_kind == ImportOrExportKind::Type {
        return;
    }
    for specifier in &export.specifiers {
        if specifier.export_kind == ImportOrExportKind::Type {
            continue;
        }
        record_export(visitor, local_name(&specifier.local));
    }
}

fn record_export(visitor: &mut ScopeVisitor<'_>, name: Option<&str>) {
    let Some(name) = name else {
        return;
    };
    let Some(constant) = exported_constant(visitor, name) else {
        return;
    };
    visitor
        .exported_constants
        .insert(name.to_string(), constant);
}

fn exported_constant(visitor: &ScopeVisitor<'_>, name: &str) -> Option<ExportedConstant> {
    let binding = visitor.lookup(name)?;
    if binding.kind != EmbeddedSqlKind::ImmutableLocal {
        return None;
    }
    let sql = binding.sql.clone()?;
    if sql.contains('\0') || sentinel::contains(&sql) {
        return None;
    }
    let fragment = is_fragment(&visitor.literal_objects, &visitor.scopes, name);
    let text = visitor.literal_objects.is_text_const(&visitor.scopes, name);
    if fragment == text {
        return None;
    }
    Some(ExportedConstant {
        fragment,
        sql: RecoveredSql {
            origins: vec![0; sql.len()],
            sql,
            line: binding.line,
            positions: binding.sql_source_positions.clone(),
        },
    })
}

fn is_fragment(
    objects: &LiteralObjects,
    scopes: &[std::collections::HashMap<String, super::super::BindingState>],
    name: &str,
) -> bool {
    objects
        .fragment_consts
        .iter()
        .zip(scopes)
        .rev()
        .find(|(_, scope)| scope.contains_key(name))
        .is_some_and(|(names, _)| names.contains(name))
}

fn imported_hole(
    visitor: &ScopeVisitor<'_>,
    name: &str,
    start: u32,
    allow_fragment: bool,
) -> Option<String> {
    if visitor.shadowed_locally(name)
        || !visitor.imported_names.contains(name)
        || visitor.functions.imported_sql_tags().contains(name)
    {
        return None;
    }
    let line = byte_offset_to_line(visitor.source, start as usize);
    let index = visitor.imported_sites.borrow_mut().intern(
        start,
        ImportedSite {
            local: name.to_string(),
            args: Vec::new(),
            value_used: true,
            line,
            constant: Some(allow_fragment),
        },
    );
    Some(sentinel::encode(index))
}

fn known_binding_sql(kind: EmbeddedSqlKind, sql: Option<String>) -> Option<String> {
    match kind {
        EmbeddedSqlKind::ImmutableLocal | EmbeddedSqlKind::Composed | EmbeddedSqlKind::Inline => {
            sql
        }
        EmbeddedSqlKind::Dynamic => None,
    }
}

#[cfg(test)]
#[path = "constants_tests.rs"]
mod constants_tests;
