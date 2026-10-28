use super::Payload;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{self, BindingPattern, Expression};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(super) struct Uses<'a> {
    params: Vec<&'a str>,
    callback: usize,
    allowed_callback: FxHashSet<u32>,
    allowed_payload: FxHashSet<u32>,
    discarded_appends: FxHashSet<u32>,
    references: Vec<(&'a str, u32)>,
    payload: Option<Payload>,
    count: usize,
    valid: bool,
    nodes: usize,
    looped: bool,
}

impl<'a> Uses<'a> {
    pub(super) fn inspect(
        body: &ast::FunctionBody<'a>,
        params: Vec<&'a str>,
        callback: usize,
    ) -> Option<Payload> {
        let mut uses = Self {
            params,
            callback,
            allowed_callback: FxHashSet::default(),
            allowed_payload: FxHashSet::default(),
            discarded_appends: FxHashSet::default(),
            references: Vec::new(),
            payload: None,
            count: 0,
            valid: true,
            nodes: 0,
            looped: false,
        };
        uses.visit_function_body(body);
        let payload = uses.payload?;
        let payload_name = match payload {
            Payload::Parameter(index) => Some(uses.params[index]),
            Payload::Fixed => None,
        };
        if !uses.valid
            || uses.count == 0
            || uses.count > 16
            || uses.looped && matches!(payload, Payload::Parameter(_))
        {
            return None;
        }
        for (name, span) in uses.references {
            if name == uses.params[callback] && !uses.allowed_callback.contains(&span)
                || payload_name == Some(name) && !uses.allowed_payload.contains(&span)
            {
                return None;
            }
        }
        Some(payload)
    }

    fn payload(&self, arg: &Expression<'a>) -> Option<Payload> {
        match unwrap_ts_wrappers(arg) {
            Expression::Identifier(id) => self
                .params
                .iter()
                .position(|name| *name == id.name && *name != self.params[self.callback])
                .map(Payload::Parameter),
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
                if super::super::syntax::head(arg).proof(false)
                    == super::super::HeadProof::Literal =>
            {
                Some(Payload::Fixed)
            }
            _ => None,
        }
    }
}

impl<'a> Visit<'a> for Uses<'a> {
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
