use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Word};

/// sqlparser 0.63 accepts TABLE as a query arm but not as a top-level SQL
/// statement. Rewrite only statement-leading TABLE to its SELECT equivalent.
pub(super) fn normalize(tokens: &mut Vec<TokenWithSpan>) {
    if !has_standalone_table(tokens) {
        return;
    }
    let mut result = Vec::with_capacity(tokens.len());
    let mut at_statement_start = true;
    for token in tokens.drain(..) {
        match &token.token {
            Token::Whitespace(_) => result.push(token),
            Token::SemiColon => {
                result.push(token);
                at_statement_start = true;
            }
            Token::Word(word)
                if at_statement_start
                    && word.quote_style.is_none()
                    && word.keyword == Keyword::TABLE =>
            {
                let span = token.span;
                result.push(TokenWithSpan::new(
                    word_token("SELECT", Keyword::SELECT),
                    span,
                ));
                result.push(TokenWithSpan::new(Token::Mul, span));
                result.push(TokenWithSpan::new(word_token("FROM", Keyword::FROM), span));
                at_statement_start = false;
            }
            _ => {
                result.push(token);
                at_statement_start = false;
            }
        }
    }
    *tokens = result;
}

fn has_standalone_table(tokens: &[TokenWithSpan]) -> bool {
    let mut at_statement_start = true;
    for token in tokens {
        match &token.token {
            Token::Whitespace(_) => {}
            Token::SemiColon => at_statement_start = true,
            Token::Word(word)
                if at_statement_start
                    && word.quote_style.is_none()
                    && word.keyword == Keyword::TABLE =>
            {
                return true;
            }
            _ => at_statement_start = false,
        }
    }
    false
}

fn word_token(value: &str, keyword: Keyword) -> Token {
    Token::Word(Word {
        value: value.to_owned(),
        quote_style: None,
        keyword,
    })
}
