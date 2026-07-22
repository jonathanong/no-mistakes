use oxc_ast::ast::{AssignmentOperator, AssignmentTarget, Expression, Program, Statement};

pub(super) fn commonjs_setup_expression<'a>(
    program: &'a Program<'a>,
    exported: &str,
) -> Option<&'a Expression<'a>> {
    program
        .body
        .iter()
        .filter_map(|statement| {
            let Statement::ExpressionStatement(statement) = statement else {
                return None;
            };
            let Expression::AssignmentExpression(assignment) = &statement.expression else {
                return None;
            };
            if assignment.operator != AssignmentOperator::Assign {
                return None;
            }
            let AssignmentTarget::StaticMemberExpression(member) = &assignment.left else {
                return None;
            };
            let object = crate::ast::expression_path(&member.object)?;
            let matches = (exported == "default"
                && object == ["module"]
                && member.property.name == "exports")
                || (object == ["exports"] && member.property.name == exported);
            matches.then_some(&assignment.right)
        })
        .next_back()
}
