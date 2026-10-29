use super::*;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_span::GetSpan;

mod exports;
mod imports;
pub(super) mod loads;
mod mock_boundaries;
mod mocks;
mod selectors;
mod type_selection;

pub(super) fn collect(semantic: &Semantic<'_>, facts: &mut TypeScriptModuleFacts, arguments: bool) {
    let mut node_loaders = super::node_loaders::Collector::new(semantic);
    let mut assignments = Vec::new();
    let mut inline_bindings = facts
        .bindings
        .iter()
        .enumerate()
        .filter(|(_, binding)| binding.scope_id == semantic.scoping().root_scope_id().index())
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    inline_bindings.sort_unstable_by_key(|&index| facts.bindings[index].span.start);
    let mut wildcards = Vec::new();
    let mut references = Vec::new();
    for node in semantic.nodes().iter() {
        let root_module = matches!(semantic.nodes().parent_kind(node.id()), AstKind::Program(_));
        if matches!(
            node.kind(),
            AstKind::ImportDeclaration(_)
                | AstKind::ExportFromDeclaration(_)
                | AstKind::ExportAllDeclaration(_)
                | AstKind::TSImportType(_)
                | AstKind::TSImportEqualsDeclaration(_)
        ) {
            references.push(node.kind().span().start);
        }
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
                picked: type_selection::selected(value, node.id(), semantic),
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
            AstKind::CallExpression(value) => {
                mock_boundaries::collect(value, node.id(), semantic, facts);
                loads::require(value, semantic, &mut node_loaders, facts);
                if arguments {
                    node_loaders.collect(value, facts);
                }
            }
            AstKind::IdentifierReference(value) => {
                loads::indirect(value, node.id(), semantic, facts);
                loads::created_escape(value, node.id(), semantic, &mut node_loaders, facts);
                exports::commonjs_reference(value, node.id(), semantic, &mut assignments, facts)
            }
            AstKind::TSExternalModuleDeclaration(value)
                if root_module && value.declare && value.id.value.contains('*') =>
            {
                unsupported(facts, value.span, UNSUPPORTED_FORM);
                wildcards.push(value.span);
            }
            AstKind::TSExternalModuleDeclaration(_)
            | AstKind::TSNamespaceDeclaration(_)
            | AstKind::TSImportEqualsDeclaration(_)
            | AstKind::TSExportAssignment(_)
            | AstKind::TSNamespaceExportDeclaration(_)
            | AstKind::WithStatement(_) => {
                unsupported(facts, node.kind().span(), UNSUPPORTED_FORM);
            }
            _ => {}
        }
    }
    exports::finish_commonjs(assignments, facts);
    for span in wildcards {
        if !references
            .iter()
            .any(|at| span.start <= *at && *at < span.end)
        {
            facts.importer_safe.push((span.start, span.end));
        }
    }
}

pub(super) fn collect_require_bindings(semantic: &Semantic<'_>, facts: &mut TypeScriptModuleFacts) {
    let loads: crate::fx::FxHashSet<_> = facts
        .loads
        .iter()
        .filter(|load| load.kind == "require")
        .map(|load| load.span.start)
        .collect();
    let mut bindings: Vec<_> = facts
        .bindings
        .iter()
        .filter(|binding| binding.runtime)
        .collect();
    bindings.sort_unstable_by_key(|binding| binding.span.start);
    for node in semantic.nodes().iter() {
        let AstKind::VariableDeclarator(declaration) = node.kind() else {
            continue;
        };
        let Some(init) = declaration.init.as_ref() else {
            continue;
        };
        let mut expression = init.get_inner_expression();
        while let Expression::StaticMemberExpression(member) = expression {
            expression = member.object.get_inner_expression();
        }
        let Expression::CallExpression(call) = expression else {
            continue;
        };
        if !loads.contains(&call.span.start) {
            continue;
        }
        let id = declaration.id.span();
        let mut named = crate::fx::FxHashMap::default();
        if let BindingPattern::ObjectPattern(object) = &declaration.id {
            for property in &object.properties {
                if let BindingPattern::BindingIdentifier(local) = &property.value {
                    if !property.computed {
                        if let Some(imported) = crate::ast::property_key_name(&property.key) {
                            named.insert(local.name.as_str(), imported.into_owned());
                        }
                    }
                }
            }
        }
        let first = bindings.partition_point(|binding| binding.span.start < id.start);
        for binding in bindings[first..]
            .iter()
            .take_while(|binding| binding.span.start < id.end)
        {
            if binding.span.end <= id.end {
                facts.require_bindings.push(RequireBinding {
                    local: binding.name.clone(),
                    load_start: call.span.start,
                    scope_id: binding.scope_id,
                    imported: named.get(binding.name.as_str()).cloned(),
                });
            }
        }
    }
}
pub(super) fn name(value: &ModuleExportName<'_>) -> String {
    match value {
        ModuleExportName::IdentifierName(value) => value.name.to_string(),
        ModuleExportName::IdentifierReference(value) => value.name.to_string(),
        ModuleExportName::StringLiteral(value) => crate::ast::js_literal(value).into_owned(),
    }
}
const UNSUPPORTED_FORM: &str = "Unsupported module or dynamic binding form";
fn unsupported(facts: &mut TypeScriptModuleFacts, span: oxc_span::Span, message: &str) {
    facts.diagnostics.push(ModuleDiagnostic {
        kind: "unsupported".into(),
        message: message.into(),
        span: Some(span.into()),
    });
}
