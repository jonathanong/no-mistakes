//! Preserve reachable values/effects while leaving opaque expression values unknown.
use super::super::Expr;
use oxc_ast::ast::{Expression, Function, IdentifierReference};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(super) fn collect(value: &Expression<'_>, source: &str) -> Vec<Expr> {
    struct Children<'s> {
        source: &'s str,
        values: Vec<Expr>,
    }
    impl<'a> Visit<'a> for Children<'_> {
        fn visit_expression(&mut self, value: &Expression<'a>) {
            self.values.push(super::expression(value, self.source));
        }
        fn visit_identifier_reference(&mut self, value: &IdentifierReference<'a>) {
            // Assignment-target identifiers are not visited as expressions.
            self.values.push(Expr::Name(value.name.to_string()));
        }
        fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {
            // Class/object method bodies execute later; an expression-valued
            // function child is already summarized by visit_expression above.
        }
    }
    let mut children = Children {
        source,
        values: Vec::new(),
    };
    walk::walk_expression(&mut children, value);
    children.values
}
