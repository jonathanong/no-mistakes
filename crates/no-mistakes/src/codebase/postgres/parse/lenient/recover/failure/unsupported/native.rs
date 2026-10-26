//! Validate missing native DDL forms through a narrowly normalized parser clone.
use super::super::keywords::reserved;
use sqlparser::tokenizer::{Token, TokenWithSpan};
use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser};

pub(super) fn valid(original: &[TokenWithSpan]) -> bool {
    let tokens: Vec<_> = original
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .cloned()
        .collect();
    if tokens
        .first()
        .is_none_or(|token| !word(token, "CREATE") && !word(token, "DROP"))
    {
        return false;
    }
    if tokens
        .iter()
        .any(|token| matches!(&token.token, Token::Word(word) if word.value.is_empty()))
    {
        return false;
    }
    let mut at = 1;
    if tokens.get(at).is_some_and(|token| word(token, "OR")) {
        if !tokens
            .get(at + 1)
            .is_some_and(|token| word(token, "REPLACE"))
        {
            return false;
        }
        at += 2;
    }
    if tokens.get(at).is_some_and(|token| word(token, "PROCEDURE")) {
        return procedure(tokens, at);
    }
    operator(tokens, at)
}

fn procedure(mut tokens: Vec<TokenWithSpan>, at: usize) -> bool {
    tokens[at].token = Token::make_keyword("FUNCTION");
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens);
    let Ok(Statement::CreateFunction(function)) = parser.parse_statement() else {
        return false;
    };
    parser.peek_token().token == Token::EOF
        && function.return_type.is_none()
        && function.behavior.is_none()
        && function.called_on_null.is_none()
        && function.parallel.is_none()
        && function.language.is_some()
        && function.function_body.is_some()
}

fn operator(mut tokens: Vec<TokenWithSpan>, mut at: usize) -> bool {
    if !tokens.get(at).is_some_and(|token| word(token, "OPERATOR")) {
        return false;
    }
    at += 1;
    if tokens.get(at).is_some_and(|token| word(token, "IF")) {
        at += 1;
        if !tokens.get(at).is_some_and(|token| word(token, "EXISTS")) {
            return false;
        }
        at += 1;
    }
    if tokens
        .get(at + 1)
        .is_some_and(|token| token.token == Token::Period)
    {
        if !matches!(&tokens[at].token, Token::Word(word) if word.quote_style == Some('"')
            || (word.quote_style.is_none() && !reserved(&word.value)))
        {
            return false;
        }
        at += 2;
    }
    let Some(end) = tokens
        .iter()
        .enumerate()
        .skip(at)
        .find_map(|(index, token)| (token.token == Token::LParen).then_some(index))
    else {
        return false;
    };
    let operator = tokens[at..end]
        .iter()
        .map(|token| token.token.to_string())
        .collect::<String>();
    if operator.is_empty()
        || !operator.chars().all(|c| "+-*/<>=~!@#%^&|`?".contains(c))
        || operator.len() > 1
            && operator.ends_with(['+', '-'])
            && !operator.chars().any(|c| "~!@#%^&|`?".contains(c))
        || tokens[at..end]
            .windows(2)
            .any(|pair| pair[0].span.end != pair[1].span.start)
    {
        return false;
    }
    tokens[at].token = Token::CustomBinaryOperator(operator);
    tokens.drain(at + 1..end);
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens);
    matches!(
        parser.parse_statement(),
        Ok(Statement::CreateOperator(_) | Statement::DropOperator(_))
    ) && parser.peek_token().token == Token::EOF
}

fn word(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word) if word.quote_style.is_none()
        && word.value.eq_ignore_ascii_case(expected))
}
