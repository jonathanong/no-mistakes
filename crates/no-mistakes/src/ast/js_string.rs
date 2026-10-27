use std::borrow::Cow;

use oxc_ast::{
    ast::{
        ComputedMemberExpression, Expression, MemberExpression, ModuleExportName, PropertyKey,
        StringLiteral, TemplateElement,
    },
    StaticPropertyName,
};
use oxc_str::JSStr;

mod legacy;
pub(super) use legacy::raw_has_lone_surrogate;

/// Preserve the string identity used by Oxc 0.153 facts and reports.
/// Ordinary UTF-8 borrows its arena storage. Only lone surrogates require the
/// historical replacement-character marker encoding (including escaped U+FFFD).
pub(crate) fn js_string(value: JSStr<'_>) -> Cow<'_, str> {
    if let Some(text) = value.as_str() {
        return Cow::Borrowed(text);
    }
    let mut text = String::with_capacity(value.len());
    for point in value.chars() {
        match point.to_char() {
            Some('\u{fffd}') => text.push_str("\u{fffd}fffd"),
            Some(character) => text.push(character),
            None => push_surrogate(&mut text, point.to_u32()),
        }
    }
    Cow::Owned(text)
}

/// Avoid `ModuleExportName::name`'s panic on recovered invalid Unicode names.
pub(crate) fn module_export_name<'a>(name: &ModuleExportName<'a>) -> Cow<'a, str> {
    match name {
        ModuleExportName::IdentifierName(identifier) => Cow::Borrowed(identifier.name.as_str()),
        ModuleExportName::IdentifierReference(identifier) => {
            Cow::Borrowed(identifier.name.as_str())
        }
        ModuleExportName::StringLiteral(literal) => js_literal(literal),
    }
}

pub(crate) fn static_property_name(name: StaticPropertyName<'_>) -> Cow<'_, str> {
    match name {
        StaticPropertyName::Borrowed(name) => js_string(name),
        StaticPropertyName::Owned(name) => Cow::Owned(name.into_string()),
    }
}

pub(crate) fn template_quasi_text<'a>(quasi: &TemplateElement<'a>) -> Cow<'a, str> {
    match quasi.value.cooked {
        Some(cooked) => {
            let text = js_string(cooked);
            let raw = quasi.value.raw.as_str();
            if cooked
                .as_str()
                .is_some_and(|value| value.as_ptr() == raw.as_ptr() && value.len() == raw.len())
            {
                return text;
            }
            if legacy::needs_raw_identity(raw) {
                legacy::from_raw(quasi.value.raw.as_str(), text, true)
            } else {
                text
            }
        }
        None => Cow::Borrowed(quasi.value.raw.as_str()),
    }
}

fn push_surrogate(text: &mut String, code: u32) {
    text.push('\u{fffd}');
    for shift in [12, 8, 4, 0] {
        text.push(b"0123456789abcdef"[((code >> shift) & 15) as usize] as char);
    }
}

pub(crate) fn js_literal<'a>(literal: &StringLiteral<'a>) -> Cow<'a, str> {
    js_raw_literal(literal.value, literal.raw.map(|raw| raw.as_str()))
}

pub(super) fn js_raw_literal<'a>(value: JSStr<'a>, raw: Option<&str>) -> Cow<'a, str> {
    let text = js_string(value);
    if let Some(raw) = raw {
        // Unescaped literals borrow the source between their quotes. Avoid
        // scanning those overwhelmingly common strings for unusual escapes.
        if value.as_str().is_some_and(|value| {
            value.as_ptr() == raw.as_ptr().wrapping_add(1) && value.len() + 2 == raw.len()
        }) {
            return text;
        }
        if legacy::needs_raw_identity(raw) {
            if let Some(contents) = raw.get(1..raw.len().saturating_sub(1)) {
                return legacy::from_raw(contents, text, false);
            }
        }
    }
    text
}

/// The legacy lexer flagged non-adjacent surrogate escapes even when the new
/// builder combines them into an ordinary Unicode scalar.
pub(crate) fn js_literal_has_lone_surrogate(literal: &StringLiteral<'_>) -> bool {
    literal.value.has_lone_surrogate() || matches!(js_literal(literal), Cow::Owned(_))
}

pub(crate) fn property_key_name<'a>(key: &PropertyKey<'a>) -> Option<Cow<'a, str>> {
    match key {
        PropertyKey::StringLiteral(literal) => Some(js_literal(literal)),
        PropertyKey::TemplateLiteral(template)
            if template.expressions.is_empty() && template.quasis.len() == 1 =>
        {
            let quasi = template.quasis.first()?;
            quasi.value.cooked?;
            Some(template_quasi_text(quasi))
        }
        _ => key.static_name().map(static_property_name),
    }
}

pub(crate) fn computed_property_name<'a>(
    member: &ComputedMemberExpression<'a>,
) -> Option<Cow<'a, str>> {
    match &member.expression {
        Expression::StringLiteral(literal) => Some(js_literal(literal)),
        Expression::TemplateLiteral(template) if template.quasis.len() == 1 => {
            let quasi = &template.quasis[0];
            quasi.value.cooked?;
            Some(template_quasi_text(quasi))
        }
        _ => member.static_property_name().map(js_string),
    }
}

pub(crate) fn member_property_name<'a>(member: &MemberExpression<'a>) -> Option<Cow<'a, str>> {
    match member {
        MemberExpression::ComputedMemberExpression(member) => computed_property_name(member),
        _ => member.static_property_name().map(js_string),
    }
}

#[cfg(test)]
mod more_tests;
#[cfg(test)]
mod tests;
