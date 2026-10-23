use oxc_ast::ast::Program;
use oxc_semantic::SemanticBuilder;
use oxc_span::GetSpan;

pub(crate) mod arguments;
mod modules;
mod report;
pub use report::*;
mod types;
pub use types::*;

pub(crate) fn extract_with_arguments(
    program: &Program<'_>,
    collect: bool,
) -> TypeScriptModuleFacts {
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .with_check_syntax_error(true)
        .build(program);
    extract_from_semantic(&built.semantic, &built.diagnostics, collect)
}

pub(crate) fn extract_from_semantic(
    semantic: &oxc_semantic::Semantic<'_>,
    diagnostics: &[oxc_diagnostics::OxcDiagnostic],
    collect: bool,
) -> TypeScriptModuleFacts {
    let scoping = semantic.scoping();
    let mut facts = TypeScriptModuleFacts::default();
    for diagnostic in diagnostics {
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
    modules::collect(semantic, &mut facts);
    if collect && facts.diagnostics.is_empty() {
        facts.call_arguments = Some(std::sync::Arc::new(arguments::collect(semantic, &facts)));
    }
    facts
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod argument_tests;

#[cfg(test)]
mod argument_matrix_tests;
