use super::EffectCallFact;
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::{Expression, VariableDeclarator};
use std::collections::HashMap;

pub(super) type EffectNames = HashMap<String, Option<String>>;

pub(super) struct EffectSink<'a> {
    pub source: &'a str,
    pub names: &'a EffectNames,
    pub caller: Option<&'a str>,
    pub hits: &'a mut Vec<EffectCallFact>,
}

pub(super) fn record_effect(sink: EffectSink<'_>, callee: &Expression<'_>, byte_offset: u32) {
    if let Some((name, category)) = match_callee(callee, sink.names) {
        sink.hits.push(EffectCallFact {
            line: byte_offset_to_line(sink.source, byte_offset as usize) as usize,
            callee: name,
            category,
            caller: sink.caller.map(str::to_string),
        });
    }
}

pub(super) fn declarator_function_name<'a>(declarator: &VariableDeclarator<'a>) -> Option<&'a str> {
    let is_function = matches!(
        declarator.init,
        Some(Expression::ArrowFunctionExpression(_)) | Some(Expression::FunctionExpression(_))
    );
    if !is_function {
        return None;
    }
    match &declarator.id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

fn match_callee(callee: &Expression<'_>, names: &EffectNames) -> Option<(String, Option<String>)> {
    for candidate in callee_candidates(callee) {
        if let Some(category) = names.get(&candidate) {
            return Some((candidate, category.clone()));
        }
    }
    None
}

fn callee_candidates(expr: &Expression<'_>) -> Vec<String> {
    match expr {
        Expression::Identifier(ident) => vec![ident.name.to_string()],
        Expression::ParenthesizedExpression(parenthesized) => {
            callee_candidates(&parenthesized.expression)
        }
        Expression::StaticMemberExpression(member) => {
            let property = member.property.name.to_string();
            let mut candidates = Vec::new();
            if let Expression::Identifier(object) = &member.object {
                candidates.push(format!("{}.{}", object.name, property));
            }
            candidates.push(property);
            candidates
        }
        _ => Vec::new(),
    }
}
