use oxc_ast::ast::Program;
use oxc_semantic::SemanticBuilder;
use oxc_span::GetSpan;

mod modules;
mod report;
pub use report::*;
mod types;
pub use types::*;

/// Project source-local semantic facts without exposing parser or semantic ASTs.
pub(crate) fn extract(program: &Program<'_>) -> TypeScriptModuleFacts {
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .with_check_syntax_error(true)
        .build(program);
    let semantic = built.semantic;
    let scoping = semantic.scoping();
    let mut facts = TypeScriptModuleFacts::default();
    for diagnostic in built.diagnostics {
        facts.diagnostics.push(ModuleDiagnostic {
            kind: "semanticError".into(),
            message: diagnostic.to_string(),
            span: None,
        });
    }
    for id in scoping.symbol_ids() {
        let scope = scoping.symbol_scope_id(id);
        let name = scoping.symbol_name(id);
        let flags = scoping.symbol_flags(id);
        if !scoping.symbol_redeclarations(id).is_empty() {
            facts.diagnostics.push(ModuleDiagnostic {
                kind: "unsupported".into(),
                message: "Merged binding declarations are unsupported".into(),
                span: Some(scoping.symbol_span(id).into()),
            });
        }
        let shadows = scoping
            .scope_parent_id(scope)
            .and_then(|parent| scoping.find_binding(parent, scoping.symbol_ident(id)))
            .map(|parent| parent.index());
        let references = scoping
            .get_resolved_references(id)
            .map(|reference| {
                let flags = reference.flags();
                // `typeof Value` is a type query, not a runtime evaluation.
                BindingReference {
                    span: semantic.nodes().kind(reference.node_id()).span().into(),
                    runtime: flags.is_value() && !flags.is_value_as_type(),
                    type_only: flags.is_type() || flags.is_value_as_type(),
                }
            })
            .collect();
        facts.bindings.push(ModuleBinding {
            id: id.index(),
            name: name.into(),
            scope_id: scope.index(),
            span: scoping.symbol_span(id).into(),
            imported: flags.is_import(),
            runtime: (flags.is_value() || (flags.is_import() && !flags.is_type_import()))
                && !flags.is_ambient(),
            type_only: !flags.is_value() && (!flags.is_import() || flags.is_type_import()),
            shadows,
            references,
        });
    }
    facts.scopes = scoping
        .scope_descendants_from_root()
        .map(|id| ModuleScope {
            id: id.index(),
            parent_id: scoping.scope_parent_id(id).map(|parent| parent.index()),
        })
        .collect();
    modules::collect(&semantic, &mut facts);
    facts
}

#[cfg(test)]
mod tests;
