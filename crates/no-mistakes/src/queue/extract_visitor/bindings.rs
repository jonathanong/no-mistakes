use super::QueueVisitor;
use crate::queue::extract_helpers::{binding_name, static_string};
use oxc_ast::ast::{Expression, VariableDeclarator};
use oxc_ast_visit::walk;

impl<'a> QueueVisitor<'a> {
    pub(super) fn note_variable(&mut self, decl: &VariableDeclarator<'a>) {
        let Some(name) = binding_name(&decl.id) else {
            walk::walk_variable_declarator(self, decl);
            return;
        };
        if let Some(init) = &decl.init {
            if let Some(value) = static_string(init) {
                self.const_strings.insert(name.clone(), value);
            }
        }
        if let Some(Expression::NewExpression(new_expr)) = &decl.init {
            if self.is_queue_constructor(&new_expr.callee) {
                if let Some(queue_name) = new_expr
                    .arguments
                    .first()
                    .and_then(|arg| self.literal_arg(arg))
                {
                    self.facts
                        .queue_bindings
                        .insert(name.clone(), queue_name.clone());
                    self.facts.queue_exports.insert(name.clone(), queue_name);
                }
            } else if self.is_flow_constructor(&new_expr.callee) {
                self.flow_bindings.insert(name.clone());
            }
        }
        if let Some(Expression::CallExpression(call_expr)) = &decl.init {
            super::super::extract_factory::note_queue_binding(self, &name, call_expr);
        }
        if let Some(Expression::Identifier(id)) = &decl.init {
            super::super::extract_factory::note_alias(self, &name, id.name.as_str());
        }
        walk::walk_variable_declarator(self, decl);
    }
}
