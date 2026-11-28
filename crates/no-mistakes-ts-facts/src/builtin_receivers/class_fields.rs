//! Instance fields whose annotations name a built-in value.
//! `this.field` is that value only in instance methods, their arrows, and
//! instance field initializers. Nested functions and static methods rebind `this`.
use super::names::{Names, ValueKind};
use super::values;
use crate::fx::FxHashMap;
use oxc_ast::ast::*;

pub(super) struct ClassFrame {
    pub(super) fields: FxHashMap<String, ValueKind>,
    pub(super) allowed: Vec<u32>,
}

pub(super) fn fields(names: &Names, class: &Class<'_>) -> FxHashMap<String, ValueKind> {
    let mut recorded: FxHashMap<String, Option<ValueKind>> = FxHashMap::default();
    for element in &class.body.body {
        let ClassElement::PropertyDefinition(property) = element else {
            continue;
        };
        if property.r#static {
            continue;
        }
        let Some(annotation) = &property.type_annotation else {
            continue;
        };
        let Some(name) = property
            .key
            .static_name()
            .and_then(|name| name.as_str().map(ToOwned::to_owned))
        else {
            continue;
        };
        record(
            &mut recorded,
            &name,
            names.type_kind(&annotation.type_annotation),
        );
    }
    recorded
        .into_iter()
        .filter_map(|(name, kind)| kind.map(|kind| (name, kind)))
        .collect()
}

/// `Some(None)` is a leaf built-in. `Some(Some(slots))` invokes those arguments.
pub(super) fn instance_callbacks(
    classes: &[ClassFrame],
    this_depth: u32,
    object: &Expression<'_>,
    method: &str,
) -> Option<Option<&'static [usize]>> {
    let Expression::StaticMemberExpression(member) = object.without_parentheses() else {
        return None;
    };
    if !matches!(
        member.object.without_parentheses(),
        Expression::ThisExpression(_)
    ) {
        return None;
    }
    let kind = field_kind(classes, this_depth, member.property.name.as_str())?;
    if !supports(kind, method) {
        return None;
    }
    Some(match kind {
        ValueKind::Array { .. } if values::ARRAY_CALLBACKS.contains(&method) => Some(&[0]),
        ValueKind::String if matches!(method, "replace" | "replaceAll") => Some(&[1]),
        _ => None,
    })
}

fn field_kind(classes: &[ClassFrame], depth: u32, name: &str) -> Option<ValueKind> {
    let frame = classes
        .iter()
        .rev()
        .find(|frame| frame.allowed.contains(&depth))?;
    frame.fields.get(name).copied()
}

fn supports(kind: ValueKind, method: &str) -> bool {
    if values::callback_name(method) {
        return match kind {
            ValueKind::Array { .. } => values::ARRAY_CALLBACKS.contains(&method),
            ValueKind::String => matches!(method, "replace" | "replaceAll"),
            _ => false,
        };
    }
    kind != ValueKind::Namespace
}

fn record(fields: &mut FxHashMap<String, Option<ValueKind>>, name: &str, kind: Option<ValueKind>) {
    match fields.get(name) {
        None => {
            fields.insert(name.to_owned(), kind);
        }
        Some(existing) if *existing == kind => {}
        Some(_) => {
            fields.insert(name.to_owned(), None);
        }
    }
}
