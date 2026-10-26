//! Parser-clone validation for CREATE expressions/options not in sqlparser 0.63.
use crate::codebase::postgres::parse::lenient::pg_ident;
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};
#[path = "native/index.rs"]
mod index;
#[path = "native/table.rs"]
mod table;

pub(crate) fn valid(original: &[TokenWithSpan]) -> bool {
    let mut tokens: Vec<_> = original
        .iter()
        .map(|item| item.token.clone())
        .filter(|token| !matches!(token, Token::Whitespace(_) | Token::SemiColon))
        .collect();
    if tokens.first().is_none_or(|token| !word(token, "CREATE")) {
        return false;
    }
    let kind = if tokens.iter().take(4).any(|token| word(token, "INDEX")) {
        Kind::Index
    } else if tokens.get(1).is_some_and(|token| word(token, "POLICY")) {
        Kind::Policy
    } else if tokens.get(1).is_some_and(|token| word(token, "VIEW")) {
        Kind::View
    } else if tokens.get(1).is_some_and(|token| word(token, "FUNCTION")) {
        Kind::Function
    } else if tokens.get(1).is_some_and(|token| word(token, "TABLE")) {
        Kind::Table
    } else {
        return false;
    };
    if !valid_head(&tokens, kind) {
        return false;
    }
    let changed = match kind {
        Kind::Index => collate_cast(&mut tokens) || index::options(&mut tokens),
        Kind::Policy => collate_cast(&mut tokens),
        Kind::View => view_options(&mut tokens),
        Kind::Function => out_arguments(&mut tokens),
        Kind::Table => table::rewrite(&mut tokens),
    };
    changed && parsed(&tokens, kind)
}

#[derive(Clone, Copy)]
enum Kind {
    Index,
    Policy,
    View,
    Function,
    Table,
}

fn valid_head(tokens: &[Token], kind: Kind) -> bool {
    let name = match kind {
        Kind::Table => return pg_ident::table_head(tokens).is_some(),
        Kind::Index => tokens
            .iter()
            .take(4)
            .position(|token| word(token, "INDEX"))
            .map(|at| at + 1),
        Kind::Policy | Kind::View | Kind::Function => Some(2),
    };
    name.and_then(|at| tokens.get(at))
        .is_some_and(pg_ident::col_id)
}

fn parsed(tokens: &[Token], kind: Kind) -> bool {
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens(tokens.to_vec());
    let ast = parser.parse_statement();
    let valid_names = ast
        .as_ref()
        .is_ok_and(crate::codebase::postgres::parse::lenient::pg_ast::valid);
    let right = matches!(
        (kind, ast),
        (Kind::Index, Ok(Statement::CreateIndex(_)))
            | (Kind::Policy, Ok(Statement::CreatePolicy(_)))
            | (Kind::View, Ok(Statement::CreateView(_)))
            | (Kind::Function, Ok(Statement::CreateFunction(_)))
            | (Kind::Table, Ok(Statement::CreateTable(_)))
    );
    right && valid_names && parser.peek_token().token == Token::EOF
}

fn out_arguments(tokens: &mut [Token]) -> bool {
    let Some(open) = tokens.iter().position(|token| *token == Token::LParen) else {
        return false;
    };
    let Some(close) = tokens
        .iter()
        .enumerate()
        .skip(open + 1)
        .find_map(|(at, token)| (*token == Token::RParen).then_some(at))
    else {
        return false;
    };
    let mut changed = false;
    for at in open + 1..close.saturating_sub(2) {
        if (tokens[at - 1] == Token::LParen || tokens[at - 1] == Token::Comma)
            && pg_ident::col_id(&tokens[at])
            && word(&tokens[at + 1], "OUT")
            && matches!(tokens[at + 2], Token::Word(_))
            && (tokens[at + 3] == Token::Comma || tokens[at + 3] == Token::RParen)
        {
            tokens.swap(at, at + 1);
            changed = true;
        }
    }
    changed
}

fn collate_cast(tokens: &mut Vec<Token>) -> bool {
    for at in 3..tokens.len().saturating_sub(1) {
        if word(&tokens[at], "COLLATE")
            && pg_ident::col_id(&tokens[at + 1])
            && matches!(tokens[at - 1], Token::Word(_))
            && tokens[at - 2] == Token::DoubleColon
        {
            tokens.drain(at..at + 2);
            return true;
        }
    }
    false
}

fn view_options(tokens: &mut Vec<Token>) -> bool {
    let Some(with) = tokens.windows(4).position(|part| {
        word(&part[0], "WITH")
            && part[1] == Token::LParen
            && word(&part[2], "SECURITY_BARRIER")
            && part[3] == Token::RParen
    }) else {
        return false;
    };
    let end = tokens.len();
    if end < with + 8
        || !word(&tokens[end - 3], "WITH")
        || !word(&tokens[end - 2], "CHECK")
        || !word(&tokens[end - 1], "OPTION")
    {
        return false;
    }
    tokens.drain(end - 3..end);
    tokens.splice(with + 3..with + 3, [Token::Eq, Token::make_keyword("TRUE")]);
    true
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
