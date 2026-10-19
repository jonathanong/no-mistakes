use super::super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

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
    /// An identifier callee is mutating only when this file declares its body.
    /// A function declared elsewhere is the one non-mutating escape.
    pub(super) fn namespace_argument_mutates(
        &self,
        call: &CallExpression<'_>,
        index: usize,
    ) -> bool {
        if known_member_mutator(call) {
            return index == 0;
        }
        match unwrap_ts_wrappers(&call.callee) {
            Expression::Identifier(name) => {
                self.functions.declares_function_body(name.name.as_str())
            }
            _ => true,
        }
    }
}

fn known_member_mutator(call: &CallExpression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return false;
    };
    let Expression::Identifier(object) = unwrap_ts_wrappers(&member.object) else {
        return false;
    };
    MEMBER_MUTATORS.iter().any(|(owner, property)| {
        object.name.as_str() == *owner && member.property.name.as_str() == *property
    })
}
