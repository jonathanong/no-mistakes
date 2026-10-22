use super::syntax;
use oxc_ast::ast::{
    BindingPattern, Expression, TSImportEqualsDeclaration, TSModuleReference, VariableDeclarator,
};
use std::collections::HashSet;

/// `const { mock } = vi` and `const alias = require` are not coverage. Later
/// calls through those names are unanalyzable at effort low.
pub(super) fn note_declarator(
    declarator: &VariableDeclarator<'_>,
    mocks: &mut HashSet<String>,
    requires: &mut HashSet<String>,
) {
    let Some(init) = &declarator.init else {
        return;
    };
    let init = crate::codebase::ts_source::unwrap_ts_wrappers(init);
    if is_vi_or_jest(init) {
        note_mock_binding(&declarator.id, mocks);
        return;
    }
    if is_require_binding(init) {
        if let Some(name) = bound_name(&declarator.id) {
            requires.insert(name);
        }
    }
}

fn note_mock_binding(pattern: &BindingPattern<'_>, mocks: &mut HashSet<String>) {
    let BindingPattern::ObjectPattern(object) = pattern else {
        return;
    };
    for property in &object.properties {
        if !property.key.is_specific_static_name("mock") {
            continue;
        }
        if let Some(name) = bound_name(&property.value) {
            mocks.insert(name);
        }
    }
}

fn bound_name(pattern: &BindingPattern<'_>) -> Option<String> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.to_string()),
        BindingPattern::AssignmentPattern(assignment) => bound_name(&assignment.left),
        _ => None,
    }
}

fn is_vi_or_jest(expression: &Expression<'_>) -> bool {
    let Expression::Identifier(identifier) = expression else {
        return false;
    };
    matches!(identifier.name.as_str(), "vi" | "jest")
}

fn is_require_binding(expression: &Expression<'_>) -> bool {
    let Expression::Identifier(identifier) = expression else {
        return false;
    };
    identifier.name == "require"
}

pub(super) fn is_computed_vi_mock(callee: &Expression<'_>) -> bool {
    let callee = crate::codebase::ts_source::unwrap_ts_wrappers(callee);
    let Expression::ComputedMemberExpression(member) = callee else {
        return false;
    };
    let object = crate::codebase::ts_source::unwrap_ts_wrappers(&member.object);
    if !is_vi_or_jest(object) {
        return false;
    }
    syntax::literal_expr(crate::codebase::ts_source::unwrap_ts_wrappers(
        &member.expression,
    ))
    .as_deref()
        == Some("mock")
}

pub(super) fn is_module_require(callee: &Expression<'_>) -> bool {
    let callee = crate::codebase::ts_source::unwrap_ts_wrappers(callee);
    match callee {
        Expression::StaticMemberExpression(member) => {
            member.property.name == "require" && is_module_object(&member.object)
        }
        Expression::ComputedMemberExpression(member) => {
            is_module_object(&member.object)
                && syntax::literal_expr(crate::codebase::ts_source::unwrap_ts_wrappers(
                    &member.expression,
                ))
                .as_deref()
                    == Some("require")
        }
        _ => false,
    }
}

fn is_module_object(expression: &Expression<'_>) -> bool {
    let expression = crate::codebase::ts_source::unwrap_ts_wrappers(expression);
    matches!(expression, Expression::Identifier(identifier) if identifier.name == "module")
}

pub(super) fn is_require_equals(declaration: &TSImportEqualsDeclaration<'_>) -> bool {
    matches!(
        declaration.module_reference,
        TSModuleReference::ExternalModuleReference(_)
    )
}

pub(super) fn callee_name<'a>(callee: &'a Expression<'a>) -> Option<&'a str> {
    let callee = crate::codebase::ts_source::unwrap_ts_wrappers(callee);
    match callee {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}
