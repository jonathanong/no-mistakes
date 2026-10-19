use super::super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, Expression};

const MEMBER_MUTATORS: &[(&str, &str)] = &[
    ("Object", "assign"),
    ("Object", "defineProperty"),
    ("Object", "defineProperties"),
    ("Object", "setPrototypeOf"),
    ("Reflect", "set"),
    ("Reflect", "defineProperty"),
    ("Reflect", "deleteProperty"),
    ("Reflect", "setPrototypeOf"),
];

impl ScopeVisitor<'_> {
    /// `true` when this argument can replace a receiver's `.query`.
    ///
    /// An identifier callee is non-mutating only when this file proves the
    /// name is an import or ambient `declare function` with no other binding.
    pub(super) fn namespace_argument_mutates(&self, callee: &Expression<'_>, index: usize) -> bool {
        if known_member_mutator(callee) {
            return index == 0;
        }
        // `new Proxy(target, handler)` replaces the target even when `Proxy`
        // is an import. Other arguments keep the ordinary callee rule.
        match callee_name(callee) {
            Some("Proxy") if index == 0 => true,
            Some(name) => !self.functions.declared_elsewhere(name),
            None => true,
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn revoke_namespace_arguments(
        &mut self,
        callee: &Expression<'_>,
        arguments: &[Argument<'_>],
    ) {
        for (index, argument) in arguments.iter().enumerate() {
            if let Some(expression) = argument.as_expression() {
                self.revoke_namespace_expression(callee, index, expression);
            }
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn revoke_namespace_expression(
        &mut self,
        callee: &Expression<'_>,
        index: usize,
        expression: &Expression<'_>,
    ) {
        if self.namespace_argument_mutates(callee, index) {
            self.revoke_query_receiver(expression);
        } else {
            self.revoke_passed_argument(expression);
        }
    }
}

fn callee_name<'a>(callee: &'a Expression<'a>) -> Option<&'a str> {
    match unwrap_ts_wrappers(callee) {
        Expression::Identifier(name) => Some(name.name.as_str()),
        _ => None,
    }
}

fn known_member_mutator(callee: &Expression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(callee) else {
        return false;
    };
    let Expression::Identifier(object) = unwrap_ts_wrappers(&member.object) else {
        return false;
    };
    MEMBER_MUTATORS.iter().any(|(owner, property)| {
        object.name.as_str() == *owner && member.property.name.as_str() == *property
    })
}
