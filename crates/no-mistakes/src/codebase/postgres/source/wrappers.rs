//! Wrapper projections borrow the one prepared AST and original token intervals.
use super::{locations::Locations, types::*};
use crate::codebase::postgres::parse::RecursiveViews;
use sqlparser::{
    ast::Statement,
    keywords::Keyword,
    parser::Parser,
    tokenizer::{Span, Token, TokenWithSpan},
};
mod atomic;
mod child;
mod header;
mod options;
mod restoration;
mod semantics;
#[cfg(test)]
mod tests;
use super::completeness::statement as complete;
pub(super) use semantics::finalize;

pub(super) struct Context<'a, 's> {
    pub source: &'s PostgresSqlSource,
    pub locations: &'a Locations<'s>,
    pub fetch: &'a [crate::codebase::postgres::parse::fetch_expression::Clause],
    pub generated: &'a [Span],
    pub markers: &'a [sqlparser::tokenizer::Location],
    pub recursive_views: &'a RecursiveViews,
}

pub(super) fn starts(parser: &Parser<'_>) -> bool {
    keyword(&parser.peek_token().token, Keyword::EXPLAIN)
        || keyword(&parser.peek_token().token, Keyword::PREPARE)
}

pub(super) fn collect(
    parser: &mut Parser<'_>,
    context: &Context<'_, '_>,
) -> PostgresSqlStatementKind {
    let kind = if keyword(&parser.peek_token().token, Keyword::EXPLAIN) {
        PostgresSqlWrapperKind::Explain
    } else {
        PostgresSqlWrapperKind::Prepare
    };
    let start = parser.index();
    let parsed = super::insert::parsing::parse(parser, context.markers);
    let mut wrapper = match parsed {
        Ok((mut statement, insert_facts)) => {
            let tokens = (start..parser.index())
                .map(|index| parser.token_at(index).clone())
                .collect::<Vec<_>>();
            match context.restore(
                &mut statement,
                parser.token_at(parser.index().saturating_sub(1)).span.end,
            ) {
                Ok(()) => project(&statement, &tokens, context, 0, insert_facts.as_ref())
                    .unwrap_or_else(|| unsupported(kind, "Unsupported wrapper syntax")),
                Err(error) => unsupported(kind, &error),
            }
        }
        Err(error) => unsupported(kind, &error.to_string()),
    };
    if !wrapper.complete {
        super::parsing::recover(parser, context.markers);
    }
    wrapper.span = span(
        &(start..parser.index())
            .map(|index| parser.token_at(index).clone())
            .collect::<Vec<_>>(),
        context,
    );
    PostgresSqlStatementKind::Wrapper { wrapper }
}

fn project(
    statement: &Statement,
    tokens: &[TokenWithSpan],
    context: &Context<'_, '_>,
    depth: usize,
    insert_facts: Option<&super::insert::parsing::ConflictFacts>,
) -> Option<PostgresSqlWrapper> {
    let (kind, execution, known, child) = match statement {
        Statement::Explain {
            analyze,
            options,
            query_plan,
            estimate,
            statement,
            ..
        } => {
            let execution = if *query_plan || *estimate {
                PostgresSqlExecution::Unknown
            } else {
                options::execution(*analyze, options.as_deref())
            };
            (
                PostgresSqlWrapperKind::Explain,
                execution,
                !query_plan && !estimate && execution != PostgresSqlExecution::Unknown,
                statement.as_ref(),
            )
        }
        Statement::Prepare { statement, .. } => (
            PostgresSqlWrapperKind::Prepare,
            PostgresSqlExecution::NonExecuting,
            true,
            statement.as_ref(),
        ),
        _ => return None,
    };
    let mut wrapper = empty(kind, execution);
    wrapper.span = span(tokens, context);
    if depth >= 16 {
        wrapper.diagnostics.push(diagnostic(
            "SQL wrapper nesting exceeds the safety limit",
            wrapper.span.clone(),
        ));
        return Some(wrapper);
    }
    let child_tokens = &tokens[header::child_start(tokens, kind)..];
    if let Some(child) = context.statement(child, child_tokens, 0, depth + 1, insert_facts) {
        wrapper.complete = known
            && matches!(statement, Statement::Explain { statement, .. } | Statement::Prepare { statement, .. } if matches!(statement.as_ref(), Statement::Insert(_) | Statement::Query(_)))
            && complete(&child.facts);
        wrapper.statements.push(child);
    }
    if !wrapper.complete {
        wrapper.diagnostics.push(diagnostic(
            "Wrapper contains unsupported options or incomplete child facts",
            wrapper.span.clone(),
        ));
    }
    Some(wrapper)
}

pub(super) fn function(
    value: &sqlparser::ast::CreateFunction,
    parser: &mut Parser<'_>,
    context: &Context<'_, '_>,
) -> PostgresSqlFunction {
    let mut function = super::ddl::function(value, context.locations);
    if keyword(&parser.peek_token().token, Keyword::BEGIN) {
        atomic::collect(&mut function, parser, context);
    }
    function
}

pub(super) fn declaration() -> PostgresSqlWrapper {
    let mut wrapper = empty(
        PostgresSqlWrapperKind::FunctionDeclaration,
        PostgresSqlExecution::NonExecuting,
    );
    wrapper.diagnostics.push(diagnostic("Function body is opaque; declaration facts do not imply body execution or complete child facts", None));
    wrapper
}

fn empty(kind: PostgresSqlWrapperKind, execution: PostgresSqlExecution) -> PostgresSqlWrapper {
    PostgresSqlWrapper {
        wrapper_kind: kind,
        execution,
        statements: Vec::new(),
        span: None,
        complete: false,
        diagnostics: Vec::new(),
    }
}
fn unsupported(kind: PostgresSqlWrapperKind, message: &str) -> PostgresSqlWrapper {
    let execution = if kind == PostgresSqlWrapperKind::Prepare {
        PostgresSqlExecution::NonExecuting
    } else {
        PostgresSqlExecution::Unknown
    };
    let mut wrapper = empty(kind, execution);
    wrapper.diagnostics.push(diagnostic(message, None));
    wrapper
}
fn diagnostic(message: &str, span: Option<PostgresSqlSpan>) -> PostgresSqlDiagnostic {
    PostgresSqlDiagnostic {
        message: message.into(),
        span,
    }
}
fn keyword(token: &Token, keyword: Keyword) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.keyword == keyword)
}
fn span(tokens: &[TokenWithSpan], context: &Context<'_, '_>) -> Option<PostgresSqlSpan> {
    let mut significant = tokens
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)));
    let first = significant.next()?;
    let last = significant.next_back().unwrap_or(first);
    context.locations.span(Span {
        start: first.span.start,
        end: crate::codebase::postgres::parse::fetch_expression::source_end(
            context.fetch,
            first.span.start,
            last.span.end,
        ),
    })
}
