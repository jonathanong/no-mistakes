use crate::codebase::ts_source::byte_offset_to_line;
use oxc::ast::ast::{Argument, CallExpression, Expression, Function, Program};
use oxc::ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::HashSet;

/// One raw call site found in a file, before it is made root-relative.
pub(crate) struct RawCallSite {
    pub line: u32,
    pub caller: Option<String>,
    pub arg_count: usize,
    pub has_spread: bool,
    pub args: Vec<&'static str>,
}

/// Collect call sites of any function whose local name is in `targets`. Only
/// direct identifier callees (`foo(...)`) match — namespace member calls
/// (`ns.foo()`) and indirect aliases (`const f = foo; f()`) are not resolved.
pub(crate) fn collect_call_sites(
    program: &Program<'_>,
    source: &str,
    targets: &HashSet<String>,
) -> Vec<RawCallSite> {
    let mut visitor = CallSiteVisitor {
        source,
        targets,
        scope: Vec::new(),
        sites: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.sites
}

struct CallSiteVisitor<'a> {
    source: &'a str,
    targets: &'a HashSet<String>,
    scope: Vec<String>,
    sites: Vec<RawCallSite>,
}

fn callee_name<'a>(callee: &'a Expression<'a>) -> Option<&'a str> {
    match callee {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

/// Coarse syntactic shape of one argument — no type inference.
fn arg_tag(arg: &Argument<'_>) -> &'static str {
    match arg {
        Argument::SpreadElement(_) => "spread",
        Argument::StringLiteral(_) | Argument::TemplateLiteral(_) => "string",
        Argument::NumericLiteral(_) | Argument::BigIntLiteral(_) => "number",
        Argument::BooleanLiteral(_) => "boolean",
        Argument::NullLiteral(_) => "null",
        Argument::Identifier(_) => "identifier",
        Argument::ObjectExpression(_) => "object",
        Argument::ArrayExpression(_) => "array",
        Argument::ArrowFunctionExpression(_) | Argument::FunctionExpression(_) => "arrow",
        Argument::CallExpression(_) => "call",
        _ => "other",
    }
}

impl<'a> Visit<'a> for CallSiteVisitor<'a> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        let name = function.id.as_ref().map(|id| id.name.as_str().to_string());
        if let Some(name) = &name {
            self.scope.push(name.clone());
        }
        walk::walk_function(self, function, flags);
        if name.is_some() {
            self.scope.pop();
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if callee_name(&call.callee).is_some_and(|name| self.targets.contains(name)) {
            self.sites.push(RawCallSite {
                line: byte_offset_to_line(self.source, call.span.start as usize),
                caller: self.scope.last().cloned(),
                arg_count: call.arguments.len(),
                has_spread: call
                    .arguments
                    .iter()
                    .any(|arg| matches!(arg, Argument::SpreadElement(_))),
                args: call.arguments.iter().map(arg_tag).collect(),
            });
        }
        walk::walk_call_expression(self, call);
    }
}
