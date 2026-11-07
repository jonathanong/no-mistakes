use super::super::ScopeVisitor;
use crate::codebase::postgres::embedded::EmbeddedSqlCall;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression, ObjectPropertyKind, PropertyKind};
mod state;
pub(in crate::codebase::postgres::embedded::walk) use state::State;

pub(super) fn annotate(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    fact: &mut EmbeddedSqlCall,
) -> Option<bool> {
    let properties = &visitor.executor_options.executor_option_properties;
    if properties.is_empty() {
        return None;
    }
    let mut found = None;
    for argument in call.arguments.iter().skip(1) {
        let Some(expression) = argument.as_expression() else {
            return Some(false);
        };
        if let Expression::Identifier(id) = unwrap_ts_wrappers(expression) {
            if found.is_some() {
                return Some(false);
            }
            let Some(captured) = state::option(visitor, id) else {
                return Some(false);
            };
            found = Some(captured);
        } else {
            match receiver(expression, properties) {
                Some(Some(receiver)) => {
                    if found.is_some() {
                        return Some(false);
                    }
                    let mut selected = EmbeddedSqlCall::default();
                    super::annotate_receiver(visitor, receiver, call.span.start, &mut selected);
                    found = Some(selected);
                }
                Some(None) => return Some(false),
                None => {}
            }
        }
    }
    if let Some(selected) = found {
        state::copy(&selected, fact);
        Some(true)
    } else {
        None
    }
}

pub(super) fn annotate_alias(
    visitor: &ScopeVisitor<'_>,
    receiver: &Expression<'_>,
    fact: &mut EmbeddedSqlCall,
) -> bool {
    state::alias(visitor, receiver, fact)
}

fn receiver<'a>(
    expression: &'a Expression<'a>,
    properties: &[String],
) -> Option<Option<&'a Expression<'a>>> {
    let Expression::ObjectExpression(object) = unwrap_ts_wrappers(expression) else {
        return (!matches!(
            unwrap_ts_wrappers(expression),
            Expression::ArrayExpression(_)
        ))
        .then_some(None);
    };
    let mut selected = None;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return Some(None);
        };
        if property.computed {
            return Some(None);
        }
        let Some(name) = property.key.static_name() else {
            return Some(None);
        };
        if !properties
            .iter()
            .any(|candidate| Some(candidate.as_str()) == name.as_str())
        {
            continue;
        }
        if selected.is_some() || property.method || property.kind != PropertyKind::Init {
            return Some(None);
        }
        selected = Some(match value_receiver(&property.value) {
            Some(value) => Some(value),
            None => return Some(None),
        });
    }
    selected
}

fn value_receiver<'a>(expression: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    let value = unwrap_ts_wrappers(expression);
    match value {
        Expression::Identifier(_) => Some(value),
        Expression::StaticMemberExpression(member) if member.property.name == "query" => {
            Some(unwrap_ts_wrappers(&member.object))
        }
        _ => None,
    }
}
