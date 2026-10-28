//! Bounded syntax proof for a local helper's fixed return head.
use super::{
    bindings::Bindings,
    syntax::{Head, Syntax},
    HeadProof, Span,
};
use crate::codebase::ts_source::module_facts::ModuleBinding;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrowFunctionBody, ArrowFunctionExpression, Expression, Function, FunctionBody, Statement,
};
use oxc_ast_visit::{walk, Visit};
mod imports;
mod values;
pub(crate) use imports::{module_projection, Kind, Pending};

#[derive(Clone, Copy)]
pub(super) struct Summary {
    pub head: Head,
    pub tag: Option<Span>,
}

pub(super) fn callee(expression: &Expression<'_>) -> Option<Span> {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    if call.optional
        || call
            .arguments
            .iter()
            .any(|arg| arg.as_expression().is_none())
    {
        return None;
    }
    let Expression::Identifier(id) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    Some((id.span.start, id.span.end))
}

pub(super) fn function(value: &Function<'_>) -> Option<Summary> {
    if value.r#async || value.generator {
        return None;
    }
    returned(value.body.as_ref()?)
}

pub(super) fn arrow(value: &ArrowFunctionExpression<'_>) -> Option<Summary> {
    if value.r#async {
        return None;
    }
    match &value.body {
        ArrowFunctionBody::FunctionBody(body) => returned(body),
        expression => expression.as_expression().and_then(return_expression),
    }
}

fn returned(body: &FunctionBody<'_>) -> Option<Summary> {
    let Statement::ReturnStatement(last) = body.statements.last()? else {
        return None;
    };
    let mut counter = Returns(0);
    counter.visit_function_body(body);
    if counter.0 != 1 {
        return None;
    }
    return_expression(last.argument.as_ref()?)
}

fn return_expression(expression: &Expression<'_>) -> Option<Summary> {
    let head = super::syntax::head(expression);
    match head {
        Head::Reference(_) => Some(Summary { head, tag: None }),
        Head::Literal => Some(Summary { head, tag: None }),
        Head::Text(value) if !value.tagged && !value.appended => Some(Summary { head, tag: None }),
        Head::Tag | Head::AppendTag | Head::AppendPrefixTag | Head::Text(_) => {
            let tag = super::trusted_append::tag(expression)?;
            Some(Summary {
                head,
                tag: Some(tag),
            })
        }
        _ => None,
    }
}

struct Returns(usize);
impl<'a> Visit<'a> for Returns {
    fn visit_function(&mut self, _: &Function<'a>, _: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
    fn visit_return_statement(&mut self, value: &oxc_ast::ast::ReturnStatement<'a>) {
        self.0 += 1;
        walk::walk_return_statement(self, value);
    }
}

pub(super) fn proof(syntax: &Syntax, bindings: &Bindings<'_>, call: Span) -> Option<HeadProof> {
    let reference = *syntax.helper_arguments.get(&call)?;
    let owner = bindings.owner(reference)?;
    if owner.imported {
        return None;
    }
    summary_proof(syntax, bindings, owner)
}

pub(super) fn summary_proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
) -> Option<HeadProof> {
    let declaration = (owner.span.start, owner.span.end);
    if !bindings.declaration_safe(declaration) {
        return None;
    }
    let Some(summary) = syntax.helper_returns.get(&declaration) else {
        return super::finite_returns::head_proof(syntax, bindings, owner);
    };
    // A callable changed through an assignment, update or unsupported write is not stable.
    if owner.references.iter().any(|usage| {
        let span = (usage.span.start, usage.span.end);
        syntax.assignments.writes.contains_key(&span)
            || syntax.assignments.unsupported_writes.contains(&span)
    }) {
        return None;
    }
    if let Some(tag) = summary.tag {
        let tag_owner = bindings.owner(tag)?;
        if !tag_owner.imported || !bindings.trusted_import(tag_owner.id) {
            return None;
        }
    }
    if let Head::Reference(reference) = summary.head {
        return bindings.returned_local_prefix(syntax, reference);
    }
    Some(match summary.head {
        Head::AppendPrefixTag => HeadProof::Literal,
        Head::Text(value) if value.opaque => value.current,
        head => head.proof(false),
    })
}
