use super::write_targets::from_expression;
use oxc_ast::ast::UnaryOperator;
use oxc_ast_visit::{walk, Visit};
use oxc_parser::Parser;
use oxc_span::SourceType;

struct DeleteOperands {
    found: Vec<Option<String>>,
}

impl<'a> Visit<'a> for DeleteOperands {
    fn visit_unary_expression(&mut self, expression: &oxc_ast::ast::UnaryExpression<'a>) {
        if expression.operator == UnaryOperator::Delete {
            self.found
                .push(from_expression(&expression.argument).map(|(name, _, _)| name));
        }
        walk::walk_unary_expression(self, expression);
    }
}

#[test]
fn from_expression_records_static_computed_and_non_member_deletes() {
    let source =
        "delete tables.users; delete tables['users']; delete ident; delete (tables as any).users;";
    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let mut visitor = DeleteOperands { found: Vec::new() };
    visitor.visit_program(&parsed.program);
    assert_eq!(
        visitor.found,
        vec![
            Some("tables.users".into()),
            Some("tables.users".into()),
            None,
            Some("tables.users".into()),
        ]
    );
}
