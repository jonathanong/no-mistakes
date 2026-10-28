use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, CallExpression, Expression, Function,
    IdentifierReference, StaticMemberExpression, TaggedTemplateExpression, UpdateExpression,
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
impl Uses<'_> {
    fn is_record_receiver(&self, expression: &Expression<'_>) -> bool {
        matches!(unwrap_ts_wrappers(expression), Expression::StaticMemberExpression(member)
            if matches!(unwrap_ts_wrappers(&member.object), Expression::Identifier(id) if id.name == self.alias))
    }
}
impl<'a> Visit<'a> for Uses<'_> {
    // A method or tag receives the selected record as `this`, allowing it to
    // replace the own SQL field even though the field access itself is a read.
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.is_record_receiver(&call.callee) {
            self.valid = false;
        }
        walk::walk_call_expression(self, call);
    }
    fn visit_tagged_template_expression(&mut self, tagged: &TaggedTemplateExpression<'a>) {
        if self.is_record_receiver(&tagged.tag) {
            self.valid = false;
        }
        walk::walk_tagged_template_expression(self, tagged);
    }
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
