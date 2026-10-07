//! Extend the prepared parser at ON CONFLICT; every expression is parsed once.
use sqlparser::{
    ast::{ConflictTarget, DoUpdate, Expr, OnConflict, OnConflictAction, OnInsert, Statement},
    keywords::Keyword,
    parser::{IsOptional, Parser, ParserError},
    tokenizer::{Location, Token},
};

mod markers;
mod with;
pub(in crate::codebase::postgres::source) use markers::prepare;

pub(in crate::codebase::postgres::source) struct ConflictFacts {
    pub predicate: Option<Expr>,
    pub span: Option<sqlparser::tokenizer::Span>,
    pub source_span: Option<sqlparser::tokenizer::Span>,
    pub unsupported_with: bool,
}

pub(in crate::codebase::postgres::source) fn parse(
    parser: &mut Parser<'_>,
    markers: &[Location],
) -> Result<(Statement, Option<ConflictFacts>), ParserError> {
    let insert = keyword(&parser.peek_token().token, Keyword::INSERT)
        || keyword(&parser.peek_token().token, Keyword::WITH);
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
    if let Statement::Insert(insert) = &mut statement {
        if markers
            .binary_search(&parser.peek_token().span.start)
            .is_ok()
        {
            let start = parser.peek_token().span.start;
            let mut predicate = None;
            // Preparation proves these two tokens; consume them without adding
            // unreachable failure paths for the synthetic delimiter.
            parser.next_token();
            parser.next_token();
            let conflict_target = if parser.parse_keywords(&[Keyword::ON, Keyword::CONSTRAINT]) {
                Some(ConflictTarget::OnConstraint(
                    parser.parse_object_name(false)?,
                ))
            } else if parser.peek_token().token == Token::LParen {
                Some(ConflictTarget::Columns(
                    parser.parse_parenthesized_column_list(IsOptional::Mandatory, false)?,
                ))
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
                let assignments = parser.parse_comma_separated(Parser::parse_assignment)?;
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
                predicate: None,
                span: None,
                source_span: None,
                unsupported_with: false,
            });
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
