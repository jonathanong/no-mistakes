use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::Expression;
use oxc_ast::ast::{
    AwaitExpression, CallExpression, Declaration, ExportDeclaration, ExportNamedDeclaration,
    IdentifierReference, Program,
};
use oxc_ast_visit::{walk, Visit};

#[derive(Default)]
struct Uses {
    completed: FxHashSet<u32>,
    direct: FxHashSet<u32>,
    escaped: FxHashSet<String>,
}
impl<'a> Visit<'a> for Uses {
    fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
        for specifier in &export.specifiers {
            self.escaped
                .insert(crate::ast::module_export_name(&specifier.local).into_owned());
        }
        walk::walk_export_named_declaration(self, export);
    }
    fn visit_export_declaration(&mut self, export: &ExportDeclaration<'a>) {
        if let Declaration::FunctionDeclaration(function) = &export.declaration {
            if let Some(name) = &function.id {
                self.escaped.insert(name.name.to_string());
            }
        }
        walk::walk_export_declaration(self, export);
    }
    fn visit_await_expression(&mut self, expression: &AwaitExpression<'a>) {
        if let Expression::CallExpression(call) = unwrap_ts_wrappers(&expression.argument) {
            self.completed.insert(call.span.start);
        }
        walk::walk_await_expression(self, expression);
    }
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if !call.optional
            && self.completed.contains(&call.span.start)
            && call
                .arguments
                .iter()
                .all(|argument| argument.as_expression().is_some())
        {
            if let Expression::Identifier(id) = unwrap_ts_wrappers(&call.callee) {
                self.direct.insert(id.span.start);
            }
        }
        walk::walk_call_expression(self, call);
    }
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        if !self.direct.contains(&id.span.start) {
            self.escaped.insert(id.name.to_string());
        }
    }
}
pub(super) fn names(program: &Program<'_>) -> FxHashSet<String> {
    let mut uses = Uses::default();
    uses.visit_program(program);
    uses.escaped
}
