use super::*;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_span::GetSpan;

mod exports;
mod imports;
mod loads;

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
            AstKind::ImportDeclaration(value) if root_module => imports::collect(value, facts),
            AstKind::TSImportType(value) => facts.imports.push(ModuleImport {
                specifier: value.source.value.to_string(),
                type_only: true,
                span: value.span.into(),
                bindings: Vec::new(),
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
            AstKind::ImportExpression(value) => loads::dynamic(value, facts),
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
