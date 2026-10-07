use super::*;
pub(super) fn from(value: &ExportFromDeclaration<'_>, facts: &mut TypeScriptModuleFacts) {
    if value.specifiers.is_empty() {
        facts.exports.push(ModuleExport {
            specifier: Some(value.source.value.to_string()),
            local: String::new(),
            exported: String::new(),
            type_only: value.export_kind.is_type(),
            span: value.span.into(),
        });
    }
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
pub(super) fn named(
    value: &ExportNamedDeclaration<'_>,
    semantic: &Semantic<'_>,
    facts: &mut TypeScriptModuleFacts,
) {
    for specifier in &value.specifiers {
        facts.exports.push(ModuleExport {
            specifier: None,
            local: name(&specifier.local),
            exported: name(&specifier.exported),
            type_only: value.export_kind.is_type()
                || specifier.export_kind.is_type()
                || local_type_only(&specifier.local, semantic, facts),
            span: specifier.span.into(),
        });
    }
}

fn local_type_only(
    local: &ModuleExportName<'_>,
    semantic: &Semantic<'_>,
    facts: &TypeScriptModuleFacts,
) -> bool {
    let ModuleExportName::IdentifierReference(local) = local else {
        return false;
    };
    semantic
        .scoping()
        .get_reference(local.reference_id.get().expect("semantic export reference"))
        .symbol_id()
        .and_then(|id| facts.bindings.get(id.index()))
        .is_some_and(|binding| binding.type_only)
}

pub(super) fn inline(
    value: &ExportDeclaration<'_>,
    inline_bindings: &[usize],
    facts: &mut TypeScriptModuleFacts,
) {
    let span = value.declaration.span();
    let candidates = inline_candidates(inline_bindings, &facts.bindings, span);
    for &index in candidates {
        let binding = &facts.bindings[index];
        if binding.span.end <= span.end {
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

fn inline_candidates<'a>(
    indices: &'a [usize],
    bindings: &[ModuleBinding],
    span: oxc_span::Span,
) -> &'a [usize] {
    let start = indices.partition_point(|&index| bindings[index].span.start < span.start);
    let end = indices.partition_point(|&index| bindings[index].span.start < span.end);
    &indices[start..end]
}

pub(super) fn default(
    value: &ExportDefaultDeclaration<'_>,
    semantic: &Semantic<'_>,
    facts: &mut TypeScriptModuleFacts,
) {
    let (local, type_only) = match &value.declaration {
        ExportDefaultDeclarationKind::Identifier(value) => (default_local(value, semantic), false),
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
        _ => match value
            .declaration
            .as_expression()
            .map(Expression::get_inner_expression)
        {
            Some(Expression::Identifier(identifier)) => {
                (default_local(identifier, semantic), false)
            }
            _ => (String::new(), false),
        },
    };
    facts.exports.push(ModuleExport {
        specifier: None,
        local,
        exported: "default".into(),
        type_only,
        span: value.span.into(),
    });
}

fn default_local(identifier: &IdentifierReference<'_>, semantic: &Semantic<'_>) -> String {
    if semantic
        .scoping()
        .get_reference(
            identifier
                .reference_id
                .get()
                .expect("semantic default export"),
        )
        .symbol_id()
        .is_some()
    {
        identifier.name.to_string()
    } else {
        String::new()
    }
}

pub(super) fn commonjs_reference(
    value: &IdentifierReference<'_>,
    node_id: oxc_semantic::NodeId,
    semantic: &Semantic<'_>,
    facts: &mut TypeScriptModuleFacts,
) {
    if value.name != "module" && value.name != "exports" {
        return;
    }
    let reference = semantic.scoping().get_reference(
        value
            .reference_id
            .get()
            .expect("semantic CommonJS reference"),
    );
    if !reference.is_value()
        || reference.flags().is_value_as_type()
        || loads::has_runtime_binding(reference, semantic)
    {
        return;
    }
    if value.name == "module" {
        let property = match loads::unwrapped_parent(node_id, semantic) {
            Some(AstKind::StaticMemberExpression(member)) => Some(member.property.name.as_str()),
            Some(AstKind::ComputedMemberExpression(member)) => {
                member.static_property_name().map(|name| name.as_str())
            }
            _ => None,
        };
        if property.is_some_and(|name| name != "exports") {
            return;
        }
    }
    unsupported(
        facts,
        value.span,
        "CommonJS module/export objects are unsupported",
    );
}

#[cfg(test)]
mod tests;
