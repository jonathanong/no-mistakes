use super::{keyword, PostgresSqlWrapperKind};
use sqlparser::{
    keywords::Keyword,
    tokenizer::{Token, TokenWithSpan},
};

/// The AST already validated this prefix; locate its child in original tokens.
pub(super) fn child_start(tokens: &[TokenWithSpan], kind: PostgresSqlWrapperKind) -> usize {
    let significant = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (!matches!(token.token, Token::Whitespace(_))).then_some(index)
        })
        .collect::<Vec<_>>();
    let mut position = 1;
    let mut depth: usize = 0;
    if kind == PostgresSqlWrapperKind::Prepare {
        for index in significant.iter().copied().skip(1) {
            match tokens[index].token {
                Token::LParen => depth += 1,
                Token::RParen => depth = depth.saturating_sub(1),
                _ => {}
            }
            if depth == 0 && keyword(&tokens[index].token, Keyword::AS) {
                return index + 1;
            }
        }
        return tokens.len();
    }
    if significant
        .get(position)
        .is_some_and(|index| tokens[*index].token == Token::LParen)
    {
        while let Some(index) = significant.get(position) {
            match tokens[*index].token {
                Token::LParen => depth += 1,
                Token::RParen => depth = depth.saturating_sub(1),
                _ => {}
            }
            position += 1;
            if depth == 0 {
                break;
            }
        }
    } else {
        while let Some(index) = significant.get(position) {
            let token = &tokens[*index].token;
            if keyword(token, Keyword::FORMAT) {
                position += 1;
                if significant
                    .get(position)
                    .is_some_and(|index| tokens[*index].token == Token::Eq)
                {
                    position += 1;
                }
                position += 1;
            } else if [
                Keyword::ANALYZE,
                Keyword::VERBOSE,
                Keyword::QUERY,
                Keyword::PLAN,
                Keyword::ESTIMATE,
            ]
            .iter()
            .any(|value| keyword(token, *value))
            {
                position += 1;
            } else {
                break;
            }
        }
    }
    significant.get(position).copied().unwrap_or(tokens.len())
}
