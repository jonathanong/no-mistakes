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

// Any prototype exposure can escape through aliases or mutate via opaque calls.
// Deliberately include reads: proving that an exposed prototype remains pristine
// would require tracking arbitrary effects beyond this bounded evaluator.
struct Exposure(bool);
impl<'a> Visit<'a> for Exposure {
    fn visit_identifier_reference(&mut self, value: &IdentifierReference<'a>) {
        // A constructor reference can escape through an arbitrary alias.
        self.0 |= value.name == "Array";
    }
    fn visit_static_member_expression(&mut self, value: &StaticMemberExpression<'a>) {
        self.0 |= value.property.name == "__proto__"
            || value.property.name == "getPrototypeOf"
            || (value.property.name == "Array" && global(&value.object))
            || (value.property.name == "prototype" && array_constructor(&value.object));
        walk::walk_static_member_expression(self, value);
    }
    fn visit_computed_member_expression(&mut self, value: &ComputedMemberExpression<'a>) {
        self.0 |= named(&value.expression, "__proto__")
            || named(&value.expression, "getPrototypeOf")
            || (named(&value.expression, "Array") && global(&value.object))
            || (named(&value.expression, "prototype") && array_constructor(&value.object));
        walk::walk_computed_member_expression(self, value);
    }
}

fn named(value: &Expression<'_>, name: &str) -> bool {
    matches!(unwrap_ts_wrappers(value), Expression::StringLiteral(value) if value.value == name)
}
fn global(value: &Expression<'_>) -> bool {
    matches!(unwrap_ts_wrappers(value), Expression::Identifier(id) if id.name == "globalThis")
}
fn array_constructor(value: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(value) {
        Expression::Identifier(id) => id.name == "Array",
        Expression::StaticMemberExpression(value) => {
            value.property.name == "Array" && global(&value.object)
        }
        Expression::ComputedMemberExpression(value) => {
            named(&value.expression, "Array") && global(&value.object)
        }
        _ => false,
    }
}
