use super::QueueVisitor;
use crate::ast::js_literal;
use crate::queue::extract_helpers::{binding_name, literal_expr, static_string};
use crate::queue::extract_job_id;
use crate::queue::extract_model::LocalQueue;
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
                let value = crate::queue::extract_priority::const_nonnegative_integer(
                    init,
                    &self.facts.const_numbers,
                );
                if let Some(value) = value {
                    self.facts.const_numbers.insert(name.clone(), value);
                }
            }
        }
        let declaration = decl.span.end.saturating_sub(1);
        if self.binding_is_const
            && decl
                .init
                .as_ref()
                .is_some_and(|init| self.is_builtin_non_queue_init(init))
        {
            self.non_queue_declarations.insert(declaration);
        }
        if let Some(Expression::NewExpression(new_expr)) = &decl.init {
            if self.is_queue_constructor(&new_expr.callee) {
                if let Some(queue_name) = new_expr
                    .arguments
                    .first()
                    .and_then(|arg| self.literal_arg(arg))
                {
                    self.note_queue_declaration(&name, declaration, LocalQueue::Known(queue_name));
                    if extract_job_id::is_imported_callee(self, &new_expr.callee) {
                        extract_job_id::note_queue_binding(self, &decl.id);
                    }
                } else if let Some(imported) = new_expr
                    .arguments
                    .first()
                    .and_then(crate::queue::extract_factory::identifier_arg)
                    .filter(|ident| self.facts.string_imports.contains_key(ident))
                {
                    self.note_queue_declaration(
                        &name,
                        declaration,
                        LocalQueue::ImportedName(imported),
                    );
                    if extract_job_id::is_imported_callee(self, &new_expr.callee) {
                        extract_job_id::note_queue_binding(self, &decl.id);
                    }
                }
                super::super::job_options::note_constructed_defaults(self, declaration, new_expr);
            } else if self.is_flow_constructor(&new_expr.callee) {
                self.flow_bindings.insert(declaration);
                if extract_job_id::is_imported_callee(self, &new_expr.callee) {
                    extract_job_id::note_flow_binding(self, &decl.id);
                }
            }
        }
        if let Some(Expression::CallExpression(call_expr)) = &decl.init {
            if super::super::extract_factory::note_queue_binding(
                self,
                &name,
                call_expr,
                declaration,
            ) && extract_job_id::is_imported_callee(self, &call_expr.callee)
            {
                extract_job_id::note_queue_binding(self, &decl.id);
            }
        }
        if let Some(Expression::Identifier(id)) = &decl.init {
            if let Some(queue) = self.local_queue(id.name.as_str(), id.span.start) {
                self.note_queue_declaration(&name, declaration, queue);
                if extract_job_id::queue_identifier(self, id) {
                    extract_job_id::note_queue_binding(self, &decl.id);
                }
            }
        }
        super::super::extract_worker_rate_limit::note_variable(self, decl);
        walk::walk_variable_declarator(self, decl);
    }
}

impl QueueVisitor<'_> {
    fn is_builtin_non_queue_init(&self, expr: &Expression<'_>) -> bool {
        match expr.get_inner_expression() {
            Expression::NewExpression(new_expr) => {
                let Expression::Identifier(id) = new_expr.callee.get_inner_expression() else {
                    return false;
                };
                let name = id.name.as_str();
                matches!(
                    name,
                    "Set"
                        | "Map"
                        | "WeakSet"
                        | "WeakMap"
                        | "Headers"
                        | "URLSearchParams"
                        | "FormData"
                        | "Array"
                        | "Date"
                        | "RegExp"
                ) && !self.facts.imports.iter().any(|imp| imp.local == name)
                    && !self.namespace_imports.contains_key(name)
                    && self.lexical_binding(name, id.span.start).is_none()
            }
            Expression::ArrayExpression(_)
            | Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::FunctionExpression(_)
            | Expression::ArrowFunctionExpression(_) => true,
            _ => false,
        }
    }

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
