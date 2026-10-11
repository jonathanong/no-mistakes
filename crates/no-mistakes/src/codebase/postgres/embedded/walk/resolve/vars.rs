use super::compose::classify_init;
use super::{for_each_bound_name, BindingState, EmbeddedSqlKind, ScopeVisitor};
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, Class, Function, Statement, VariableDeclaration,
    VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::HashMap;

/// Reserve var names before visiting statements: a later declaration shadows
/// outer SQL bindings, and names never cross a function boundary.
pub(in crate::codebase::postgres::embedded::walk) fn hoist_vars(
    statements: &[Statement<'_>],
    visitor: &mut ScopeVisitor<'_>,
) {
    let names = hoisted_names(statements);
    let source = visitor.source;
    if let Some(scope) = visitor.current_scope() {
        for (name, position) in names {
            scope.entry(name).or_insert_with(|| BindingState {
                builder_identity: None,
                condition_key: None,
                variants: None,
                sql: None,
                kind: EmbeddedSqlKind::Dynamic,
                line: crate::codebase::ts_source::byte_offset_to_line(source, position as usize),
                initialized: false,
                sql_builder: false,
                sql_source_positions: Vec::new(),
            });
        }
    }
}

pub(crate) fn hoisted_names(statements: &[Statement<'_>]) -> HashMap<String, u32> {
    let mut collector = VarNames::default();
    for statement in statements {
        collector.visit_statement(statement);
    }
    collector.names
}

#[derive(Default)]
struct VarNames {
    names: HashMap<String, u32>,
}

impl<'a> Visit<'a> for VarNames {
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind == VariableDeclarationKind::Var {
            for declarator in &declaration.declarations {
                for_each_bound_name(&declarator.id, &mut |name| {
                    self.names
                        .entry(name.to_string())
                        .or_insert(declarator.span.start);
                });
            }
        }
        walk::walk_variable_declaration(self, declaration);
    }

    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
    fn visit_class(&mut self, _: &Class<'a>) {}
}

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
    let sql_source_positions = super::super::super::source_positions::for_expression(
        init,
        visitor.source,
        ident.span.start as usize,
        line,
    );
    let builder_identity = visitor.builder_identity_for_init(init);
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
                builder_identity,
                condition_key: is_const.then_some((1u64 << 32) | u64::from(ident.span.start)),
                variants: None,
                sql_builder: sql.is_some()
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
    binding.initialized = true;
    binding.sql = None;
    binding.sql_builder = false;
    binding.sql_source_positions.clear();
    binding.condition_key = None;
    binding.variants = None;
    binding.builder_identity = None;
    binding.kind = EmbeddedSqlKind::Dynamic;
}
