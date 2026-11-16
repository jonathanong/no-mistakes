//! A parameterized tag keeps unknown identifier and frozen-member holes.
//! Value interpolations stay the ordinary placeholder spelling unless one of
//! those holes shares the template. Then each value bind is `$N`, numbered
//! only among the binds, and the text after every hole stays.

use super::super::super::ScopeVisitor;
use super::untrusted_tag;
use crate::codebase::postgres::embedded::helpers::sentinel;
use crate::codebase::postgres::embedded::tags::{self, quasi_text};
use crate::codebase::postgres::embedded::{text::positional_digits, unpublished_sql_text};
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
    let (spliced, fragment) = splice(visitor, tagged);
    if fragment || sentinel::contains(&spliced) {
        Some(spliced)
    } else {
        Some(ordinary)
    }
}

fn splice(visitor: &ScopeVisitor<'_>, tagged: &TaggedTemplateExpression<'_>) -> (String, bool) {
    let mut out = String::new();
    let mut binds = 0usize;
    let mut fragment = false;
    for (index, quasi) in tagged.quasi.quasis.iter().enumerate() {
        let before = quasi_text(quasi, false);
        out.push_str(&before);
        let Some(hole) = tagged.quasi.expressions.get(index) else {
            continue;
        };
        if let Some(piece) = piece(visitor, hole, index as u32 + 1, &out) {
            fragment = true;
            out.push_str(&piece);
        } else if before.ends_with('$') {
            out.push_str(&positional_digits(hole, index + 1));
        } else {
            binds += 1;
            if !out.is_empty() && before.is_empty() {
                out.push(' ');
            }
            out.push('$');
            out.push_str(&binds.to_string());
        }
    }
    (out, fragment)
}

fn piece(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
    placeholder: u32,
    prefix: &str,
) -> Option<String> {
    // A value hole stays a bind unless it is itself a SQL fragment.
    if sentinel::value_position(prefix) && !fragment_hole(visitor, expr) {
        return None;
    }
    let text = match unwrap_ts_wrappers(expr) {
        Expression::Identifier(ident) => {
            let piece = super::super::super::imported::constants::identifier_piece(visitor, ident)?;
            if visitor
                .literal_objects
                .is_text_const(&visitor.scopes, ident.name.as_str())
            {
                // A value-position const already returned above. Elsewhere only
                // an empty or punctuation-leading fragment is spliced; a word
                // stays a bind so `widget-…` and `keep-…` are not inlined.
                return starts_with_own_punctuation(&piece).then_some(piece);
            }
            return sentinel::contains(&piece).then_some(piece);
        }
        Expression::StaticMemberExpression(member) => {
            super::super::super::frozen_alias::splice_member(visitor, member, placeholder)
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
