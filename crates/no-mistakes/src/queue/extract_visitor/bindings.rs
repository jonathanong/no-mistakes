use super::QueueVisitor;
use crate::ast::js_literal;
use crate::queue::extract_helpers::{binding_name, literal_expr, static_string};
use crate::queue::extract_job_id;
use oxc_ast::ast::{Argument, Expression, VariableDeclarator};
use oxc_ast_visit::walk;

impl<'a> QueueVisitor<'a> {
    pub(super) fn note_const_declaration(&mut self, decl: &oxc_ast::ast::VariableDeclaration<'a>) {
        let previous = self.binding_is_const;
        self.binding_is_const = decl.kind == oxc_ast::ast::VariableDeclarationKind::Const;
        self.note_wait_aliases(decl);
        super::super::extract_factory::visit_variable_declaration(self, decl);
        self.binding_is_const = previous;
    }

    pub(super) fn note_variable(&mut self, decl: &VariableDeclarator<'a>) {
        let Some(name) = binding_name(&decl.id) else {
            walk::walk_variable_declarator(self, decl);
            return;
        };
        if let Some(init) = &decl.init {
            if let Some(value) = static_string(init) {
                self.const_strings.insert(name.clone(), value);
            }
            if self.binding_is_const {
                let value =
                    crate::queue::extract_priority::const_u32(init, &self.facts.const_numbers);
                if let Some(value) = value {
                    self.facts.const_numbers.insert(name.clone(), value);
                }
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
                    if extract_job_id::is_imported_callee(self, &new_expr.callee) {
                        extract_job_id::note_queue_binding(self, &decl.id);
                    }
                }
            } else if self.is_flow_constructor(&new_expr.callee) {
                self.flow_bindings.insert(name.clone());
                if extract_job_id::is_imported_callee(self, &new_expr.callee) {
                    extract_job_id::note_flow_binding(self, &decl.id);
                }
            }
        }
        if let Some(Expression::CallExpression(call_expr)) = &decl.init {
            if super::super::extract_factory::note_queue_binding(self, &name, call_expr)
                && extract_job_id::is_imported_callee(self, &call_expr.callee)
            {
                extract_job_id::note_queue_binding(self, &decl.id);
            }
        }
        if let Some(Expression::Identifier(id)) = &decl.init {
            super::super::extract_factory::note_alias(self, &name, id.name.as_str());
            if extract_job_id::queue_identifier(self, id) {
                extract_job_id::note_queue_binding(self, &decl.id);
            }
        }
        walk::walk_variable_declarator(self, decl);
    }
}

impl QueueVisitor<'_> {
    pub fn literal_arg(&self, arg: &Argument<'_>) -> Option<String> {
        match arg {
            Argument::StringLiteral(value) => Some(js_literal(value).into_owned()),
            Argument::Identifier(id) => self.const_strings.get(id.name.as_str()).cloned(),
            _ => arg
                .as_expression()
                .and_then(|expr| literal_expr(expr, &self.const_strings)),
        }
    }
}
