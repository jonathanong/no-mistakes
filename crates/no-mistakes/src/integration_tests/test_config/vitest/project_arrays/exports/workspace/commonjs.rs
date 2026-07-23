use oxc_ast::ast::{
    Argument, AssignmentOperator, AssignmentTarget, Expression, Program, Statement,
};

pub(super) enum CommonjsWorkspaceExport<'a> {
    Expression(&'a Expression<'a>),
    Require(String),
}

pub(super) fn commonjs_workspace_export<'a>(
    program: &'a Program<'a>,
) -> Option<CommonjsWorkspaceExport<'a>> {
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
            if crate::ast::expression_path(&member.object)? != ["module"]
                || member.property.name != "exports"
            {
                return None;
            }
            direct_literal_require(&assignment.right)
                .map(CommonjsWorkspaceExport::Require)
                .or(Some(CommonjsWorkspaceExport::Expression(&assignment.right)))
        })
        .next_back()
}

fn direct_literal_require(expression: &Expression<'_>) -> Option<String> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if callee.name != "require" {
        return None;
    }
    let [Argument::StringLiteral(source)] = call.arguments.as_slice() else {
        return None;
    };
    Some(source.value.to_string())
}
