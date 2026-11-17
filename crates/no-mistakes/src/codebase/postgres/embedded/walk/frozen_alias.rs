//! Cross-file member holes for consts bound to imported record helpers.
use super::imported;
use super::state::FrozenAlias;
use super::variants::Recovered;
use super::ScopeVisitor;
use crate::codebase::postgres::embedded::helpers::{sentinel, ImportedSite, SiteKind};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{CallExpression, Expression, StaticMemberExpression};

pub(super) fn note(
    visitor: &mut ScopeVisitor<'_>,
    init: &Expression<'_>,
    scope: usize,
    name: &str,
) {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(init) else {
        return;
    };
    let Expression::Identifier(id) = unwrap_ts_wrappers(&call.callee) else {
        return;
    };
    let local = id.name.as_str();
    if call.optional || visitor.shadowed_locally(local) || !visitor.imported_names.contains(local) {
        return;
    }
    let Some(args) = imported::clause_args(visitor, call) else {
        return;
    };
    visitor.frozen_aliases.push(FrozenAlias {
        scope,
        name: name.to_string(),
        local: local.to_string(),
        span: call.span.start,
        args,
    });
}

pub(super) fn splice_member(
    visitor: &ScopeVisitor<'_>,
    member: &StaticMemberExpression<'_>,
    placeholder: u32,
    kind: SiteKind,
) -> Option<String> {
    imported::constants::namespace::piece(visitor, member, false)
        .or_else(|| member_index(visitor, member, kind).map(sentinel::encode))
        .or_else(|| visitor.spliced_member(member, placeholder))
}

pub(super) fn member_hole(
    visitor: &ScopeVisitor<'_>,
    member: &StaticMemberExpression<'_>,
) -> Option<Vec<Recovered>> {
    marker(
        visitor,
        member_index(visitor, member, SiteKind::PlainClause)?,
        member.span.start,
    )
}

pub(super) fn call_hole(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    local: &str,
) -> Option<Vec<Recovered>> {
    let args = imported::clause_args(visitor, call)?;
    let index = visitor.imported_sites.borrow_mut().intern(
        call.span.start,
        ImportedSite {
            local: local.to_string(),
            member: None,
            kind: SiteKind::PlainClause,
            args,
            value_used: true,
            line: byte_offset_to_line(visitor.source, call.span.start as usize),
            constant: None,
        },
    );
    marker(visitor, index, call.span.start)
}

/// A clause that is only a hole marker is the imported helper's text.
pub(super) fn rejects_clause(sql: &str) -> bool {
    let holes = sentinel::find_all(sql);
    if holes.len() == 1 && holes[0].0.start == 0 && holes[0].0.end == sql.len() {
        false
    } else {
        sentinel::contains(sql)
    }
}

fn marker(visitor: &ScopeVisitor<'_>, index: usize, origin: u32) -> Option<Vec<Recovered>> {
    let sql = sentinel::encode(index);
    let mut value = Recovered::empty(byte_offset_to_line(visitor.source, origin as usize));
    value.origins = vec![origin; sql.len()];
    value.sql = sql;
    value.enumerated = true;
    Some(vec![value])
}

fn member_index(
    visitor: &ScopeVisitor<'_>,
    member: &StaticMemberExpression<'_>,
    kind: SiteKind,
) -> Option<usize> {
    let Expression::Identifier(object) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    let alias = alias(visitor, object.name.as_str())?;
    let index = visitor.imported_sites.borrow_mut().intern(
        member.span.start,
        ImportedSite {
            local: alias.local.clone(),
            member: Some(member.property.name.to_string()),
            kind,
            args: alias.args.clone(),
            value_used: true,
            line: byte_offset_to_line(visitor.source, member.span.start as usize),
            constant: None,
        },
    );
    visitor.site_groups.borrow_mut().insert(index, alias.span);
    Some(index)
}

fn alias<'a>(visitor: &'a ScopeVisitor<'_>, name: &str) -> Option<&'a FrozenAlias> {
    let scope = visitor
        .scopes
        .iter()
        .rposition(|frame| frame.contains_key(name))?;
    visitor
        .frozen_aliases
        .iter()
        .rfind(|alias| alias.scope == scope && alias.name == name)
}
