//! Sequential setup precedes the fresh heap; no control-flow interpretation.
use super::Candidate;
use crate::codebase::postgres::query_annotation::heads::{append, effects, syntax};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

pub(in super::super) fn function(value: &Function<'_>) -> Option<Candidate> {
    if !value.r#async
        || value.generator
        || value.params.rest.is_some()
        || value.params.items.iter().any(|p| {
            p.initializer.is_some()
                || !p.decorators.is_empty()
                || !matches!(p.pattern, BindingPattern::BindingIdentifier(_))
        })
    {
        return None;
    }
    let body = value.body.as_ref()?;
    if body.statements.len() > 64 {
        return None;
    }
    let (last, setup) = body.statements.split_last()?;
    let Statement::ReturnStatement(ret) = last else {
        return None;
    };
    let Expression::Identifier(returned) = unwrap_ts_wrappers(ret.argument.as_ref()?) else {
        return None;
    };
    let mut candidate = None;
    for statement in setup {
        if let Some(current) = candidate.as_mut() {
            let current: &mut Candidate = current;
            let Statement::ExpressionStatement(statement) = statement else {
                return None;
            };
            if let Some(reference) = discarded(&statement.expression) {
                current.appends.push(reference);
            } else {
                current
                    .effects
                    .push(effects::call(&statement.expression)[0]?.builder);
            }
            continue;
        }
        match statement {
            Statement::VariableDeclaration(declaration) => {
                // Resource disposal runs after the return and is not sequential setup.
                if !matches!(
                    declaration.kind,
                    VariableDeclarationKind::Const
                        | VariableDeclarationKind::Let
                        | VariableDeclarationKind::Var
                ) {
                    return None;
                }
                // Only a single direct const native-root candidate can own the heap.
                if !declaration.kind.is_const() || declaration.declarations.len() != 1 {
                    continue;
                }
                let binding = &declaration.declarations[0];
                let BindingPattern::BindingIdentifier(id) = &binding.id else {
                    continue;
                };
                if id.name != returned.name {
                    continue;
                }
                let init = binding.init.as_ref()?;
                let (tag, proof) = root(init)?;
                if proof == super::super::HeadProof::Unproven {
                    return None;
                }
                candidate = Some(Candidate {
                    function: span(value.span),
                    body: span(body.span),
                    declaration: span(id.span),
                    tag,
                    returned: span(returned.span),
                    initializer_start: init.span().start,
                    proof,
                    appends: Vec::new(),
                    effects: Vec::new(),
                });
            }
            Statement::ExpressionStatement(_) => {}
            _ => return None,
        }
    }
    candidate
}
fn span(value: oxc_span::Span) -> super::super::Span {
    (value.start, value.end)
}

fn root(mut expression: &Expression<'_>) -> Option<(super::super::Span, super::super::HeadProof)> {
    for _ in 0..=8 {
        match unwrap_ts_wrappers(expression) {
            Expression::CallExpression(call) => expression = append::prefix_receiver(call)?,
            Expression::TaggedTemplateExpression(tagged) => {
                let Expression::Identifier(tag) = unwrap_ts_wrappers(&tagged.tag) else {
                    return None;
                };
                return Some((span(tag.span), syntax::head(expression).proof(true)));
            }
            _ => return None,
        }
    }
    None
}

fn discarded(mut expression: &Expression<'_>) -> Option<super::super::Span> {
    for depth in 0..=8 {
        match unwrap_ts_wrappers(expression) {
            Expression::CallExpression(call) => expression = append::prefix_receiver(call)?,
            Expression::Identifier(id) if depth > 0 => return Some(span(id.span)),
            _ => return None,
        }
    }
    None
}
