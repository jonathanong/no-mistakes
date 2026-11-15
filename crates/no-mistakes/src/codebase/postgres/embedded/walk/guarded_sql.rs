//! Finite strings admitted by a dominating literal Set membership guard.
use oxc_ast::ast::{self, Statement};
use oxc_ast_visit::Visit;
mod intrinsics;
mod parse;
mod shadow;
mod uses;
pub(super) use intrinsics::intrinsic_safe;
use oxc_span::GetSpan;
pub(in crate::codebase::postgres::embedded::walk) use shadow::shadows_set;

pub(super) struct GuardedCase {
    name: String,
    values: Vec<String>,
    sink: (u32, u32),
    depth: usize,
    decision: u64,
}
impl GuardedCase {
    pub(super) fn from_function(function: &ast::Function<'_>, depth: usize) -> Option<Self> {
        if function.generator {
            return None;
        }
        Self::from_parts(&function.params, function.body.as_ref()?, depth)
    }
    fn from_parts(
        params: &ast::FormalParameters<'_>,
        body: &ast::FunctionBody<'_>,
        depth: usize,
    ) -> Option<Self> {
        if params.rest.is_some()
            || params.items.iter().any(|parameter| {
                parameter.initializer.is_some()
                    || !parameter.decorators.is_empty()
                    || !matches!(parameter.pattern, ast::BindingPattern::BindingIdentifier(_))
            })
        {
            return None;
        }
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
        let membership = parse::throwing_membership(params, guard, &set.name)?;
        let mut uses = Uses {
            set: &set.name,
            value: &membership.argument,
            allowed_set: membership.receiver,
            allowed_constructor: set.constructor,
            set_declaration: set.declaration,
            valid: true,
            nodes: 0,
        };
        uses.visit_function_body(body);
        uses.valid.then_some(Self {
            name: membership.argument,
            values: set.values,
            sink: (sink.span().start, sink.span().end),
            depth,
            decision: u64::from(guard.span.start) | (1u64 << 61),
        })
    }
    pub(in crate::codebase::postgres::embedded::walk) fn returned_values(
        params: &ast::FormalParameters<'_>,
        body: &ast::FunctionBody<'_>,
    ) -> Option<Vec<String>> {
        let proof = Self::from_parts(params, body, 0)?;
        let Some(Statement::ReturnStatement(returned)) = body.statements.last() else {
            return None;
        };
        let Some(ast::Expression::Identifier(id)) = returned.argument.as_ref() else {
            return None;
        };
        (id.name.as_str() == proof.name
            && proof.sink.0 <= id.span.start
            && id.span.start < proof.sink.1)
            .then_some(proof.values)
    }
    pub(super) fn values(&self, name: &str, depth: usize, span: u32) -> Option<(&[String], u64)> {
        (self.name == name && self.depth == depth && self.sink.0 <= span && span < self.sink.1)
            .then_some((self.values.as_slice(), self.decision))
    }
}
struct Uses<'a> {
    set: &'a str,
    value: &'a str,
    allowed_set: u32,
    allowed_constructor: u32,
    set_declaration: u32,
    valid: bool,
    nodes: usize,
}
