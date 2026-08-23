use super::ImportBinding;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, Expression};

pub(super) fn is_direct_commonjs_require(expression: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(expression),
        Expression::CallExpression(call)
            if matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == "require")
    )
}

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn direct_literal_require_binding(
    expression: &Expression<'_>,
) -> Option<ImportBinding> {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) else {
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
    Some(ImportBinding {
        source: source.value.to_string(),
        imported: "default".to_string(),
    })
}

/// Follow only literal CommonJS requires. Static members are equivalent to
/// named imports; dynamic or computed forms remain conservative.
pub(super) fn require_binding(expression: &Expression<'_>) -> Option<(String, String)> {
    match unwrap_ts_wrappers(expression) {
        Expression::StaticMemberExpression(member) => require_binding(&member.object)
            .map(|(source, _)| (source, member.property.name.to_string())),
        _ => direct_literal_require_binding(expression)
            .map(|binding| (binding.source, binding.imported)),
    }
}
