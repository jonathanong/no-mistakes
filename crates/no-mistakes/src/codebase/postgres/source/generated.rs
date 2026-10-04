//! Restore original PostgreSQL 18 generated storage after one parser compatibility pass.
use sqlparser::{
    ast::{AlterTableOperation, ColumnDef, ColumnOption, GeneratedExpressionMode, Statement},
    keywords::Keyword,
    tokenizer::{Location, Span, Token, TokenWithSpan, Word},
};

pub(super) fn prepare(tokens: &mut Vec<TokenWithSpan>) -> Vec<Span> {
    let indices = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (!matches!(token.token, Token::Whitespace(_))).then_some(index)
        })
        .collect::<Vec<_>>();
    let mut changes = Vec::new();
    for window in indices.windows(4) {
        if !keyword(&tokens[window[0]].token, Keyword::GENERATED)
            || !keyword(&tokens[window[1]].token, Keyword::ALWAYS)
            || !keyword(&tokens[window[2]].token, Keyword::AS)
            || tokens[window[3]].token != Token::LParen
        {
            continue;
        }
        let open = window[3];
        let mut depth = 0usize;
        let close = (open..tokens.len()).find(|index| {
            match tokens[*index].token {
                Token::LParen => depth += 1,
                Token::RParen => depth -= 1,
                _ => {}
            }
            depth == 0
        });
        let Some(close) = close else {
            continue;
        };
        let next = indices
            .get(indices.partition_point(|index| *index <= close))
            .copied();
        if next.is_some_and(|index| keyword(&tokens[index].token, Keyword::STORED)) {
            continue;
        }
        let explicit = next.filter(|index| keyword(&tokens[*index].token, Keyword::VIRTUAL));
        changes.push((
            close,
            explicit,
            Span {
                start: tokens[open].span.start,
                end: tokens[close].span.end,
            },
        ));
    }
    let spans = changes.iter().map(|(_, _, span)| *span).collect();
    for (close, explicit, _) in changes.into_iter().rev() {
        let span = explicit.map_or(
            Span {
                start: tokens[close].span.end,
                end: tokens[close].span.end,
            },
            |index| tokens[index].span,
        );
        let token = TokenWithSpan {
            token: Token::Word(Word {
                value: "STORED".into(),
                quote_style: None,
                keyword: Keyword::STORED,
            }),
            span,
        };
        if let Some(index) = explicit {
            tokens[index] = token;
        } else {
            tokens.insert(close + 1, token);
        }
    }
    spans
}

fn keyword(token: &Token, expected: Keyword) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.keyword == expected)
}

pub(super) fn restore(statement: &mut Statement, spans: &[Span], end: Location) {
    match statement {
        Statement::CreateTable(table) => {
            for index in 0..table.columns.len() {
                let end = table
                    .columns
                    .get(index + 1)
                    .map_or(end, |column| column.name.span.start);
                restore_column(&mut table.columns[index], spans, end);
            }
        }
        Statement::AlterTable(table) => {
            for index in 0..table.operations.len() {
                if !matches!(
                    table.operations[index],
                    AlterTableOperation::AddColumn { .. }
                ) {
                    continue;
                }
                let end = table.operations[index + 1..]
                    .iter()
                    .find_map(|operation| {
                        if let AlterTableOperation::AddColumn { column_def, .. } = operation {
                            Some(column_def.name.span.start)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(end);
                if let AlterTableOperation::AddColumn { column_def, .. } =
                    &mut table.operations[index]
                {
                    restore_column(column_def, spans, end);
                }
            }
        }
        _ => {}
    }
}

fn restore_column(column: &mut ColumnDef, spans: &[Span], end: Location) {
    // A declaration owns its option range even when a constant Expr has an empty span.
    let index = spans.partition_point(|span| span.start < column.name.span.end);
    let virtual_storage = spans.get(index).is_some_and(|span| span.end <= end);
    if !virtual_storage {
        return;
    }
    for option in &mut column.options {
        if let ColumnOption::Generated {
            generation_expr: Some(_),
            generation_expr_mode,
            ..
        } = &mut option.option
        {
            *generation_expr_mode = Some(GeneratedExpressionMode::Virtual);
        }
    }
}
