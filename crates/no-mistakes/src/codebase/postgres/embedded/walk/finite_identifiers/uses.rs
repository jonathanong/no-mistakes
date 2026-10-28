use crate::fx::FxHashSet;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, Function, IdentifierReference,
    StaticMemberExpression, UpdateExpression,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(super) struct Uses<'a> {
    pub(super) map: &'a str,
    pub(super) alias: &'a str,
    pub(super) key: &'a str,
    pub(super) allowed: FxHashSet<u32>,
    pub(super) declarations: FxHashSet<u32>,
    pub(super) valid: bool,
    pub(super) nodes: usize,
}
impl<'a> Visit<'a> for Uses<'_> {
    fn enter_node(&mut self, _: oxc_ast::AstKind<'a>) {
        self.nodes += 1;
        if self.nodes > 512 {
            self.valid = false;
        }
    }
    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if let oxc_ast::ast::Expression::Identifier(id) = member.object.get_inner_expression() {
            if id.name == self.alias
                && matches!(member.property.name.as_str(), "sql" | "value")
                && !member.optional
            {
                self.allowed.insert(id.span.start);
            }
        }
        walk::walk_static_member_expression(self, member);
    }
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        if matches!(id.name.as_str(), "eval" | "arguments")
            || ([self.map, self.alias, self.key].contains(&id.name.as_str())
                && !self.allowed.contains(&id.span.start))
        {
            self.valid = false;
        }
    }
    fn visit_binding_identifier(&mut self, id: &oxc_ast::ast::BindingIdentifier<'a>) {
        if [self.map, self.alias, self.key].contains(&id.name.as_str())
            && !self.declarations.contains(&id.span.start)
        {
            self.valid = false;
        }
    }
    fn visit_assignment_expression(&mut self, _: &AssignmentExpression<'a>) {
        self.valid = false;
    }
    fn visit_update_expression(&mut self, _: &UpdateExpression<'a>) {
        self.valid = false;
    }
    fn visit_unary_expression(&mut self, expression: &oxc_ast::ast::UnaryExpression<'a>) {
        if expression.operator.is_delete() {
            self.valid = false;
        }
        walk::walk_unary_expression(self, expression);
    }
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {
        self.valid = false;
    }
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {
        self.valid = false;
    }
    fn visit_class(&mut self, _: &oxc_ast::ast::Class<'a>) {
        self.valid = false;
    }
}
