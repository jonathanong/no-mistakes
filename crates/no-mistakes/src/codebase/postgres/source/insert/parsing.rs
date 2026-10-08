//! Extend the prepared parser at ON CONFLICT; every expression is parsed once.
use sqlparser::{
    ast::{ConflictTarget, DoUpdate, Expr, OnConflict, OnConflictAction, OnInsert, Statement},
    keywords::Keyword,
    parser::{Parser, ParserError},
    tokenizer::{Location, Token},
};

pub(super) mod assignment;
mod markers;
mod with;
pub(in crate::codebase::postgres::source) use markers::prepare;

pub(in crate::codebase::postgres::source) struct ConflictFacts {
    pub expressions: Vec<(Expr, sqlparser::tokenizer::Span)>,
    pub targets: Vec<Option<assignment::Target>>,
    pub predicate: Option<Expr>,
    pub span: Option<sqlparser::tokenizer::Span>,
    pub source_span: Option<sqlparser::tokenizer::Span>,
    pub unsupported_with: bool,
}

/// Normalize a child already owned by the enclosing conditional AST.
/// The temporary sentinel is replaced before projection and is never emitted.
pub(in crate::codebase::postgres::source) fn normalize(
    statement: &mut Statement,
) -> Option<ConflictFacts> {
    let original = std::mem::replace(
        statement,
        Statement::Commit {
            chain: false,
            end: false,
            modifier: None,
        },
    );
    let (normalized, facts) = with::normalize(original);
    *statement = normalized;
    facts
}

pub(in crate::codebase::postgres::source) fn parse(
    parser: &mut Parser<'_>,
    markers: &[Location],
) -> Result<(Statement, Option<ConflictFacts>), ParserError> {
    let insert = keyword(&parser.peek_token().token, Keyword::INSERT)
        || keyword(&parser.peek_token().token, Keyword::WITH)
        || keyword(&parser.peek_token().token, Keyword::EXPLAIN)
        || keyword(&parser.peek_token().token, Keyword::PREPARE);
    let result = parse_inner(parser, markers);
    if insert && result.is_err() {
        let previous = parser.token_at(parser.index().saturating_sub(1));
        if previous.token == Token::SemiColon
            && markers.binary_search(&previous.span.start).is_err()
        {
            // Expression/identifier failures may consume the real delimiter.
            // Return it to recovery so the following statement stays intact.
            parser.prev_token();
        }
    }
    result
}

fn parse_inner(
    parser: &mut Parser<'_>,
    markers: &[Location],
) -> Result<(Statement, Option<ConflictFacts>), ParserError> {
    let (mut statement, mut facts) = with::normalize(parser.parse_statement()?);
    if let Some(insert) = inner_insert(&mut statement) {
        if markers
            .binary_search(&parser.peek_token().span.start)
            .is_ok()
        {
            let start = parser.peek_token().span.start;
            let mut predicate = None;
            let mut expressions = Vec::new();
            let mut targets = Vec::new();
            // Preparation proves these two tokens; consume them without adding
            // unreachable failure paths for the synthetic delimiter.
            parser.next_token();
            parser.next_token();
            let conflict_target = if parser.parse_keywords(&[Keyword::ON, Keyword::CONSTRAINT]) {
                Some(ConflictTarget::OnConstraint(
                    parser.parse_object_name(false)?,
                ))
            } else if parser.peek_token().token == Token::LParen {
                parser.next_token();
                expressions = parser.parse_comma_separated(|parser| {
                    let start = parser.peek_token().span.start;
                    let expr = parser.parse_expr()?;
                    let end = parser.token_at(parser.index().saturating_sub(1)).span.end;
                    Ok((expr, sqlparser::tokenizer::Span { start, end }))
                })?;
                parser.expect_token(&Token::RParen)?;
                let columns = expressions
                    .iter()
                    .filter_map(|(expr, _)| match expr {
                        Expr::Identifier(ident) => Some(ident.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if columns.len() == expressions.len() {
                    expressions.clear();
                }
                Some(ConflictTarget::Columns(columns))
            } else {
                None
            };
            if parser.parse_keyword(Keyword::WHERE) {
                if !matches!(conflict_target, Some(ConflictTarget::Columns(_))) {
                    return Err(ParserError::ParserError(
                        "Conflict predicates require a column target".into(),
                    ));
                }
                predicate = Some(parser.parse_expr()?);
            }
            parser.expect_keyword_is(Keyword::DO)?;
            let action = if parser.parse_keyword(Keyword::NOTHING) {
                OnConflictAction::DoNothing
            } else {
                parser.expect_keyword_is(Keyword::UPDATE)?;
                parser.expect_keyword_is(Keyword::SET)?;
                let assignments = parser.parse_comma_separated(|parser| {
                    let (assignment, target) = assignment::parse(parser)?;
                    targets.push(target);
                    Ok(assignment)
                })?;
                let selection = if parser.parse_keyword(Keyword::WHERE) {
                    Some(parser.parse_expr()?)
                } else {
                    None
                };
                OnConflictAction::DoUpdate(DoUpdate {
                    assignments,
                    selection,
                })
            };
            insert.on = Some(OnInsert::OnConflict(OnConflict {
                conflict_target,
                action,
            }));
            let metadata = facts.get_or_insert(ConflictFacts {
                expressions: Vec::new(),
                targets: Vec::new(),
                predicate: None,
                span: None,
                source_span: None,
                unsupported_with: false,
            });
            metadata.expressions = expressions;
            metadata.targets = targets;
            metadata.predicate = predicate;
            metadata.span = Some(sqlparser::tokenizer::Span {
                start,
                end: parser.token_at(parser.index().saturating_sub(1)).span.end,
            });
            if parser.parse_keyword(Keyword::RETURNING) {
                insert.returning = Some(parser.parse_comma_separated(Parser::parse_select_item)?);
            }
        }
    }
    if markers
        .binary_search(&parser.peek_token().span.start)
        .is_ok()
    {
        return Err(ParserError::ParserError(
            "Duplicate ON CONFLICT clause".into(),
        ));
    }
    Ok((statement, facts))
}

fn keyword(token: &Token, expected: Keyword) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.keyword == expected)
}

fn inner_insert(statement: &mut Statement) -> Option<&mut sqlparser::ast::Insert> {
    match statement {
        Statement::Insert(insert) => Some(insert),
        Statement::Explain { statement, .. } | Statement::Prepare { statement, .. } => {
            inner_insert(statement)
        }
        _ => None,
    }
}
