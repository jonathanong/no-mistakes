//! Typed branch occurrences reuse the enclosing program's prepared tokens and AST.
use super::{expressions::expression, locations::Locations, types::*};
use sqlparser::{
    ast::{ConditionalStatements, IfStatement, Statement},
    keywords::Keyword,
    tokenizer::{Span, Token, TokenWithSpan, Whitespace},
};

/// Project branches only when prepared tokens identify nonempty statement ranges.
pub(super) fn project(
    value: &mut IfStatement,
    tokens: &[&TokenWithSpan],
    source: &PostgresSqlSource,
    locations: &Locations<'_>,
    generated: &[Span],
    recursive_views: &crate::codebase::postgres::parse::RecursiveViews,
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
            let range = super::conditional_source::statement_range(statement, tokens, cursor)?;
            let owned = &tokens[range];
            let span = locations
                .span(Span {
                    start: owned[0].span.start,
                    end: owned.last().unwrap().span.end,
                })
                .ok_or("Conditional statement source span is unavailable")?;
            super::generated::restore(statement, generated, owned.last().unwrap().span.end);
            let mut facts = if let Statement::If(nested) = statement {
                project(nested, owned, source, locations, generated, recursive_views)?
            } else {
                let tables = crate::codebase::postgres::statements::TableTokenIndex::from_iter(
                    owned.iter().copied(),
                );
                super::projection::project(statement, locations, &tables, recursive_views)
            };
            if let PostgresSqlStatementKind::Insert { insert } = &mut facts {
                insert.span = Some(span.clone());
            }
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
    let mut case_depth = 0usize;
    for (at, index) in significant.iter().enumerate() {
        let previous = at.checked_sub(1).map(|at| &tokens[significant[at]].token);
        let boundary = matches!(previous, Some(Token::SemiColon))
            || matches!(previous, Some(Token::Word(word)) if word.keyword == Keyword::THEN);
        let Token::Word(word) = &mut tokens[*index].token else {
            continue;
        };
        if word.quote_style.is_none() {
            if word.keyword == Keyword::CASE {
                case_depth += 1;
            } else if word.keyword == Keyword::END {
                case_depth = case_depth.saturating_sub(1);
            } else if case_depth == 0 && boundary && word.value.eq_ignore_ascii_case("ELSIF") {
                word.value = "ELSEIF".into();
                word.keyword = Keyword::ELSEIF;
            }
        }
    }
    for indices in significant.windows(3) {
        if matches!(&tokens[indices[0]].token, Token::Word(word) if word.keyword == Keyword::END)
            && tokens[indices[1]].token == Token::SemiColon
            && matches!(&tokens[indices[2]].token, Token::Word(word) if word.quote_style.is_none() && ([Keyword::ELSE, Keyword::ELSEIF, Keyword::END].contains(&word.keyword) || word.value.eq_ignore_ascii_case("ELSIF")))
        {
            // The AST owns this nested BEGIN/END; retain the original delimiter span.
            tokens[indices[1]].token = Token::Whitespace(Whitespace::Space);
        }
    }
}
