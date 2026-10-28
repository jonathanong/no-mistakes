//! Finite strings admitted by a dominating literal Set membership guard.
use oxc_ast::ast::{self, Statement};
use oxc_ast_visit::Visit;
mod intrinsics;
mod parse;
mod uses;
pub(super) use intrinsics::intrinsic_safe;
use oxc_span::GetSpan;

pub(super) struct GuardedCase {
    name: String,
    values: Vec<String>,
    sink: (u32, u32),
    depth: usize,
}
impl GuardedCase {
    pub(super) fn from_function(function: &ast::Function<'_>, depth: usize) -> Option<Self> {
        if function.generator || function.params.rest.is_some() {
            return None;
        }
        let body = function.body.as_ref()?;
        let [Statement::VariableDeclaration(declaration), Statement::IfStatement(guard), sink, tail @ ..] =
            body.statements.as_slice()
        else {
            return None;
        };
        if tail.len() > 1
            || tail
                .first()
                .is_some_and(|s| !matches!(s, Statement::ReturnStatement(_)))
        {
            return None;
        }
        if !matches!(
            sink,
            Statement::ExpressionStatement(_)
                | Statement::VariableDeclaration(_)
                | Statement::ReturnStatement(_)
        ) {
            return None;
        }
        let set = parse::literal_set(declaration)?;
        let membership = parse::throwing_membership(function, guard, &set.name)?;
        let mut uses = Uses {
            set: &set.name,
            value: &membership.argument,
            allowed_set: membership.receiver,
            allowed_constructor: set.constructor,
            set_declaration: set.declaration,
            guard: guard.span.start,
            valid: true,
            nodes: 0,
        };
        uses.visit_function_body(body);
        uses.valid.then_some(Self {
            name: membership.argument,
            values: set.values,
            sink: (sink.span().start, sink.span().end),
            depth,
        })
    }
    pub(super) fn values(&self, name: &str, depth: usize, span: u32) -> Option<&[String]> {
        (self.name == name && self.depth == depth && self.sink.0 <= span && span < self.sink.1)
            .then_some(self.values.as_slice())
    }
}
struct Uses<'a> {
    set: &'a str,
    value: &'a str,
    allowed_set: u32,
    allowed_constructor: u32,
    set_declaration: u32,
    guard: u32,
    valid: bool,
    nodes: usize,
}
