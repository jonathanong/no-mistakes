use oxc_allocator::Allocator;
use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{walk, Visit};
use oxc_span::SourceType;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrpcCallFact {
    pub path: String,
}

#[cfg(test)]
pub fn extract_trpc_calls(source: &str) -> Vec<TrpcCallFact> {
    let allocator = Allocator::default();
    let ret = crate::ast::parse(
        std::path::Path::new("trpc-calls.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    extract_trpc_calls_from_program(&ret.program)
}

pub fn extract_trpc_calls_from_program(program: &Program<'_>) -> Vec<TrpcCallFact> {
    let mut visitor = CallVisitor { calls: Vec::new() };
    visitor.visit_program(program);
    visitor
        .calls
        .sort_by(|left, right| left.path.cmp(&right.path));
    visitor
        .calls
        .dedup_by(|left, right| left.path == right.path);
    visitor.calls
}

struct CallVisitor {
    calls: Vec<TrpcCallFact>,
}

impl<'a> Visit<'a> for CallVisitor {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(path) = procedure_path_from_call(call) {
            self.calls.push(TrpcCallFact { path });
        }
        walk::walk_call_expression(self, call);
    }
}

fn procedure_path_from_call(call: &CallExpression<'_>) -> Option<String> {
    let Expression::StaticMemberExpression(terminal) = &call.callee else {
        return None;
    };
    if !matches!(
        terminal.property.name.as_str(),
        "query" | "mutate" | "mutation"
    ) {
        return None;
    }
    let mut segments = Vec::new();
    let mut current = &terminal.object;
    loop {
        match current {
            Expression::StaticMemberExpression(member) => {
                segments.push(member.property.name.to_string());
                current = &member.object;
            }
            Expression::Identifier(_) => break,
            _ => return None,
        }
    }
    if segments.len() < 2 {
        return None;
    }
    segments.reverse();
    Some(segments.join("."))
}

#[cfg(test)]
mod tests;
