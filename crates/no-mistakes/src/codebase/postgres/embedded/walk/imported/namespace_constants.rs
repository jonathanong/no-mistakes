//! Namespace constant reads keep their static selector until prepared resolution.

use super::super::super::ScopeVisitor;
use crate::codebase::postgres::embedded::helpers::{sentinel, ImportedSite};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{Expression, StaticMemberExpression};

pub(in crate::codebase::postgres::embedded::walk) fn piece(
    visitor: &ScopeVisitor<'_>,
    member: &StaticMemberExpression<'_>,
    allow_fragment: bool,
) -> Option<String> {
    let Expression::Identifier(object) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    let name = object.name.as_str();
    if member.optional
        || visitor.shadowed_locally(name)
        || !visitor.constant_namespaces.contains(name)
    {
        return None;
    }
    let index = visitor.imported_sites.borrow_mut().intern(
        member.span.start,
        ImportedSite {
            local: name.to_string(),
            member: Some(member.property.name.to_string()),
            args: Vec::new(),
            value_used: true,
            line: byte_offset_to_line(visitor.source, member.span.start as usize),
            constant: Some(allow_fragment),
        },
    );
    Some(sentinel::encode(index))
}
