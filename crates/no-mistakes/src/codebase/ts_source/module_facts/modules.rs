use super::*;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_span::GetSpan;

mod exports;
mod imports;
mod loads;
mod mocks;
mod selectors;

pub(super) fn collect(semantic: &Semantic<'_>, facts: &mut TypeScriptModuleFacts) {
    let mut inline_bindings = facts
        .bindings
        .iter()
        .enumerate()
        .filter(|(_, binding)| binding.scope_id == semantic.scoping().root_scope_id().index())
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    inline_bindings.sort_unstable_by_key(|&index| facts.bindings[index].span.start);
    for node in semantic.nodes().iter() {
        let root_module = matches!(semantic.nodes().parent_kind(node.id()), AstKind::Program(_));
        match node.kind() {
            AstKind::ImportDeclaration(value) if root_module => {
                imports::collect(value, semantic, facts)
            }
            AstKind::TSImportType(value) => facts.imports.push(ModuleImport {
                specifier: crate::ast::js_literal(&value.source).into_owned(),
                type_only: true,
                span: value.span.into(),
                bindings: Vec::new(),
                qualifier: imports::qualifier_root(value),
            }),
            AstKind::ExportFromDeclaration(value) if root_module => exports::from(value, facts),
            AstKind::ExportAllDeclaration(value) if root_module => exports::all(value, facts),
            AstKind::ExportNamedDeclaration(value) if root_module => {
                exports::named(value, semantic, facts)
            }
            AstKind::ExportDeclaration(value) if root_module => {
                exports::inline(value, &inline_bindings, facts)
            }
            AstKind::ExportDefaultDeclaration(value) if root_module => {
                exports::default(value, semantic, facts)
            }
            AstKind::ImportExpression(value) => {
                if mocks::typed_mock_import(value, node.id(), semantic) {
                    facts.mock_import_spans.push(value.span.into());
                }
                if let Some(names) = selectors::proven(node.id(), semantic) {
                    facts
                        .dynamic_import_selectors
                        .push((value.span.start, names));
                }
                loads::dynamic(value, facts);
            }
            AstKind::CallExpression(value) => loads::require(value, semantic, facts),
            AstKind::IdentifierReference(value) => {
                loads::indirect(value, node.id(), semantic, facts);
                exports::commonjs_reference(value, node.id(), semantic, facts)
            }
            AstKind::TSExternalModuleDeclaration(_)
            | AstKind::TSNamespaceDeclaration(_)
            | AstKind::TSImportEqualsDeclaration(_)
            | AstKind::TSExportAssignment(_)
            | AstKind::TSNamespaceExportDeclaration(_)
            | AstKind::WithStatement(_) => {
                unsupported(
                    facts,
                    node.kind().span(),
                    "Unsupported module or dynamic binding form",
                );
            }
            _ => {}
        }
    }
}
fn name(value: &ModuleExportName<'_>) -> String {
    match value {
        ModuleExportName::IdentifierName(value) => value.name.to_string(),
        ModuleExportName::IdentifierReference(value) => value.name.to_string(),
        ModuleExportName::StringLiteral(value) => crate::ast::js_literal(value).into_owned(),
    }
}
fn unsupported(facts: &mut TypeScriptModuleFacts, span: oxc_span::Span, message: &str) {
    facts.diagnostics.push(ModuleDiagnostic {
        kind: "unsupported".into(),
        message: message.into(),
        span: Some(span.into()),
    });
}
