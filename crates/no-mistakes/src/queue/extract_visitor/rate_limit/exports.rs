use super::QueueVisitor;
use crate::ast::js_literal;
use crate::queue::extract_helpers::{binding_name, export_name};
use crate::queue::extract_model::RateLimitExport;
use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, ImportDeclarationSpecifier, Program, Statement,
    VariableDeclaration, VariableDeclarationKind,
};
use std::collections::HashMap;

#[path = "exports_classify.rs"]
mod classify;
use classify::{classify_class, classify_expr, lookup_local};

pub(super) struct ExportIndex {
    pub(super) named_imports: HashMap<String, (String, String)>,
    pub(super) ns_imports: HashMap<String, String>,
    pub(super) locals: HashMap<String, RateLimitExport>,
}

impl QueueVisitor<'_> {
    pub(in crate::queue) fn collect_rate_limit_exports(&mut self, program: &Program<'_>) {
        let mut index = ExportIndex {
            named_imports: HashMap::new(),
            ns_imports: HashMap::new(),
            locals: HashMap::new(),
        };
        for stmt in &program.body {
            match stmt {
                Statement::ImportDeclaration(decl) if !decl.import_kind.is_type() => {
                    record_imports(&mut index, decl);
                }
                Statement::ClassDeclaration(class) => {
                    if let Some(id) = &class.id {
                        let kind = classify_class(class, &index);
                        index.locals.insert(id.name.to_string(), kind);
                    }
                }
                Statement::FunctionDeclaration(func) => {
                    if let Some(id) = &func.id {
                        index
                            .locals
                            .insert(id.name.to_string(), RateLimitExport::NonQueue);
                    }
                }
                Statement::VariableDeclaration(decl) => {
                    record_variables(&mut index, decl, None);
                }
                Statement::ExportDeclaration(export) => {
                    record_exported_decl(
                        &mut index,
                        &export.declaration,
                        &mut self.facts.rate_limit_exports,
                    );
                }
                _ => {}
            }
        }
        record_reexports(&index, program, &mut self.facts.rate_limit_exports);
    }
}

fn record_reexports(
    index: &ExportIndex,
    program: &Program<'_>,
    exports: &mut HashMap<String, RateLimitExport>,
) {
    for stmt in &program.body {
        match stmt {
            Statement::ExportNamedDeclaration(decl) if !decl.export_kind.is_type() => {
                for spec in &decl.specifiers {
                    if spec.export_kind.is_type() {
                        continue;
                    }
                    let local = export_name(&spec.local);
                    let exported = export_name(&spec.exported);
                    exports.insert(exported, lookup_local(index, &local));
                }
            }
            Statement::ExportAllDeclaration(decl) if !decl.export_kind.is_type() => {
                if let Some(exported) = &decl.exported {
                    exports.insert(
                        export_name(exported),
                        RateLimitExport::FromNamespace {
                            source: js_literal(&decl.source).into_owned(),
                        },
                    );
                }
            }
            Statement::ExportDefaultDeclaration(decl) => {
                let kind = match &decl.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(_) => {
                        RateLimitExport::NonQueue
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                        classify_class(class, index)
                    }
                    other => other
                        .as_expression()
                        .map_or(RateLimitExport::Opaque, |expr| classify_expr(expr, index)),
                };
                exports.insert("default".to_string(), kind);
            }
            _ => {}
        }
    }
}

fn record_imports(index: &mut ExportIndex, decl: &oxc_ast::ast::ImportDeclaration<'_>) {
    let source = js_literal(&decl.source).into_owned();
    for spec in decl.specifiers.iter().flatten() {
        match spec {
            ImportDeclarationSpecifier::ImportSpecifier(spec) if !spec.import_kind.is_type() => {
                index.named_imports.insert(
                    spec.local.name.to_string(),
                    (source.clone(), export_name(&spec.imported)),
                );
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                index.named_imports.insert(
                    spec.local.name.to_string(),
                    (source.clone(), "default".to_string()),
                );
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                index
                    .ns_imports
                    .insert(spec.local.name.to_string(), source.clone());
            }
            _ => {}
        }
    }
}

fn record_exported_decl(
    index: &mut ExportIndex,
    decl: &Declaration<'_>,
    exports: &mut HashMap<String, RateLimitExport>,
) {
    match decl {
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                let kind = classify_class(class, index);
                index.locals.insert(id.name.to_string(), kind.clone());
                exports.insert(id.name.to_string(), kind);
            }
        }
        Declaration::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                index
                    .locals
                    .insert(id.name.to_string(), RateLimitExport::NonQueue);
                exports.insert(id.name.to_string(), RateLimitExport::NonQueue);
            }
        }
        Declaration::VariableDeclaration(var_decl) => {
            record_variables(index, var_decl, Some(exports));
        }
        _ => {}
    }
}

fn record_variables(
    index: &mut ExportIndex,
    decl: &VariableDeclaration<'_>,
    mut exports: Option<&mut HashMap<String, RateLimitExport>>,
) {
    for declarator in &decl.declarations {
        let Some(name) = binding_name(&declarator.id) else {
            continue;
        };
        let kind = if decl.kind == VariableDeclarationKind::Const {
            declarator
                .init
                .as_ref()
                .map_or(RateLimitExport::Opaque, |init| classify_expr(init, index))
        } else {
            RateLimitExport::Opaque
        };
        index.locals.insert(name.clone(), kind.clone());
        if let Some(out) = exports.as_deref_mut() {
            out.insert(name, kind);
        }
    }
}
