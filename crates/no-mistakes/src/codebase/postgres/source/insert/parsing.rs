//! Extend the prepared parser at ON CONFLICT; every expression is parsed once.
use sqlparser::{
    ast::{ConflictTarget, DoUpdate, Expr, OnConflict, OnConflictAction, OnInsert, Statement},
    keywords::Keyword,
    parser::{IsOptional, Parser, ParserError},
    tokenizer::{Location, Token, TokenWithSpan},
};

pub(in crate::codebase::postgres::source) fn prepare(
    tokens: &mut [TokenWithSpan],
) -> Vec<Location> {
    let significant = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (!matches!(token.token, Token::Whitespace(_))).then_some(index)
        })
        .collect::<Vec<_>>();
    let mut markers = Vec::new();
    let mut beginning = true;
    let mut insert = false;
    let mut depth: usize = 0;
    for (position, index) in significant.iter().copied().enumerate() {
        if tokens[index].token == Token::SemiColon && depth == 0 {
            beginning = true;
            continue;
        }
        if beginning {
            insert = keyword(&tokens[index].token, Keyword::INSERT);
            beginning = false;
        }
        match tokens[index].token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            _ => {}
        }
        if insert
            && depth == 0
            && keyword(&tokens[index].token, Keyword::ON)
            && significant
                .get(position + 1)
                .is_some_and(|next| keyword(&tokens[*next].token, Keyword::CONFLICT))
        {
            // A delimiter lets the standard parser finish the INSERT prefix. The
            // same parser resumes here, retaining original expression locations.
            markers.push(tokens[index].span.start);
            tokens[index].token = Token::SemiColon;
        }
    }
    markers
}

pub(in crate::codebase::postgres::source) struct ConflictFacts {
    pub predicate: Option<Expr>,
    pub span: sqlparser::tokenizer::Span,
}

pub(in crate::codebase::postgres::source) fn parse(
    parser: &mut Parser<'_>,
    markers: &[Location],
) -> Result<(Statement, Option<ConflictFacts>), ParserError> {
    let insert = keyword(&parser.peek_token().token, Keyword::INSERT);
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
    let mut statement = parser.parse_statement()?;
    let mut facts = None;
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
            facts = Some(ConflictFacts {
                predicate,
                span: sqlparser::tokenizer::Span {
                    start,
                    end: parser.token_at(parser.index().saturating_sub(1)).span.end,
                },
            });
            if parser.parse_keyword(Keyword::RETURNING) {
                insert.returning = Some(parser.parse_comma_separated(Parser::parse_select_item)?);
            }
        }
    }
    Ok((statement, facts))
}

fn keyword(token: &Token, expected: Keyword) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.keyword == expected)
}
