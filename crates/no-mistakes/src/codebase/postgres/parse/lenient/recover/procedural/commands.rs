use super::{assignment, label, word};
use sqlparser::ast::{SetExpr, Statement};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Token;
mod loops;
pub(super) use loops::{for_header, foreach_header};

pub(super) fn matches(code: &[&Token]) -> bool {
    let Some(first) = code.first() else {
        return false;
    };
    if word(first, "GET") {
        return diagnostics(code);
    }
    if word(first, "OPEN") {
        return open(&code[1..]);
    }
    if word(first, "FETCH") {
        return fetch(&code[1..]);
    }
    if word(first, "MOVE") {
        return cursor_move(&code[1..]);
    }
    word(first, "CLOSE") && code.len() == 2 && label(code[1])
}

pub(super) fn query(code: &[&Token]) -> bool {
    matches!(parsed_statement(code), Some(Statement::Query(_)))
}

pub(super) fn row_query(code: &[&Token]) -> bool {
    match parsed_statement(code) {
        Some(Statement::Query(query)) => match query.body.as_ref() {
            SetExpr::Insert(statement)
            | SetExpr::Update(statement)
            | SetExpr::Delete(statement) => dml_returning(statement),
            _ => true,
        },
        Some(statement) => dml_returning(&statement),
        _ => false,
    }
}

fn dml_returning(statement: &Statement) -> bool {
    let returning = match statement {
        Statement::Insert(statement) => &statement.returning,
        Statement::Update(statement) => &statement.returning,
        Statement::Delete(statement) => &statement.returning,
        _ => return false,
    };
    returning.as_ref().is_some_and(|items| !items.is_empty())
}

fn parsed_statement(code: &[&Token]) -> Option<Statement> {
    if code.is_empty() {
        return None;
    }
    let tokens = code.iter().map(|token| (**token).clone()).collect();
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens(tokens);
    let statement = parser.parse_statement().ok()?;
    (parser.peek_token().token == Token::EOF).then_some(statement)
}

fn diagnostics(code: &[&Token]) -> bool {
    let mut at = 1;
    if code
        .get(at)
        .is_some_and(|token| word(token, "CURRENT") || word(token, "STACKED"))
    {
        at += 1;
    }
    if !code.get(at).is_some_and(|token| word(token, "DIAGNOSTICS")) {
        return false;
    }
    let parts = &code[at + 1..];
    !parts.is_empty()
        && assignment::comma_parts(parts).is_some_and(|parts| {
            parts.into_iter().all(|part| {
                assignment::matches(part)
                    && assignment::operator(part).is_some_and(|at| {
                        part.len() == at + 2 && matches!(part[at + 1], Token::Word(_))
                    })
            })
        })
}

fn open(code: &[&Token]) -> bool {
    let Some(name) = code.first() else {
        return false;
    };
    if !label(name) {
        return false;
    }
    if code.len() == 1 {
        return true;
    }
    let mut tail = &code[1..];
    if tail.len() >= 2
        && matches!(tail.first(), Some(Token::LParen))
        && matches!(tail.last(), Some(Token::RParen))
    {
        return tail.len() == 2
            || assignment::comma_parts(&tail[1..tail.len() - 1])
                .is_some_and(|parts| parts.into_iter().all(assignment::expression));
    }
    if tail.first().is_some_and(|token| word(token, "NO")) {
        if !tail.get(1).is_some_and(|token| word(token, "SCROLL")) {
            return false;
        }
        tail = &tail[1..];
    }
    if tail.first().is_some_and(|token| word(token, "SCROLL")) {
        tail = &tail[1..];
    }
    if tail.first().is_some_and(|token| word(token, "FOR")) {
        return query(&tail[1..]) || static_execute(&tail[1..]);
    }
    false
}

fn static_execute(code: &[&Token]) -> bool {
    if !code.first().is_some_and(|token| word(token, "EXECUTE"))
        || !matches!(
            code.get(1),
            Some(
                Token::SingleQuotedString(_)
                    | Token::DollarQuotedString(_)
                    | Token::EscapedStringLiteral(_)
                    | Token::UnicodeStringLiteral(_)
            )
        )
    {
        return false;
    }
    code.len() == 2
        || code.get(2).is_some_and(|token| word(token, "USING"))
            && assignment::comma_parts(&code[3..])
                .is_some_and(|parts| parts.into_iter().all(assignment::expression))
}

fn fetch(code: &[&Token]) -> bool {
    let Some(into) = code.iter().position(|token| word(token, "INTO")) else {
        return false;
    };
    let source = &code[..into];
    let target = &code[into + 1..];
    cursor_source(source)
        && assignment::comma_parts(target)
            .is_some_and(|parts| parts.into_iter().all(assignment::target))
}

fn cursor_move(code: &[&Token]) -> bool {
    cursor_source(code)
}

fn cursor_source(code: &[&Token]) -> bool {
    let Some(name) = code.last() else {
        return false;
    };
    if !label(name) {
        return false;
    }
    let mut direction = &code[..code.len() - 1];
    if direction
        .last()
        .is_some_and(|token| word(token, "FROM") || word(token, "IN"))
    {
        direction = &direction[..direction.len() - 1];
    }
    direction.is_empty()
        || direction.len() == 1
            && ["NEXT", "PRIOR", "FIRST", "LAST", "ALL"]
                .iter()
                .any(|name| word(direction[0], name))
        || direction.len() >= 2
            && ["ABSOLUTE", "RELATIVE", "FORWARD", "BACKWARD"]
                .iter()
                .any(|name| word(direction[0], name))
            && assignment::expression(&direction[1..])
}
