use super::*;
use oxc_ast_visit::{walk, Visit};
struct Calls {
    unknown: bool,
    invoked: bool,
}
impl<'a> Visit<'a> for Calls {
    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        let supported = matches!(unwrap_ts_wrappers(&call.callee), Expression::StaticMemberExpression(member) if member.property.name == "map");
        self.unknown |= !supported;
        let outer = self.invoked;
        self.invoked = true;
        walk::walk_call_expression(self, call);
        self.invoked = outer;
    }
    fn visit_new_expression(&mut self, _: &oxc_ast::ast::NewExpression<'a>) {
        self.unknown = true;
    }
    fn visit_tagged_template_expression(&mut self, _: &oxc_ast::ast::TaggedTemplateExpression<'a>) {
        self.unknown = true;
    }
    fn visit_function(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        if self.invoked {
            walk::walk_function(self, function, flags);
        }
    }
    fn visit_arrow_function_expression(
        &mut self,
        function: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        if self.invoked {
            walk::walk_arrow_function_expression(self, function);
        }
    }
}
pub(super) fn has_unknown_call(expression: &Expression<'_>) -> bool {
    let mut calls = Calls {
        unknown: false,
        invoked: false,
    };
    calls.visit_expression(expression);
    calls.unknown
}
