use super::super::keyword_of;
use sqlparser::ast::{AlterTableOperation, Statement};
use sqlparser::dialect::GenericDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};

// sqlparser's PostgreSQL dialect does not yet parse PostgreSQL's ALTER TABLE
// ATTACH/DETACH PARTITION grammar. Recover only that complete statement with its
// compatible generic-dialect AST; leave all other failed SQL on the normal path.
pub(super) fn recover_partition_change(
    chunk: &[Token],
    original: Option<&[TokenWithSpan]>,
    allow_concurrent_detach: bool,
) -> Option<Statement> {
    let dialect = GenericDialect {};
    let mut parser = match original {
        Some(tokens) => Parser::new(&dialect).with_tokens_with_locations(tokens.to_vec()),
        None => Parser::new(&dialect).with_tokens(chunk.to_vec()),
    };
    let statement = parser.parse_statement().ok()?;
    let Statement::AlterTable(table) = &statement else {
        return None;
    };
    let [operation] = table.operations.as_slice() else {
        return None;
    };
    match operation {
        AlterTableOperation::AttachPartition { .. } => {
            if !partition_bound_suffix(&mut parser) {
                return None;
            }
        }
        AlterTableOperation::DetachPartition { .. } => {
            let next = parser.peek_token().token;
            if !matches!(next, Token::EOF) {
                if !matches!(next, Token::Word(ref word) if word.quote_style.is_none()
                    && (word.value.eq_ignore_ascii_case("CONCURRENTLY")
                        || word.value.eq_ignore_ascii_case("FINALIZE")))
                {
                    return None;
                }
                parser.next_token();
                if !allow_concurrent_detach
                    && matches!(next, Token::Word(ref word) if word.value.eq_ignore_ascii_case("CONCURRENTLY"))
                {
                    return None;
                }
                if !matches!(parser.peek_token().token, Token::EOF) {
                    return None;
                }
            }
        }
        _ => return None,
    }
    Some(statement)
}

// The generic dialect stops after the partition name. Validate the remaining
// PostgreSQL bound syntax without broadening recovery to unrelated ALTER TABLE SQL.
fn partition_bound_suffix(parser: &mut Parser) -> bool {
    if keyword_of(&parser.peek_token().token) == Some(Keyword::DEFAULT) {
        parser.next_token();
        return matches!(parser.peek_token().token, Token::EOF);
    }
    if keyword_of(&parser.next_token().token) != Some(Keyword::FOR)
        || keyword_of(&parser.next_token().token) != Some(Keyword::VALUES)
    {
        return false;
    }
    let valid = match keyword_of(&parser.next_token().token) {
        Some(Keyword::FROM) => {
            let lower = bound_expressions(parser, true);
            lower.is_some()
                && keyword_of(&parser.next_token().token) == Some(Keyword::TO)
                && bound_expressions(parser, true) == lower
        }
        Some(Keyword::IN) => bound_expressions(parser, false).is_some(),
        Some(Keyword::WITH) => hash_bound(parser),
        _ => false,
    };
    valid && matches!(parser.peek_token().token, Token::EOF)
}

// Parse complete comma-separated bound expressions instead of treating balanced
// parentheses as proof that PostgreSQL could execute the ownership transition.
fn bound_expressions(parser: &mut Parser, allow_sentinels: bool) -> Option<usize> {
    if parser.next_token().token != Token::LParen {
        return None;
    }
    let mut count = 0;
    loop {
        match keyword_of(&parser.peek_token().token) {
            Some(Keyword::MINVALUE | Keyword::MAXVALUE) if allow_sentinels => {
                parser.next_token();
            }
            Some(Keyword::MINVALUE | Keyword::MAXVALUE) => return None,
            _ => {
                parser.parse_expr().ok()?;
            }
        }
        count += 1;
        match parser.next_token().token {
            Token::Comma => continue,
            Token::RParen => return Some(count),
            _ => return None,
        }
    }
}

fn hash_bound(parser: &mut Parser) -> bool {
    if parser.next_token().token != Token::LParen {
        return false;
    }
    let mut modulus = None;
    let mut remainder = None;
    hash_option(parser, &mut modulus, &mut remainder)
        && parser.next_token().token == Token::Comma
        && hash_option(parser, &mut modulus, &mut remainder)
        && parser.next_token().token == Token::RParen
        && matches!((modulus, remainder), (Some(modulus), Some(remainder)) if remainder < modulus)
}

fn hash_option(
    parser: &mut Parser,
    modulus: &mut Option<i32>,
    remainder: &mut Option<i32>,
) -> bool {
    match keyword_of(&parser.next_token().token) {
        Some(Keyword::MODULUS) if modulus.is_none() => {
            *modulus = positive_integer(parser.next_token().token);
            modulus.is_some()
        }
        Some(Keyword::REMAINDER) if remainder.is_none() => {
            *remainder = nonnegative_integer(parser.next_token().token);
            remainder.is_some()
        }
        _ => false,
    }
}

fn positive_integer(token: Token) -> Option<i32> {
    nonnegative_integer(token).filter(|value| *value > 0)
}

fn nonnegative_integer(token: Token) -> Option<i32> {
    match token {
        Token::Number(raw, _) => raw.parse::<i32>().ok().filter(|value| *value >= 0),
        _ => None,
    }
}
