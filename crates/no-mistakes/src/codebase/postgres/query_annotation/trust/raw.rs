use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, SimpleAssignmentTarget};

fn object_is_string(object: &Expression<'_>) -> bool {
    matches!(unwrap_ts_wrappers(object), Expression::Identifier(id) if id.name == "String")
}
pub(super) fn target(value: &SimpleAssignmentTarget<'_>) -> bool {
    match value {
        SimpleAssignmentTarget::StaticMemberExpression(value) => {
            value.property.name == "raw" && object_is_string(&value.object)
        }
        SimpleAssignmentTarget::ComputedMemberExpression(value) => {
            object_is_string(&value.object)
                && matches!(unwrap_ts_wrappers(&value.expression), Expression::StringLiteral(value) if value.value == "raw")
        }
        SimpleAssignmentTarget::TSAsExpression(value) => expression(&value.expression),
        SimpleAssignmentTarget::TSSatisfiesExpression(value) => expression(&value.expression),
        SimpleAssignmentTarget::TSNonNullExpression(value) => expression(&value.expression),
        SimpleAssignmentTarget::TSTypeAssertion(value) => expression(&value.expression),
        _ => false,
    }
}
pub(super) fn expression(value: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(value) {
        Expression::StaticMemberExpression(value) => {
            value.property.name == "raw" && object_is_string(&value.object)
        }
        Expression::ComputedMemberExpression(value) => {
            object_is_string(&value.object)
                && matches!(unwrap_ts_wrappers(&value.expression), Expression::StringLiteral(value) if value.value == "raw")
        }
        _ => false,
    }
}
