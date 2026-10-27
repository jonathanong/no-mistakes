use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ComputedMemberExpression, Expression, IdentifierReference, Program, StaticMemberExpression,
};
use oxc_ast_visit::{walk, Visit};

pub(super) fn exposed(program: &Program<'_>) -> bool {
    let mut visitor = Exposure(false);
    visitor.visit_program(program);
    visitor.0
}

// Reads can escape through aliases. Member names are deliberately conservative:
// resolving arbitrary namespace/constructor aliases would exceed this evaluator.
struct Exposure(bool);
impl<'a> Visit<'a> for Exposure {
    fn visit_identifier_reference(&mut self, value: &IdentifierReference<'a>) {
        self.0 |= value.name == "Array";
    }
    fn visit_static_member_expression(&mut self, value: &StaticMemberExpression<'a>) {
        self.0 |= sensitive_member(value.property.name.as_str());
        walk::walk_static_member_expression(self, value);
    }
    fn visit_computed_member_expression(&mut self, value: &ComputedMemberExpression<'a>) {
        self.0 |= matches!(unwrap_ts_wrappers(&value.expression),
            Expression::StringLiteral(value) if sensitive_member(&crate::ast::js_literal(value)));
        walk::walk_computed_member_expression(self, value);
    }
}

fn sensitive_member(name: &str) -> bool {
    matches!(
        name,
        "Array" | "__proto__" | "constructor" | "getPrototypeOf"
    )
}
