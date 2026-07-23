use super::super::{objects, shared, ExprMap};
use crate::ast;
use oxc_ast::ast::{AssignmentOperator, AssignmentTarget, Expression, Program, Statement};

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn commonjs_exported_expression<
    'a,
>(
    program: &'a Program<'a>,
    exported: &str,
    bindings: &ExprMap<'a>,
) -> Option<&'a Expression<'a>> {
    let mut resolved = None;
    let mut exports_detached = false;
    for statement in &program.body {
        let Some((path, right)) = commonjs_assignment(statement) else {
            continue;
        };
        if path.len() == 2 && path[0] == "module" && path[1] == "exports" {
            exports_detached = true;
            resolved = (exported == "default")
                .then_some(right)
                .or_else(|| named_object_property(right, exported, bindings));
        } else if (exported != "default"
            && path.len() == 3
            && path[0] == "module"
            && path[1] == "exports"
            && path[2] == exported)
            || (!exports_detached && path.len() == 2 && path[0] == "exports" && path[1] == exported)
        {
            resolved = Some(right);
        }
    }
    resolved
}

fn commonjs_assignment<'a>(
    statement: &'a Statement<'a>,
) -> Option<(Vec<String>, &'a Expression<'a>)> {
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
    let mut path = ast::expression_path(&member.object)?;
    path.push(member.property.name.to_string());
    Some((path, &assignment.right))
}

fn named_object_property<'a>(
    expression: &'a Expression<'a>,
    exported: &str,
    bindings: &ExprMap<'a>,
) -> Option<&'a Expression<'a>> {
    let object = objects::expression_object(expression, bindings)?;
    shared::property_expression_deep(object, exported, bindings)
}
