//! Computed `object[key]()` calls. A const string key can name a registry member.
use super::super::PathProvenance;
use oxc_ast::ast::*;
use oxc_semantic::Semantic;

pub(super) fn from_call(
    semantic: &Semantic<'_>,
    call: &CallExpression<'_>,
    out: &mut PathProvenance,
) {
    let Expression::ComputedMemberExpression(member) = call.callee.get_inner_expression() else {
        return;
    };
    let Expression::Identifier(object) = member.object.get_inner_expression() else {
        return;
    };
    let object_name = object.name.to_string();
    let label = call_label(semantic, call, &object_name);
    out.registry_dispatches.push((
        call.span.start,
        object_name,
        const_string(semantic, &member.expression),
        label,
    ));
}

/// Callee text at the call, so a later report can name the dispatch after source text is dropped.
fn call_label(semantic: &Semantic<'_>, call: &CallExpression<'_>, object: &str) -> String {
    let start = call.span.start as usize;
    let text = semantic.source_text().get(start..).unwrap_or("");
    let callee = text
        .split_once('(')
        .map(|(head, _)| head.trim())
        .unwrap_or("");
    let head = callee.split(['.', '[']).next().unwrap_or("");
    if is_name(head) && !callee.is_empty() && callee.len() <= 80 {
        callee.to_string()
    } else {
        format!("{object}[...]")
    }
}

pub(crate) fn is_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c == '_' || c == '$' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric())
}

/// A string literal, or a `const` binding initialized with one and never written.
fn const_string(semantic: &Semantic<'_>, expression: &Expression<'_>) -> Option<String> {
    match expression.get_inner_expression() {
        Expression::StringLiteral(text) => Some(crate::ast::js_literal(text).into_owned()),
        Expression::Identifier(id) => bound_string(semantic, id),
        _ => None,
    }
}

fn bound_string(semantic: &Semantic<'_>, id: &IdentifierReference<'_>) -> Option<String> {
    let reference = id.reference_id.get()?;
    let symbol = semantic.scoping().get_reference(reference).symbol_id()?;
    if !semantic.scoping().symbol_flags(symbol).is_const_variable()
        || semantic
            .scoping()
            .get_resolved_references(symbol)
            .any(|reference| reference.flags().is_write())
    {
        return None;
    }
    let declarator = semantic
        .nodes()
        .kind(semantic.scoping().symbol_declaration(symbol))
        .as_variable_declarator()?;
    match declarator.init.as_ref()?.get_inner_expression() {
        Expression::StringLiteral(text) => Some(crate::ast::js_literal(text).into_owned()),
        _ => None,
    }
}
