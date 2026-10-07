use super::*;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_span::GetSpan;

mod exports;
mod imports;
mod loads;

pub(super) fn collect(semantic: &Semantic<'_>, facts: &mut TypeScriptModuleFacts) {
    for node in semantic.nodes().iter() {
        match node.kind() {
            AstKind::ImportDeclaration(value) => imports::collect(value, facts),
            AstKind::TSImportType(value) => facts.imports.push(ModuleImport {
                specifier: value.source.value.to_string(),
                type_only: true,
                span: value.span.into(),
                bindings: Vec::new(),
            }),
            AstKind::ExportFromDeclaration(value) => exports::from(value, facts),
            AstKind::ExportAllDeclaration(value) => exports::all(value, facts),
            AstKind::ExportNamedDeclaration(value) => exports::named(value, facts),
            AstKind::ExportDeclaration(value) => exports::inline(value, semantic, facts),
            AstKind::ExportDefaultDeclaration(value) => exports::default(value, facts),
            AstKind::ImportExpression(value) => loads::dynamic(value, facts),
            AstKind::CallExpression(value) => loads::require(value, semantic, facts),
            AstKind::IdentifierReference(value) => {
                loads::indirect(value, node.id(), semantic, facts)
            }
            AstKind::TSImportEqualsDeclaration(_)
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
        ModuleExportName::StringLiteral(value) => value.value.to_string(),
    }
}
fn unsupported(facts: &mut TypeScriptModuleFacts, span: oxc_span::Span, message: &str) {
    facts.diagnostics.push(ModuleDiagnostic {
        kind: "unsupported".into(),
        message: message.into(),
        span: Some(span.into()),
    });
}
