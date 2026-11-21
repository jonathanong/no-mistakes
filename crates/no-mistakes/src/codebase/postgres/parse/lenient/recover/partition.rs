use super::super::keyword_of;
use super::super::LocatedStatement;
use sqlparser::ast::{AlterTableOperation, Expr, Partition, Statement};
use sqlparser::dialect::GenericDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};

// sqlparser 0.64 parses PostgreSQL ATTACH/DETACH PARTITION. A concurrent detach
// inside a transaction stays untrusted. Generic recovery covers the rest.
pub(super) fn parsed(statement: Statement, allow_concurrent_detach: bool) -> Vec<LocatedStatement> {
    if !super::partition_sound::acceptable(&statement, allow_concurrent_detach) {
        return Vec::new();
    }
    let mut located = LocatedStatement::plain(statement);
    located.complete = true;
    located.partition_complete = named_partition(&located.statement);
    vec![located]
}

fn named_partition(statement: &Statement) -> bool {
    let Statement::AlterTable(table) = statement else {
        return false;
    };
    table.operations.iter().any(|operation| {
        matches!(
            operation,
            AlterTableOperation::AttachPartitionOf { .. }
                | AlterTableOperation::DetachPartitionOf { .. }
        )
    })
}
pub(super) fn recover_partition_change(
    chunk: &[Token],
    original: Option<&[TokenWithSpan]>,
    allow_concurrent_detach: bool,
) -> Option<(Statement, bool)> {
    let dialect = GenericDialect {};
    let mut parser = match original {
        Some(tokens) => {
            let mut tokens = tokens.to_vec();
            crate::codebase::postgres::source::conceal_body_keyword(&mut tokens);
            Parser::new(&dialect).with_tokens_with_locations(tokens)
        }
        None => {
            let mut tokens: Vec<TokenWithSpan> =
                chunk.iter().cloned().map(TokenWithSpan::wrap).collect();
            crate::codebase::postgres::source::conceal_body_keyword(&mut tokens);
            Parser::new(&dialect).with_tokens_with_locations(tokens)
        }
    };
    let statement = parser.parse_statement().ok()?;
    let Statement::AlterTable(table) = &statement else {
        return None;
    };
    let [operation] = table.operations.as_slice() else {
        return None;
    };
    let partition = match operation {
        AlterTableOperation::AttachPartition { partition } => {
            if !partition_bound_suffix(&mut parser) {
                return None;
            }
            partition
        }
        AlterTableOperation::DetachPartition { partition } => {
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
            partition
        }
        _ => return None,
    };
    let named = matches!(
        partition,
        Partition::Expr(Expr::Identifier(_) | Expr::CompoundIdentifier(_))
    );
    Some((statement, named))
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
