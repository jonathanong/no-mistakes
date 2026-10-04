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
            parenthesized_bound(parser)
                && keyword_of(&parser.next_token().token) == Some(Keyword::TO)
                && parenthesized_bound(parser)
        }
        Some(Keyword::IN | Keyword::WITH) => parenthesized_bound(parser),
        _ => false,
    };
    valid && matches!(parser.peek_token().token, Token::EOF)
}

fn parenthesized_bound(parser: &mut Parser) -> bool {
    if parser.next_token().token != Token::LParen {
        return false;
    }
    let mut depth = 1;
    let mut content = false;
    loop {
        match parser.next_token().token {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    return content;
                }
            }
            Token::EOF => return false,
            _ => content = true,
        }
    }
}
