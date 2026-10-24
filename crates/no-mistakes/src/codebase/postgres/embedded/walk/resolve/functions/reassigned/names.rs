use super::super::super::for_each_bound_name;
use super::super::shadows::is_function_shaped;
use oxc_ast::ast::{
    AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty, Expression,
    VariableDeclarationKind, VariableDeclarator,
};

/// A declarator's own initializer is exempted from counting as a
/// reassignment when it is itself a same-file callable-helper shape
/// (`function`/arrow expression, matching [`is_function_shaped`]) **and**
/// its declaration is `const`: that combination is exactly what
/// `collect::const_resolvable` collects as a legitimate same-file helper,
/// not a rebinding of one — a literal function/arrow value can't itself be
/// "the wrong body" the way an identifier reference like `externalBuilder`
/// can.
///
/// `var`/`let` never reach `const_resolvable` — only `const` function
/// expressions are collected as helpers — so a same-shaped
/// `var build = () => …;` following an earlier `function build() {}` is a
/// real runtime rebinding to a body `LocalFunctions` never re-collects: it
/// must still count as a reassignment, or calls to `build` keep resolving
/// through the stale `function` declaration's body. A re-declaration whose
/// initializer resolves to something else (an identifier, a call, ...)
/// still counts regardless of kind, which is what keeps
/// `var build = externalBuilder;` rejected after an earlier
/// `function build() {}`.
pub(super) fn record_declarator_reassignment<'a>(
    kind: VariableDeclarationKind,
    declarator: &VariableDeclarator<'a>,
    on_name: &mut impl FnMut(&'a str),
) {
    let is_collectible_helper = kind == VariableDeclarationKind::Const
        && declarator.init.as_ref().is_some_and(is_function_shaped);
    let is_reassignment = declarator.init.is_some() && !is_collectible_helper;
    if is_reassignment {
        for_each_bound_name(&declarator.id, on_name);
    }
}

/// Every name a (possibly destructuring) assignment target writes to,
/// however deeply nested — `x`, `{ a: x }`, `[x]`, `{ x = 1 }`, and any
/// rest element or combination of those. A check that only handled a bare
/// `AssignmentTargetIdentifier` would miss `({ build } = providers)`
/// reassigning a same-named top-level helper through destructuring, letting
/// `LocalFunctions::collect` keep resolving calls through its stale body.
pub(in crate::codebase::postgres::embedded::walk) fn for_each_assigned_name<'a>(
    target: &AssignmentTarget<'a>,
    on_name: &mut impl FnMut(&'a str),
) {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(ident) => on_name(ident.name.as_str()),
        AssignmentTarget::ArrayAssignmentTarget(array) => {
            for element in array.elements.iter().flatten() {
                for_each_assigned_name_maybe_default(element, on_name);
            }
            if let Some(rest) = &array.rest {
                for_each_assigned_name(&rest.target, on_name);
            }
        }
        AssignmentTarget::ObjectAssignmentTarget(object) => {
            for property in &object.properties {
                match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                        on_name(property.binding.name.as_str());
                    }
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                        for_each_assigned_name_maybe_default(&property.binding, on_name);
                    }
                }
            }
            if let Some(rest) = &object.rest {
                for_each_assigned_name(&rest.target, on_name);
            }
        }
        AssignmentTarget::TSAsExpression(wrapper) => {
            for_each_wrapped_assigned_name(&wrapper.expression, on_name);
        }
        AssignmentTarget::TSSatisfiesExpression(wrapper) => {
            for_each_wrapped_assigned_name(&wrapper.expression, on_name);
        }
        AssignmentTarget::TSNonNullExpression(wrapper) => {
            for_each_wrapped_assigned_name(&wrapper.expression, on_name);
        }
        AssignmentTarget::TSTypeAssertion(wrapper) => {
            for_each_wrapped_assigned_name(&wrapper.expression, on_name);
        }
        _ => {}
    }
}

fn for_each_wrapped_assigned_name<'a>(
    expression: &Expression<'a>,
    on_name: &mut impl FnMut(&'a str),
) {
    if let Expression::Identifier(ident) = expression.get_inner_expression() {
        on_name(ident.name.as_str());
    }
}

fn for_each_assigned_name_maybe_default<'a>(
    target: &AssignmentTargetMaybeDefault<'a>,
    on_name: &mut impl FnMut(&'a str),
) {
    if let Some(target) = target.as_assignment_target() {
        for_each_assigned_name(target, on_name);
        return;
    }
    if let AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) = target {
        for_each_assigned_name(&with_default.binding, on_name);
    }
}
