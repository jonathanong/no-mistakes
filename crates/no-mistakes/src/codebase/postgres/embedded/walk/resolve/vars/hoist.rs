use super::super::super::super::EmbeddedSqlKind;
use super::super::{for_each_bound_name, BindingState, ScopeVisitor};
use oxc_ast::ast::{
    ArrowFunctionExpression, Class, Function, Statement, VariableDeclaration,
    VariableDeclarationKind,
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
                executor_alias: None,
                builder_identity: None,
                builder_effect_revision: 0,
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
