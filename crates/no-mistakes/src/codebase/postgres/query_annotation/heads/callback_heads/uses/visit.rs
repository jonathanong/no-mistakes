//! Full callback and payload reference census.
use super::{Payload, Uses};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{self, BindingPattern, Expression};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

impl<'a> Visit<'a> for Uses<'a, '_> {
    fn enter_node(&mut self, _: oxc_ast::AstKind<'a>) {
        self.nodes += 1;
        if self.nodes > 2048 {
            self.valid = false;
        }
    }
    fn visit_call_expression(&mut self, call: &ast::CallExpression<'a>) {
        if call.optional
            || call
                .arguments
                .iter()
                .any(|arg| arg.as_expression().is_none())
        {
            self.valid = false;
        }
        if let Expression::Identifier(id) = unwrap_ts_wrappers(&call.callee) {
            if id.name == self.params[self.callback] {
                let payload = (call.arguments.len() == 1)
                    .then(|| call.arguments.first()?.as_expression())
                    .flatten()
                    .and_then(|arg| self.payload(arg));
                if let Some(payload) =
                    payload.filter(|value| self.payload.is_none_or(|old| old == *value))
                {
                    self.payload = Some(payload);
                    self.count += 1;
                    self.allowed_callback.insert(id.span.start);
                    if let Payload::Parameter(_) = payload {
                        if let Some(Expression::Identifier(arg)) = call
                            .arguments
                            .first()
                            .and_then(|arg| arg.as_expression())
                            .map(unwrap_ts_wrappers)
                        {
                            self.allowed_payload.insert(arg.span.start);
                        }
                    }
                } else {
                    self.valid = false;
                }
            }
        }
        if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) {
            if !member.optional
                && member.property.name == "append"
                && call.arguments.len() == 1
                && self.discarded_appends.contains(&call.span.start)
            {
                if let Expression::Identifier(id) = unwrap_ts_wrappers(&member.object) {
                    if self.params.iter().any(|name| *name == id.name) {
                        self.allowed_payload.insert(id.span.start);
                    }
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
    fn visit_expression_statement(&mut self, statement: &ast::ExpressionStatement<'a>) {
        if let Expression::CallExpression(call) = unwrap_ts_wrappers(&statement.expression) {
            self.discarded_appends.insert(call.span.start);
        }
        walk::walk_expression_statement(self, statement);
    }
    fn visit_identifier_reference(&mut self, id: &ast::IdentifierReference<'a>) {
        if matches!(id.name.as_str(), "eval" | "arguments") {
            self.valid = false;
        }
        if self.params.iter().any(|name| *name == id.name) {
            self.references.push((id.name.as_str(), id.span.start));
        }
    }
    fn visit_binding_identifier(&mut self, id: &ast::BindingIdentifier<'a>) {
        if self.params.iter().any(|name| *name == id.name) {
            self.valid = false;
        }
    }
    fn visit_assignment_expression(&mut self, _: &ast::AssignmentExpression<'a>) {
        self.valid = false;
    }
    fn visit_update_expression(&mut self, _: &ast::UpdateExpression<'a>) {
        self.valid = false;
    }
    fn visit_unary_expression(&mut self, value: &ast::UnaryExpression<'a>) {
        if value.operator.is_delete() {
            self.valid = false;
        }
        walk::walk_unary_expression(self, value);
    }
    fn visit_for_of_statement(&mut self, value: &ast::ForOfStatement<'a>) {
        self.looped = true;
        if !matches!(&value.left, ast::ForStatementLeft::VariableDeclaration(declaration)
            if declaration.kind.is_const()
                && declaration.declarations.iter().all(|item| matches!(&item.id, BindingPattern::BindingIdentifier(id) if !self.params.contains(&id.name.as_str()))))
        {
            self.valid = false;
        }
        walk::walk_for_of_statement(self, value);
    }
    fn visit_for_in_statement(&mut self, _: &ast::ForInStatement<'a>) {
        self.valid = false;
    }
    fn visit_for_statement(&mut self, _: &ast::ForStatement<'a>) {
        self.valid = false;
    }
    fn visit_while_statement(&mut self, _: &ast::WhileStatement<'a>) {
        self.valid = false;
    }
    fn visit_function(&mut self, _: &ast::Function<'a>, _: ScopeFlags) {
        self.valid = false;
    }
    fn visit_arrow_function_expression(&mut self, _: &ast::ArrowFunctionExpression<'a>) {
        self.valid = false;
    }
    fn visit_class(&mut self, _: &ast::Class<'a>) {
        self.valid = false;
    }
}
