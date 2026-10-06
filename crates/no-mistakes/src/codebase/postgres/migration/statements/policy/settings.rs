use super::word;
use crate::codebase::postgres::types::SqlSettingUse;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn collect(
    header: &[&TokenWithSpan],
    code: &[&TokenWithSpan],
    out: &mut Vec<SqlSettingUse>,
) {
    let set = if header.first().is_some_and(|token| word(token, "SET")) {
        Some(0)
    } else if header.first().is_some_and(|token| word(token, "ALTER"))
        && header
            .get(1)
            .is_some_and(|token| word(token, "DATABASE") || word(token, "SYSTEM"))
    {
        header.iter().position(|token| word(token, "SET"))
    } else {
        None
    };
    if let Some(set) = set {
        let mut name = set + 1;
        if header
            .get(name)
            .is_some_and(|token| word(token, "LOCAL") || word(token, "SESSION"))
        {
            name += 1;
        }
        if let Some(token) = header.get(name) {
            if let Token::Word(value) = &token.token {
                let mut parameter = value.value.clone();
                let mut at = name + 1;
                while header
                    .get(at)
                    .is_some_and(|token| token.token == Token::Period)
                {
                    let Some(Token::Word(part)) = header.get(at + 1).map(|token| &token.token)
                    else {
                        break;
                    };
                    parameter.push('.');
                    parameter.push_str(&part.value);
                    at += 2;
                }
                push(&parameter, header[set], out);
            }
        }
    }
    for (at, token) in code.iter().enumerate() {
        if !builtin(token, "set_config")
            || code
                .get(at + 1)
                .is_none_or(|token| token.token != Token::LParen)
        {
            continue;
        }
        if at >= 1
            && code[at - 1].token == Token::Period
            && (at < 2 || !builtin(code[at - 2], "pg_catalog"))
        {
            continue;
        }
        if code
            .get(at + 3)
            .is_none_or(|token| token.token != Token::Comma)
        {
            continue;
        }
        let argument = code[at + 2];
        let name = match &argument.token {
            Token::SingleQuotedString(value) | Token::EscapedStringLiteral(value) => {
                Some(value.as_str())
            }
            Token::DollarQuotedString(value) => Some(value.value.as_str()),
            _ => None,
        };
        if let Some(name) = name {
            push(name, token, out);
        }
    }
}

fn builtin(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word) if if word.quote_style.is_some() { word.value == expected } else { word.value.eq_ignore_ascii_case(expected) })
}

fn push(name: &str, token: &TokenWithSpan, out: &mut Vec<SqlSettingUse>) {
    out.push(SqlSettingUse {
        name: name.to_ascii_lowercase(),
        line: token.span.start.line.max(1) as usize,
    });
}
