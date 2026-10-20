use super::super::super::EmbeddedSqlKind;
use super::compose::classify_init;
use super::{for_each_bound_name, BindingState, ScopeVisitor};
use oxc_ast::ast::{
    BindingPattern, VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
};
use oxc_span::GetSpan;

mod hoist;
mod namespace;
pub(in crate::codebase::postgres::embedded::walk) use hoist::hoist_vars;
pub(crate) use hoist::hoisted_names;
pub(in crate::codebase::postgres::embedded::walk) use namespace::bind_query_property;

pub(in crate::codebase::postgres::embedded::walk) fn initialize_vars(
    declaration: &VariableDeclaration<'_>,
    visitor: &mut ScopeVisitor<'_>,
) {
    if declaration.kind == VariableDeclarationKind::Var {
        for declarator in &declaration.declarations {
            record_declarator(declarator, false, true, visitor);
        }
    }
}

pub(super) fn record_declarator(
    declarator: &VariableDeclarator<'_>,
    is_const: bool,
    is_var: bool,
    visitor: &mut ScopeVisitor<'_>,
) {
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
        // Destructuring has no single SQL init to classify; bind every name
        // as a shadow so a nested `const { tag } = …` cannot keep a trusted
        // imported tag alias.
        if is_var {
            invalidate_vars(&declarator.id, visitor);
        } else {
            visitor.bind_param(&declarator.id, false);
        }
        return;
    };
    let Some(init) = &declarator.init else {
        if !is_var {
            visitor.bind_self_name(ident.name.as_str());
        }
        return;
    };
    let line =
        crate::codebase::ts_source::byte_offset_to_line(visitor.source, ident.span.start as usize);
    let (sql, kind) = classify_init(init, is_const, visitor);
    // The helper returns its input builder. A stored result still aliases the
    // caller's binding, which may be appended later before execution.
    let kind = if super::compose::contains_parameter_helper(init, visitor) {
        EmbeddedSqlKind::Dynamic
    } else {
        kind
    };
    let sql_source_positions = super::compose::helper_provenance(init, visitor)
        .filter(|provenance| sql.as_deref() == Some(provenance.sql.as_str()))
        .map(|provenance| provenance.positions)
        .unwrap_or_else(|| {
            super::super::super::source_positions::for_expression(
                init,
                visitor.source,
                ident.span.start as usize,
                line,
            )
        });
    let builder_identity = visitor.builder_identity_for_init(init);
    let namespace_executor = is_const && visitor.namespace_expression(init);
    let relative_executor_proof = visitor
        .relative_declaration_proof(ident.name.as_str(), declarator.id.span().start)
        .or_else(|| {
            is_const
                .then(|| visitor.relative_executor_proof(init))
                .flatten()
        });
    let condition_key = is_const.then(|| visitor.declaration_choice_key(init, ident.span.start));
    let scope_index = if is_var {
        var_scope(visitor, ident.name.as_str())
    } else {
        visitor.scopes.len().saturating_sub(1)
    };
    if let Some(scope) = visitor.scopes.get_mut(scope_index) {
        if is_var
            && scope
                .get(ident.name.as_str())
                .is_some_and(|binding| binding.initialized)
        {
            // Conflicting initializer sites do not identify one statement.
            let binding = scope.get_mut(ident.name.as_str()).unwrap();
            clear_binding(binding);
            return;
        }
        scope.insert(
            ident.name.to_string(),
            BindingState {
                executor_alias: if visitor
                    .scoped
                    .contains_declaration(ident.name.as_str(), declarator.id.span().start)
                {
                    Some(true)
                } else {
                    relative_executor_proof
                        .map(|_| false)
                        .or(is_const.then_some(false))
                },
                builder_identity,
                builder_effect_revision: 0,
                condition_key,
                variants: None,
                // A bare imported call is not yet known to be SQL, so it must not
                // make `value.append(...)` look like a builder mutation.
                sql_builder: sql
                    .as_deref()
                    .is_some_and(|text| !super::super::super::helpers::sentinel::is_bare(text))
                    && matches!(
                        kind,
                        EmbeddedSqlKind::ImmutableLocal | EmbeddedSqlKind::Composed
                    ),
                sql,
                kind,
                line,
                initialized: true,
                sql_source_positions,
            },
        );
    }
    if let Some(index) = relative_executor_proof {
        visitor.relative_proofs[scope_index].insert(ident.name.to_string(), index);
    }
    if namespace_executor {
        visitor.namespace_proofs[scope_index].insert(ident.name.to_string());
    }
    if is_const {
        visitor.note_const_object(init, scope_index, ident.name.as_str());
        visitor.note_text_const(init, scope_index, ident.name.as_str());
        super::super::imported::constants::note_fragment_const(
            visitor,
            init,
            scope_index,
            ident.name.as_str(),
        );
    }
}

pub(super) fn invalidate_vars(pattern: &BindingPattern<'_>, visitor: &mut ScopeVisitor<'_>) {
    for_each_bound_name(pattern, &mut |name| {
        let index = var_scope(visitor, name);
        if let Some(binding) = visitor.scopes[index].get_mut(name) {
            clear_binding(binding);
        }
    });
}

fn var_scope(visitor: &ScopeVisitor<'_>, name: &str) -> usize {
    let owner = visitor.function_scopes.last().copied().unwrap_or(0);
    // A simple catch parameter may legally share a hoisted var name. Its
    // initializer assigns the catch binding, leaving the outer var alone.
    (owner..visitor.scopes.len())
        .rev()
        .find(|&index| visitor.scopes[index].contains_key(name))
        .unwrap_or(owner)
}

fn clear_binding(binding: &mut BindingState) {
    binding.executor_alias = None;
    binding.initialized = true;
    binding.sql = None;
    binding.sql_builder = false;
    binding.sql_source_positions.clear();
    binding.condition_key = None;
    binding.variants = None;
    binding.builder_identity = None;
    binding.kind = EmbeddedSqlKind::Dynamic;
}
