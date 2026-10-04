//! Typed branch occurrences reuse the enclosing program's prepared tokens and AST.
use super::{expressions::expression, locations::Locations, types::*};
use sqlparser::{
    ast::{ConditionalStatements, IfStatement, Spanned, Statement},
    keywords::Keyword,
    tokenizer::{Span, Token, TokenWithSpan, Whitespace},
};

pub(super) fn project(
    value: &mut IfStatement,
    tokens: &[&TokenWithSpan],
    source: &PostgresSqlSource,
    locations: &Locations<'_>,
    generated: &[Span],
) -> Result<PostgresSqlStatementKind, String> {
    let mut branches = Vec::new();
    for branch in std::iter::once(&mut value.if_block)
        .chain(value.elseif_blocks.iter_mut())
        .chain(value.else_block.iter_mut())
    {
        let mut cursor = branch
            .then_token
            .as_ref()
            .unwrap_or(&branch.start_token)
            .0
            .span
            .end;
        let mut block_end = cursor;
        let statements = match &mut branch.conditional_statements {
            ConditionalStatements::Sequence { statements } => statements,
            ConditionalStatements::BeginEnd(block) => {
                cursor = block.begin_token.0.span.end;
                block_end = block.end_token.0.span.end;
                &mut block.statements
            }
        };
        let mut projected = Vec::new();
        for (ordinal, statement) in statements.iter_mut().enumerate() {
            let start_index = tokens.partition_point(|token| token.span.start < cursor);
            let start_index = start_index
                + tokens[start_index..]
                    .iter()
                    .position(|token| !matches!(token.token, Token::Whitespace(_)))
                    .ok_or("Conditional statement source start is unavailable")?;
            let end = statement.span().end;
            let end_index = tokens.partition_point(|token| token.span.end < end);
            let finish = end_index
                + tokens[end_index..]
                    .iter()
                    .position(|token| token.token == Token::SemiColon)
                    .ok_or("Conditional statement delimiter is unavailable")?;
            let owned = &tokens[start_index..=finish];
            let span = locations
                .span(Span {
                    start: owned[0].span.start,
                    end: owned.last().unwrap().span.end,
                })
                .ok_or("Conditional statement source span is unavailable")?;
            super::generated::restore(statement, generated, owned.last().unwrap().span.end);
            let facts = if let Statement::If(nested) = statement {
                project(nested, owned, source, locations, generated)?
            } else {
                let tables = crate::codebase::postgres::statements::TableTokenIndex::from_iter(
                    owned.iter().copied(),
                );
                super::parsing::project(statement, locations, &tables)
            };
            cursor = owned.last().unwrap().span.end;
            projected.push(PostgresSqlStatement {
                ordinal,
                sql: source.sql[span.start.offset..span.end.offset].into(),
                span,
                facts,
            });
        }
        let span = locations
            .span(Span {
                start: branch.start_token.0.span.start,
                end: cursor.max(block_end),
            })
            .ok_or("Conditional branch source span is unavailable")?;
        branches.push(PostgresSqlConditionalBranch {
            condition: branch
                .condition
                .as_ref()
                .map(|value| expression(value, locations)),
            span,
            statements: projected,
        });
    }
    Ok(PostgresSqlStatementKind::Conditional { branches })
}

/// Match procedural spellings to the same parser's typed conditional grammar.
pub(super) fn prepare(tokens: &mut [TokenWithSpan]) {
    let significant = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (!matches!(token.token, Token::Whitespace(_))).then_some(index)
        })
        .collect::<Vec<_>>();
    for indices in significant.windows(3) {
        if matches!(&tokens[indices[0]].token, Token::Word(word) if word.keyword == Keyword::END)
            && tokens[indices[1]].token == Token::SemiColon
            && matches!(&tokens[indices[2]].token, Token::Word(word) if ([Keyword::ELSE, Keyword::END].contains(&word.keyword) || word.value.eq_ignore_ascii_case("ELSIF")))
        {
            // The AST owns this nested BEGIN/END; retain the original delimiter span.
            tokens[indices[1]].token = Token::Whitespace(Whitespace::Space);
        }
    }
    for token in tokens {
        if let Token::Word(word) = &mut token.token {
            if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("ELSIF") {
                word.value = "ELSEIF".into();
                word.keyword = Keyword::ELSEIF;
            }
        }
    }
}
