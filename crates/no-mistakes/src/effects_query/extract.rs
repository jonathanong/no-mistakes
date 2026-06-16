//! AST extraction for the `effects` query: parse one reachable file and collect
//! call/constructor sites whose name is in the configured set, with line numbers
//! and a best-effort enclosing-function (`caller`) attribution.

use std::collections::HashMap;
use std::path::Path;

use oxc_ast::ast::{CallExpression, Expression, Function, NewExpression, VariableDeclarator};
use oxc_ast_visit::{walk, Visit};

use crate::codebase::ts_source::{byte_offset_to_line, relative_slash_path};

use super::EffectCallSite;

pub(super) fn scan_file(
    root: &Path,
    path: &Path,
    depth: usize,
    names: &HashMap<String, Option<String>>,
) -> Vec<EffectCallSite> {
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let rel = relative_slash_path(root, path);
    crate::ast::with_program(path, &source, |program, source| {
        let mut visitor = EffectVisitor {
            source,
            names,
            caller_stack: Vec::new(),
            hits: Vec::new(),
        };
        visitor.visit_program(program);
        visitor
            .hits
            .into_iter()
            .map(|hit| EffectCallSite {
                file: rel.clone(),
                line: hit.line,
                callee: hit.callee,
                category: hit.category,
                caller: hit.caller,
                depth,
            })
            .collect()
    })
    .unwrap_or_default()
}

struct RawHit {
    line: usize,
    callee: String,
    category: Option<String>,
    caller: Option<String>,
}

struct EffectVisitor<'a> {
    source: &'a str,
    names: &'a HashMap<String, Option<String>>,
    caller_stack: Vec<String>,
    hits: Vec<RawHit>,
}

impl EffectVisitor<'_> {
    fn record(&mut self, callee: &Expression<'_>, byte_offset: u32) {
        if let Some((name, category)) = match_callee(callee, self.names) {
            self.hits.push(RawHit {
                line: byte_offset_to_line(self.source, byte_offset as usize) as usize,
                callee: name,
                category,
                caller: self.caller_stack.last().cloned(),
            });
        }
    }
}

impl<'a> Visit<'a> for EffectVisitor<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.record(&call.callee, call.span.start);
        walk::walk_call_expression(self, call);
    }

    fn visit_new_expression(&mut self, new: &NewExpression<'a>) {
        self.record(&new.callee, new.span.start);
        walk::walk_new_expression(self, new);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        let pushed = function
            .id
            .as_ref()
            .map(|id| id.name.to_string())
            .inspect(|name| self.caller_stack.push(name.clone()))
            .is_some();
        walk::walk_function(self, function, flags);
        if pushed {
            self.caller_stack.pop();
        }
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let name = declarator_function_name(declarator);
        if let Some(name) = &name {
            self.caller_stack.push(name.clone());
        }
        walk::walk_variable_declarator(self, declarator);
        if name.is_some() {
            self.caller_stack.pop();
        }
    }
}

/// The binding name of a `const NAME = () => ...` / `const NAME = function...`
/// declarator, used to attribute nested effect calls to a caller.
fn declarator_function_name(declarator: &VariableDeclarator<'_>) -> Option<String> {
    let is_function = matches!(
        declarator.init,
        Some(Expression::ArrowFunctionExpression(_)) | Some(Expression::FunctionExpression(_))
    );
    if !is_function {
        return None;
    }
    match &declarator.id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
        _ => None,
    }
}

fn match_callee(
    callee: &Expression<'_>,
    names: &HashMap<String, Option<String>>,
) -> Option<(String, Option<String>)> {
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
            // Try the qualified `object.property` form before the bare property
            // so a configured `cache.invalidate` wins over a bare `invalidate`.
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
