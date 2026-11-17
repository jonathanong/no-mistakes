//! Parameterized values keep generated bind identities through composition.
//! Identifier and frozen-member holes retain their independently resolved text.

use super::super::super::ScopeVisitor;
use super::untrusted_tag;
use crate::codebase::postgres::embedded::helpers::{sentinel, SiteKind};
use crate::codebase::postgres::embedded::tags::{self, quasi_text};
use crate::codebase::postgres::embedded::{
    placeholders::{internal_bind, internal_placeholder},
    text::render_hole,
    unpublished_sql_text,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, TaggedTemplateExpression};

pub(super) fn text(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> Option<String> {
    if untrusted_tag(expr, visitor) {
        return None;
    }
    let Expression::TaggedTemplateExpression(tagged) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    let ordinary = unpublished_sql_text(expr)?;
    if !visitor
        .functions
        .imported_sql_tags()
        .parameterizes_all(&tagged.tag)
    {
        return Some(ordinary);
    }
    Some(splice(visitor, tagged))
}

fn splice(visitor: &ScopeVisitor<'_>, tagged: &TaggedTemplateExpression<'_>) -> String {
    let mut out = String::new();
    let mut binds = 0usize;
    for (index, quasi) in tagged.quasi.quasis.iter().enumerate() {
        let before = quasi_text(quasi, false);
        out.push_str(&before);
        let Some(hole) = tagged.quasi.expressions.get(index) else {
            continue;
        };
        if before.ends_with('$') {
            out.push_str(&render_hole(&before, hole, index + 1, &out, false).text);
        } else if let Some(piece) = piece(
            visitor,
            hole,
            index as u32 + 1,
            &out,
            index > 0 && before.is_empty(),
        ) {
            out.push_str(&piece);
        } else if sentinel::in_relation_slot(&out, out.len()) {
            out.push_str(&internal_placeholder(index + 1));
        } else {
            binds += 1;
            if !out.is_empty() && before.is_empty() {
                out.push(' ');
            }
            out.push_str(&internal_bind(binds));
        }
    }
    out
}

fn piece(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
    placeholder: u32,
    prefix: &str,
    adjacent_fragment: bool,
) -> Option<String> {
    // A value hole stays a bind unless it is itself a SQL fragment.
    if sentinel::value_position(prefix) && !fragment_hole(visitor, expr) {
        return None;
    }
    let text = match unwrap_ts_wrappers(expr) {
        Expression::Identifier(ident) => {
            if !visitor
                .literal_objects
                .is_text_const(&visitor.scopes, ident.name.as_str())
                && !sentinel::in_relation_slot(prefix, prefix.len())
                && !fragment_hole(visitor, expr)
            {
                return None;
            }
            let piece = super::super::super::imported::constants::identifier_piece(visitor, ident)?;
            if visitor
                .literal_objects
                .is_text_const(&visitor.scopes, ident.name.as_str())
            {
                return (adjacent_fragment && starts_with_own_punctuation(&piece)).then_some(piece);
            }
            return sentinel::contains(&piece).then_some(piece);
        }
        Expression::StaticMemberExpression(member) => {
            if matches!(unwrap_ts_wrappers(&member.object), Expression::Identifier(object)
                if !visitor.shadowed_locally(object.name.as_str()) && visitor.constant_namespaces.contains(object.name.as_str()))
                && !sentinel::in_relation_slot(prefix, prefix.len())
                && !fragment_hole(visitor, expr)
            {
                return None;
            }
            let kind = if sentinel::in_relation_slot(prefix, prefix.len()) {
                SiteKind::PlainClause
            } else {
                SiteKind::ParameterizedMember
            };
            super::super::super::frozen_alias::splice_member(visitor, member, placeholder, kind)
        }
        _ => None,
    }?;
    sentinel::contains(&text).then_some(text)
}

fn starts_with_own_punctuation(text: &str) -> bool {
    text.is_empty() || text.starts_with(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
}

fn fragment_hole(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>) -> bool {
    tags::fragments::is_sql_fragment(
        expr,
        &mut |name| super::tag_shadowed(name, visitor),
        visitor.functions.imported_sql_tags(),
    )
}
