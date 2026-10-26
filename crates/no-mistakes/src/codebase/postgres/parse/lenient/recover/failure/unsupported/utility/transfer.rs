use super::{boolean, columns, expression, string, token, Cursor, Token};
use sqlparser::{dialect::PostgreSqlDialect, parser::Parser, tokenizer::TokenWithSpan};
#[path = "transfer/symmetric.rs"]
mod symmetric;

pub(super) fn valid(cursor: &mut Cursor<'_>) -> bool {
    if cursor.word("EXPLAIN") {
        return explain(cursor);
    }
    if !cursor.word("COPY") {
        return false;
    }
    if token(cursor, Token::LParen) {
        let Some(end) = closing(cursor) else {
            return false;
        };
        if !query(&cursor.tokens[cursor.at..end]) {
            return false;
        }
        cursor.at = end + 1;
    } else {
        if !cursor.name() {
            return false;
        }
        if matches!(cursor.tokens.get(cursor.at), Some(Token::LParen)) && !columns(cursor) {
            return false;
        }
    }
    if !(cursor.word("FROM") || cursor.word("TO")) {
        return false;
    }
    if cursor.word("PROGRAM") {
        if !string(cursor) {
            return false;
        }
    } else if !(cursor.word("STDIN") || cursor.word("STDOUT") || string(cursor)) {
        return false;
    }
    let with = cursor.word("WITH");
    if token(cursor, Token::LParen) {
        loop {
            if !option(cursor) {
                return false;
            }
            if !cursor.comma() {
                break;
            }
        }
        if !token(cursor, Token::RParen) {
            return false;
        }
    } else if with {
        return false;
    }
    if cursor.word("WHERE") {
        if !expression(&cursor.tokens[cursor.at..]) {
            return false;
        }
        cursor.at = cursor.tokens.len();
    }
    true
}

fn closing(cursor: &Cursor<'_>) -> Option<usize> {
    let mut depth = 1;
    for index in cursor.at..cursor.tokens.len() {
        match cursor.tokens[index] {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn query(tokens: &[&Token]) -> bool {
    let mut parser = Parser::new(&PostgreSqlDialect {})
        .with_tokens(tokens.iter().map(|t| (*t).clone()).collect());
    matches!(parser.parse_statement(), Ok(statement @ sqlparser::ast::Statement::Query(_))
        if super::super::query::valid_ast(&statement))
        && parser.peek_token().token == Token::EOF
}

fn option(cursor: &mut Cursor<'_>) -> bool {
    if cursor.word("FORMAT") {
        return ["TEXT", "CSV", "BINARY"]
            .iter()
            .any(|word| cursor.word(word));
    }
    if ["FORCE_NOT_NULL", "FORCE_NULL", "FORCE_QUOTE"]
        .iter()
        .any(|word| cursor.word(word))
    {
        return token(cursor, Token::Mul) || columns(cursor);
    }
    if [
        "DELIMITER",
        "NULL",
        "QUOTE",
        "ESCAPE",
        "ENCODING",
        "DEFAULT",
    ]
    .iter()
    .any(|word| cursor.word(word))
    {
        return string(cursor);
    }
    if cursor.word("HEADER") {
        if !cursor.word("MATCH") {
            boolean(cursor);
        }
        return true;
    }
    if cursor.word("FREEZE") {
        boolean(cursor);
        return true;
    }
    false
}

fn explain(cursor: &mut Cursor<'_>) -> bool {
    if !token(cursor, Token::LParen) {
        return false;
    }
    loop {
        if ![
            "ANALYZE",
            "VERBOSE",
            "COSTS",
            "SETTINGS",
            "GENERIC_PLAN",
            "BUFFERS",
            "WAL",
            "TIMING",
            "SUMMARY",
        ]
        .iter()
        .any(|word| cursor.word(word))
        {
            return false;
        }
        boolean(cursor);
        if !cursor.comma() {
            break;
        }
    }
    if !token(cursor, Token::RParen) {
        return false;
    }
    let mut tokens: Vec<_> = cursor.tokens[cursor.at..]
        .iter()
        .map(|token| TokenWithSpan::wrap((*token).clone()))
        .collect();
    symmetric::erase_on_parser_clone(&mut tokens);
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens.clone());
    let statement = parser.parse_statement().ok();
    let parsed = match statement.as_ref() {
        Some(
            sqlparser::ast::Statement::Query(_)
            | sqlparser::ast::Statement::Insert(_)
            | sqlparser::ast::Statement::Update(_)
            | sqlparser::ast::Statement::Delete(_)
            | sqlparser::ast::Statement::Merge(_)
            | sqlparser::ast::Statement::Execute { .. },
        ) => true,
        Some(sqlparser::ast::Statement::CreateTable(table)) => table.query.is_some(),
        _ => false,
    } && parser.peek_token().token == Token::EOF
        && statement
            .as_ref()
            .is_some_and(super::super::query::valid_ast);
    // The missing CTAS column-name-only form is validated by the DDL recognizer.
    let mut head = Cursor {
        tokens: cursor.tokens,
        at: cursor.at,
    };
    let create = head.word("CREATE");
    ["TEMP", "TEMPORARY", "UNLOGGED"]
        .iter()
        .any(|word| head.word(word));
    let ctas = create
        && head.word("TABLE")
        && tokens.iter().any(|token| {
            matches!(&token.token, Token::Word(word)
        if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("AS"))
        })
        && super::super::super::ddl_metadata::valid(&tokens);
    if !(parsed || ctas) {
        return false;
    }
    cursor.at = cursor.tokens.len();
    true
}
