use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    AssignmentOperator, AssignmentTarget, Expression, ObjectPropertyKind, Program, Statement,
};

pub(super) fn commonjs_setup_expression<'a>(
    program: &'a Program<'a>,
    exported: &str,
) -> Option<&'a Expression<'a>> {
    let mut exports_alias = None;
    for statement in program.body.iter().rev() {
        let Some((object, property, right)) = commonjs_assignment(statement) else {
            continue;
        };
        if object == ["module"] && property == "exports" {
            // This replacement also detaches the original `exports` alias.
            return (exported == "default")
                .then_some(right)
                .or_else(|| commonjs_object_export(right, exported));
        }
        if object == ["module", "exports"] && property == exported {
            return Some(right);
        }
        if object == ["exports"] && property == exported {
            exports_alias = exports_alias.or(Some(right));
        }
    }
    exports_alias
}

fn commonjs_assignment<'a>(
    statement: &'a Statement<'a>,
) -> Option<(Vec<String>, &'a str, &'a Expression<'a>)> {
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
    Some((
        crate::ast::expression_path(&member.object)?,
        member.property.name.as_str(),
        &assignment.right,
    ))
}

fn commonjs_object_export<'a>(
    expression: &'a Expression<'a>,
    exported: &str,
) -> Option<&'a Expression<'a>> {
    let Expression::ObjectExpression(object) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    object.properties.iter().rev().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        (!property.computed
            && !property.method
            && crate::integration_tests::test_config::shared_literals::property_key_name(
                &property.key,
            )
            .as_deref()
                == Some(exported))
        .then_some(&property.value)
    })
}
