//! Adapt PostgreSQL trigger literal arguments to the parser's FunctionDesc shape.
use sqlparser::{
    keywords::Keyword,
    tokenizer::{Token, TokenWithSpan, Word},
};

pub(in super::super) fn prepare(tokens: &mut [TokenWithSpan]) {
    for statement in tokens.split_mut(|token| token.token == Token::SemiColon) {
        let words: Vec<_> = statement
            .iter()
            .enumerate()
            .filter(|(_, token)| !matches!(token.token, Token::Whitespace(_)))
            .map(|(index, _)| index)
            .collect();
        let Some(first) = words.first() else {
            continue;
        };
        if !keyword(&statement[*first].token, Keyword::CREATE) {
            continue;
        }
        let kind = words.iter().skip(1).find(|index| !matches!(&statement[**index].token, Token::Word(word) if matches!(word.keyword, Keyword::OR | Keyword::REPLACE | Keyword::CONSTRAINT | Keyword::TEMP | Keyword::TEMPORARY)));
        if !kind.is_some_and(|index| keyword(&statement[*index].token, Keyword::TRIGGER)) {
            continue;
        }
        let Some(execute) = words
            .iter()
            .position(|index| keyword(&statement[*index].token, Keyword::EXECUTE))
        else {
            continue;
        };
        let Some(kind) = words.get(execute + 1) else {
            continue;
        };
        if !matches!(&statement[*kind].token, Token::Word(word) if matches!(word.keyword, Keyword::FUNCTION | Keyword::PROCEDURE))
        {
            continue;
        }
        let Some(open) = words
            .iter()
            .skip(execute + 2)
            .find(|index| statement[**index].token == Token::LParen)
            .copied()
        else {
            continue;
        };
        for token in &mut statement[open + 1..] {
            if token.token == Token::RParen {
                break;
            }
            let replacement = match &token.token {
                Token::SingleQuotedString(value) => Some((value.clone(), Some('\''))),
                Token::Number(value, false) => Some((value.clone(), None)),
                // Trigger words are strings, not SQL datatype names. Preserve their
                // PostgreSQL identifier folding instead of datatype aliases such as INT.
                Token::Word(word) => Some((
                    if word.quote_style.is_some() {
                        word.value.clone()
                    } else {
                        word.value.to_ascii_lowercase()
                    },
                    Some('\''),
                )),
                _ => None,
            };
            if let Some((value, quote_style)) = replacement {
                token.token = Token::Word(Word {
                    value,
                    quote_style,
                    keyword: Keyword::NoKeyword,
                });
            }
        }
    }
}

fn keyword(token: &Token, expected: Keyword) -> bool {
    matches!(token, Token::Word(word) if word.keyword == expected)
}
