use super::super::extract_job_id::{is_imported_callee, referenced_symbol};
use super::super::extract_visitor::QueueVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers as unwrap;
use crate::config::v2::schema::{EnqueuePropertySelector, EnqueueSelector};
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ObjectExpression, ObjectPropertyKind, PropertyKind,
};
use oxc_span::GetSpan;

#[derive(Clone, Copy)]
pub(super) enum Selected<'a> {
    Missing,
    Unknown,
    Value(&'a Expression<'a>),
}

impl<'a> Selected<'a> {
    pub(super) fn value(self) -> Option<&'a Expression<'a>> {
        match self {
            Self::Value(expr) => Some(expr),
            Self::Missing | Self::Unknown => None,
        }
    }

    pub(super) fn argument(call: &'a CallExpression<'a>, index: usize) -> Self {
        if call
            .arguments
            .iter()
            .take(index.saturating_add(1))
            .any(|arg| matches!(arg, Argument::SpreadElement(_)))
        {
            return Self::Unknown;
        }
        call.arguments
            .get(index)
            .and_then(Argument::as_expression)
            .map_or(Self::Missing, |expr| Self::Value(unwrap(expr)))
    }

    pub(super) fn select(
        visitor: &QueueVisitor<'a>,
        call: &'a CallExpression<'a>,
        selector: Option<&EnqueueSelector>,
    ) -> Self {
        match selector {
            None => Self::Missing,
            Some(EnqueueSelector::Positional(index)) => Self::argument(call, *index),
            Some(EnqueueSelector::Property(EnqueuePropertySelector { arg, property })) => {
                let Some(Expression::ObjectExpression(object)) = Self::argument(call, *arg)
                    .value()
                    .and_then(|expr| local_value(visitor, expr))
                else {
                    return Self::Unknown;
                };
                closed_property(object, property)
                    .flatten()
                    .map_or(Self::Unknown, |expr| Self::Value(unwrap(expr)))
            }
        }
    }
}

pub(super) fn local_value<'a>(
    visitor: &QueueVisitor<'a>,
    mut expr: &'a Expression<'a>,
) -> Option<&'a Expression<'a>> {
    if !visitor
        .wrapper_mutations
        .stable(expr, &visitor.object_bindings)
    {
        return None;
    }
    loop {
        expr = unwrap(expr);
        let Expression::Identifier(id) = expr else {
            return Some(expr);
        };
        let symbol = referenced_symbol(visitor, id)?;
        let scoping = visitor.scoping?;
        if !scoping.symbol_flags(symbol).is_const_variable()
            || scoping
                .get_resolved_references(symbol)
                .any(|reference| reference.is_write())
        {
            return None;
        }
        let init = visitor
            .object_bindings
            .lookup(id.name.as_str(), id.span.start)?
            .init?;
        if init.span().end > id.span.start {
            return None;
        }
        expr = init;
    }
}

pub(super) fn imported_local<'a>(
    visitor: &QueueVisitor<'a>,
    expr: &'a Expression<'a>,
) -> Option<String> {
    let Expression::Identifier(id) = expr else {
        return None;
    };
    (visitor
        .wrapper_mutations
        .stable(expr, &visitor.object_bindings)
        && is_imported_callee(visitor, expr)
        && visitor.facts.string_imports.contains_key(id.name.as_str()))
    .then(|| id.name.to_string())
}

pub(super) fn closed_property<'a>(
    object: &'a ObjectExpression<'a>,
    key: &str,
) -> Option<Option<&'a Expression<'a>>> {
    let mut value = None;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property.kind != PropertyKind::Init {
            return None;
        }
        let name = crate::ast::property_key_name(&property.key)?;
        if name == "__proto__" {
            return None;
        }
        if name == key {
            value = Some(&property.value);
        }
    }
    Some(value)
}
