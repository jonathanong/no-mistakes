use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::*;

mod writes;
pub(super) use writes::{ParameterFields, Writes};

fn origins(visitor: &ScopeVisitor<'_>, call: &CallExpression<'_>) -> Vec<(u32, u32)> {
    let Some(Expression::TaggedTemplateExpression(tag)) = call
        .arguments
        .first()
        .and_then(Argument::as_expression)
        .map(unwrap_ts_wrappers)
    else {
        return Vec::new();
    };
    let mut origins = Vec::new();
    for expr in &tag.quasi.expressions {
        match expr {
            Expression::Identifier(id) => {
                let Some(binding) = visitor.lookup(id.name.as_str()) else {
                    return Vec::new();
                };
                if let Some(identity) = binding.value_identity {
                    let declaration = identity.get() - 1;
                    if visitor.stable_primitives.contains(&declaration) {
                        origins.push((id.span.start, declaration));
                    }
                }
            }
            Expression::StaticMemberExpression(member) => {
                let Some(identity) = visitor.stable_member_identity(member, call.span, tag.span)
                else {
                    return Vec::new();
                };
                origins.push((member.span.start, identity));
            }
            Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_) => {}
            _ => return Vec::new(),
        }
    }
    let counts = origins
        .iter()
        .fold(FxHashMap::<u32, usize>::default(), |mut counts, (_, id)| {
            *counts.entry(*id).or_default() += 1;
            counts
        });
    origins.retain(|(_, id)| counts[id] > 1);
    origins
}

pub(super) fn annotate(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    fact: &mut super::super::EmbeddedSqlCall,
) {
    let ids: FxHashMap<_, _> = origins(visitor, call).into_iter().collect();
    if ids.is_empty() {
        return;
    }
    let Some(argument) = call.arguments.first().and_then(Argument::as_expression) else {
        return;
    };
    let Some(raw) = super::super::unpublished_sql_text(argument) else {
        return;
    };
    let source =
        super::super::source_positions::origins::expression(argument, visitor.source, &raw);
    let offsets = super::resolve::rewrite_origins(&raw, &source, 0, true);
    let (published, positions) = super::super::publish_placeholders_with_positions(raw);
    if fact.sql_text.as_deref() != Some(published.as_str()) {
        return;
    }
    let positions: FxHashSet<_> = positions.into_iter().collect();
    let (mut line, mut column) = (1, 1);
    for (offset, character) in published.char_indices() {
        if positions.contains(&(line, column)) {
            if let Some(id) = offsets.get(offset).and_then(|origin| ids.get(origin)) {
                fact.bound_value_origins.push((line, column, *id));
            }
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    fact.bound_value_sql = Some(published);
}
