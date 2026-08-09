use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::{Argument, CallExpression, Expression, Function, Program};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

/// A direct identifier or one-level static member call recorded during the
/// shared TypeScript fact pass. Query consumers select the relevant callee
/// names without reparsing callers.
#[derive(Debug, Clone)]
pub struct CallSiteFact {
    pub callee: String,
    pub line: u32,
    pub caller: Option<String>,
    pub static_arg: Option<String>,
    pub arg_count: usize,
    pub has_spread: bool,
    pub args: Vec<&'static str>,
}

pub(crate) fn collect_call_site_facts(program: &Program<'_>, source: &str) -> Vec<CallSiteFact> {
    let mut visitor = CallSiteVisitor {
        source,
        scope: Vec::new(),
        sites: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.sites
}

struct CallSiteVisitor<'a> {
    source: &'a str,
    scope: Vec<String>,
    sites: Vec<CallSiteFact>,
}

fn callee_name(callee: &Expression<'_>) -> Option<String> {
    match callee {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        Expression::StaticMemberExpression(member) => match &member.object {
            Expression::Identifier(object) => Some(format!(
                "{}.{}",
                object.name.as_str(),
                member.property.name.as_str()
            )),
            _ => None,
        },
        _ => None,
    }
}

fn static_first_string_arg(call: &CallExpression<'_>) -> Option<String> {
    match call.arguments.first()? {
        Argument::StringLiteral(string) => Some(string.value.to_string()),
        Argument::TemplateLiteral(template) if template.expressions.is_empty() => Some(
            template
                .quasis
                .iter()
                .map(|quasi| {
                    quasi
                        .value
                        .cooked
                        .as_ref()
                        .unwrap_or(&quasi.value.raw)
                        .as_str()
                })
                .collect(),
        ),
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
        if let Some(callee) = callee_name(&call.callee) {
            self.sites.push(CallSiteFact {
                callee,
                line: byte_offset_to_line(self.source, call.span.start as usize),
                caller: self.scope.last().cloned(),
                static_arg: static_first_string_arg(call),
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
