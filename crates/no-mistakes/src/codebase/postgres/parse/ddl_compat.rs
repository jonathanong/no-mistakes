//! Narrow PostgreSQL utility syntax absent from sqlparser's AST/grammar.
//! Unrepresentable metadata uses DISCARD PLANS: neutral in the analysis model,
//! preserving temporary relations, transactions, and prepared statement facts.
//! It never invents table/index names. Original source owns diagnostics.
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::{Parser, ParserError};
use sqlparser::tokenizer::{Token, TokenWithSpan};

mod metadata;
mod trigger;

pub(super) fn normalize(tokens: &mut Vec<TokenWithSpan>) {
    let mut output = Vec::with_capacity(tokens.len());
    for chunk in tokens.split_inclusive(|token| token.token == Token::SemiColon) {
        let mut head = chunk
            .iter()
            .filter(|token| !matches!(token.token, Token::Whitespace(_)));
        let candidate = match (head.next(), head.next()) {
            (Some(first), Some(second)) if word(first, "COMMENT") => word(second, "ON"),
            (Some(first), Some(second)) if word(first, "ALTER") => {
                word(second, "INDEX") || word(second, "TEXT")
            }
            (Some(first), Some(second)) if word(first, "CREATE") => {
                word(second, "INDEX")
                    || word(second, "UNIQUE")
                    || word(second, "TRIGGER")
                    || word(second, "OR")
                    || word(second, "CONSTRAINT")
            }
            _ => false,
        };
        if !candidate {
            output.extend_from_slice(chunk);
            continue;
        }
        let code: Vec<_> = chunk
            .iter()
            .filter(|token| {
                !matches!(
                    token.token,
                    Token::Whitespace(_) | Token::SemiColon | Token::EOF
                )
            })
            .cloned()
            .collect();
        // PostgreSQL rejects zero-length quoted identifiers. Do not let a native
        // metadata parser accept them and turn malformed DDL into a safe marker.
        if code
            .iter()
            .any(|token| matches!(&token.token, Token::Word(word) if word.value.is_empty()))
        {
            output.extend_from_slice(chunk);
            continue;
        }
        let replacement = trigger::rewrite(&code)
            .or_else(|| rewrite(&code))
            .or_else(|| {
                metadata::validate(&code).then(|| {
                    ["DISCARD", "PLANS"]
                        .map(|word| TokenWithSpan {
                            token: Token::make_keyword(word),
                            span: code[0].span,
                        })
                        .to_vec()
                })
            });
        if let Some(replacement) = replacement {
            output.extend(replacement);
            if let Some(last) = chunk.last().filter(|token| token.token == Token::SemiColon) {
                output.push(last.clone());
            }
        } else {
            output.extend_from_slice(chunk);
        }
    }
    *tokens = output;
}

fn rewrite(code: &[TokenWithSpan]) -> Option<Vec<TokenWithSpan>> {
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(code.to_vec());
    if parser.parse_keywords(&[Keyword::COMMENT, Keyword::ON, Keyword::FUNCTION]) {
        parser.parse_object_name(false).ok()?;
        let start = code.iter().position(|token| token.token == Token::LParen)?;
        let mut depth = 0;
        let end = code
            .iter()
            .enumerate()
            .skip(start)
            .find_map(|(at, token)| {
                match token.token {
                    Token::LParen => depth += 1,
                    Token::RParen => depth -= 1,
                    _ => {}
                }
                (depth == 0).then_some(at + 1)
            })?;
        let signature = [Token::make_keyword("DROP"), Token::make_keyword("FUNCTION")]
            .into_iter()
            .chain(code[3..end].iter().map(|token| token.token.clone()))
            .collect();
        let mut check = Parser::new(&PostgreSqlDialect {}).with_tokens(signature);
        check.parse_statement().ok()?;
        if check.peek_token().token != Token::EOF {
            return None;
        }
        let rewritten: Vec<_> = code[..start].iter().chain(&code[end..]).cloned().collect();
        let mut check =
            Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(rewritten.clone());
        check.parse_statement().ok()?;
        return (check.peek_token().token == Token::EOF).then_some(rewritten);
    }
    if parser.parse_keyword(Keyword::CREATE) {
        let _ = parser.parse_keyword(Keyword::UNIQUE);
        if !parser.parse_keyword(Keyword::INDEX) {
            return None;
        }
        // ONLY belongs directly after the index's ON, before its table name.
        let at = code
            .windows(2)
            .position(|pair| word(&pair[0], "ON") && word(&pair[1], "ONLY"))?
            + 1;
        let rewritten: Vec<_> = code
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != at)
            .map(|(_, token)| token.clone())
            .collect();
        // A malformed modifier must not be reinterpreted as a table named ONLY.
        return Some(rewritten);
    }
    None
}

fn word(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}

fn expect_end(parser: &mut Parser<'_>) -> Result<(), ParserError> {
    parser.expect_token(&Token::EOF).map(|_| ())
}

#[cfg(test)]
mod tests;
