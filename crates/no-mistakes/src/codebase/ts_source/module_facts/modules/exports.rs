use super::*;
pub(super) fn from(value: &ExportFromDeclaration<'_>, facts: &mut TypeScriptModuleFacts) {
    for specifier in &value.specifiers {
        facts.exports.push(ModuleExport {
            specifier: Some(value.source.value.to_string()),
            local: name(&specifier.local),
            exported: name(&specifier.exported),
            type_only: value.export_kind.is_type() || specifier.export_kind.is_type(),
            span: specifier.span.into(),
        });
    }
}
pub(super) fn all(value: &ExportAllDeclaration<'_>, facts: &mut TypeScriptModuleFacts) {
    facts.exports.push(ModuleExport {
        specifier: Some(value.source.value.to_string()),
        local: "*".into(),
        exported: value
            .exported
            .as_ref()
            .map(name)
            .unwrap_or_else(|| "*".into()),
        type_only: value.export_kind.is_type(),
        span: value.span.into(),
    });
}
pub(super) fn named(value: &ExportNamedDeclaration<'_>, facts: &mut TypeScriptModuleFacts) {
    for specifier in &value.specifiers {
        facts.exports.push(ModuleExport {
            specifier: None,
            local: name(&specifier.local),
            exported: name(&specifier.exported),
            type_only: value.export_kind.is_type() || specifier.export_kind.is_type(),
            span: specifier.span.into(),
        });
    }
}

pub(super) fn inline(
    value: &ExportDeclaration<'_>,
    semantic: &Semantic<'_>,
    facts: &mut TypeScriptModuleFacts,
) {
    let span = value.declaration.span();
    for binding in &facts.bindings {
        let scope = semantic
            .scoping()
            .symbol_scope_id(oxc_semantic::SymbolId::from_usize(binding.id));
        if scope == semantic.scoping().root_scope_id()
            && binding.span.start >= span.start
            && binding.span.end <= span.end
        {
            facts.exports.push(ModuleExport {
                specifier: None,
                local: binding.name.clone(),
                exported: binding.name.clone(),
                type_only: binding.type_only,
                span: binding.span,
            });
        }
    }
}
pub(super) fn default(value: &ExportDefaultDeclaration<'_>, facts: &mut TypeScriptModuleFacts) {
    let (local, type_only) = match &value.declaration {
        ExportDefaultDeclarationKind::Identifier(value) => (value.name.to_string(), false),
        ExportDefaultDeclarationKind::FunctionDeclaration(value) => (
            value
                .id
                .as_ref()
                .map(|id| id.name.to_string())
                .unwrap_or_default(),
            false,
        ),
        ExportDefaultDeclarationKind::ClassDeclaration(value) => (
            value
                .id
                .as_ref()
                .map(|id| id.name.to_string())
                .unwrap_or_default(),
            false,
        ),
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(value) => {
            (value.id.name.to_string(), true)
        }
        _ => (String::new(), false),
    };
    facts.exports.push(ModuleExport {
        specifier: None,
        local,
        exported: "default".into(),
        type_only,
        span: value.span.into(),
    });
}
