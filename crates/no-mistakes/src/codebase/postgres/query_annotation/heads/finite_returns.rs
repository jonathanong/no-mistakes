//! Explicit total return paths, without evaluating helper arguments or SQL.
use super::{syntax::Head, Span};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Function, ReturnStatement, Statement};
use oxc_ast_visit::{walk, Visit};
mod proof;
pub(super) use proof::{builder_summary, head_proof};

#[derive(Clone)]
pub(super) enum Candidate {
    ThrowOnly,
    Returns {
        heads: Vec<Return>,
        abrupt: Vec<Span>,
    },
}
#[derive(Clone, Copy)]
pub(super) struct Return {
    pub head: Head,
    pub tag: Option<Span>,
}
impl Candidate {
    pub fn references(&self) -> Vec<Span> {
        match self {
            Self::ThrowOnly => Vec::new(),
            Self::Returns { heads, abrupt } => heads
                .iter()
                .filter_map(|value| value.head.reference().or(value.tag))
                .chain(abrupt.iter().copied())
                .collect(),
        }
    }
    pub fn returned_locals(&self) -> impl Iterator<Item = Span> + '_ {
        let heads = match self {
            Self::Returns { heads, .. } => heads.as_slice(),
            Self::ThrowOnly => &[],
        };
        heads.iter().filter_map(|value| match value.head {
            Head::Reference(reference) => Some(reference),
            _ => None,
        })
    }
}

pub(super) fn function(value: &Function<'_>) -> Option<Candidate> {
    if value.r#async || value.generator {
        return None;
    }
    let body = value.body.as_ref()?;
    if matches!(body.statements.as_slice(), [Statement::ThrowStatement(_)]) {
        return Some(Candidate::ThrowOnly);
    }
    let last = body.statements.last()?;
    let mut heads = Vec::new();
    let mut abrupt = Vec::new();
    match last {
        Statement::ReturnStatement(value) => {
            for statement in &body.statements[..body.statements.len() - 1] {
                if let Statement::IfStatement(branch) = statement {
                    collect_if(branch, &mut heads)?;
                }
            }
            heads.push(return_head(value)?);
            if heads.len() < 2 {
                return None;
            }
        }
        Statement::SwitchStatement(value) => {
            let mut default = false;
            for case in &value.cases {
                let [statement] = case.consequent.as_slice() else {
                    return None;
                };
                if case.test.is_none() {
                    if default {
                        return None;
                    }
                    default = true;
                    match statement {
                        Statement::ThrowStatement(_) => {}
                        Statement::ReturnStatement(value) => {
                            abrupt.push(super::helper_heads::callee(value.argument.as_ref()?)?);
                        }
                        _ => return None,
                    }
                } else {
                    match statement {
                        Statement::ReturnStatement(value) => heads.push(return_head(value)?),
                        Statement::ThrowStatement(_) => {}
                        _ => return None,
                    }
                }
            }
            if !default || heads.is_empty() {
                return None;
            }
        }
        _ => return None,
    }
    let mut count = Returns(0);
    count.visit_function_body(body);
    (count.0 == heads.len() + abrupt.len()).then_some(Candidate::Returns { heads, abrupt })
}

fn collect_if(value: &oxc_ast::ast::IfStatement<'_>, heads: &mut Vec<Return>) -> Option<()> {
    // Guard conditions and non-return branch bodies are intentionally ignored: only literal, tag,
    // or append-safe local heads are proven, and the return-count check rejects hidden returns.
    if let Some(head) = branch_return(&value.consequent)? {
        heads.push(head);
    }
    if let Some(alternate) = &value.alternate {
        if let Some(head) = branch_return(alternate)? {
            heads.push(head);
        }
    }
    Some(())
}
fn branch_return(value: &Statement<'_>) -> Option<Option<Return>> {
    match value {
        Statement::ReturnStatement(value) => Some(Some(return_head(value)?)),
        // Prefix statements cannot hide a return: function() compares every
        // same-function return against the collected terminal alternatives.
        Statement::BlockStatement(block) => match block.body.last() {
            Some(Statement::ReturnStatement(value)) => Some(Some(return_head(value)?)),
            _ => Some(None),
        },
        _ => Some(None),
    }
}
fn return_head(value: &ReturnStatement<'_>) -> Option<Return> {
    let expression = unwrap_ts_wrappers(value.argument.as_ref()?);
    let head = super::syntax::head(expression);
    let tag = match head {
        Head::Reference(_) | Head::Literal => None,
        Head::Text(value) if !value.tagged && !value.appended => None,
        Head::Tag | Head::AppendTag | Head::AppendPrefixTag | Head::Text(_) => {
            Some(super::trusted_append::tag(expression)?)
        }
        _ => return None,
    };
    Some(Return { head, tag })
}
struct Returns(usize);
impl<'a> Visit<'a> for Returns {
    fn visit_function(&mut self, _: &Function<'a>, _: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &oxc_ast::ast::ArrowFunctionExpression<'a>) {}
    fn visit_return_statement(&mut self, value: &ReturnStatement<'a>) {
        self.0 += 1;
        walk::walk_return_statement(self, value);
    }
}
