use oxc_ast::ast::{AssignmentTarget, Expression};

pub(super) fn commonjs_default_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a Expression<'a>> {
    let Expression::AssignmentExpression(assignment) = expression else {
        return None;
    };
    if assignment_target_path(&assignment.left)
        .as_deref()
        .is_none_or(|parts| parts != ["module", "exports"])
    {
        return None;
    }
    Some(&assignment.right)
}

fn assignment_target_path(target: &AssignmentTarget<'_>) -> Option<Vec<String>> {
    match target {
        AssignmentTarget::StaticMemberExpression(member) => {
            let mut parts = crate::ast::expression_path(&member.object)?;
            parts.push(member.property.name.to_string());
            Some(parts)
        }
        _ => None,
    }
}
