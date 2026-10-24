use super::{assignment, label, word};
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Token;

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
    if code.is_empty() {
        return false;
    }
    let tokens = code.iter().map(|token| (**token).clone()).collect();
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens(tokens);
    matches!(parser.parse_statement(), Ok(Statement::Query(_)))
        && parser.peek_token().token == Token::EOF
}

pub(super) fn for_header(code: &[&Token]) -> bool {
    let Some(into) = code.iter().position(|token| word(token, "IN")) else {
        return false;
    };
    if into < 2 || !matches!(code[1], Token::Word(_)) {
        return false;
    }
    let mut source = &code[into + 1..];
    if source.first().is_some_and(|token| word(token, "REVERSE")) {
        source = &source[1..];
    }
    if query(source) {
        return true;
    }
    // sqlparser lexes the compact PL/pgSQL range `1..0` as `1.` and `.0`.
    if let [Token::Number(left, _), Token::Number(right, _)] = source {
        if left.ends_with('.') && right.starts_with('.') {
            return true;
        }
    }
    let Some(range) = source
        .windows(2)
        .position(|pair| *pair[0] == Token::Period && *pair[1] == Token::Period)
    else {
        return false;
    };
    assignment::expression(&source[..range]) && assignment::expression(&source[range + 2..])
}

pub(super) fn foreach_header(code: &[&Token]) -> bool {
    code.len() >= 5
        && matches!(code[1], Token::Word(_))
        && word(code[2], "IN")
        && word(code[3], "ARRAY")
        && assignment::expression(&code[4..])
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
        && parts
            .split(|token| matches!(token, Token::Comma))
            .all(|part| {
                assignment::matches(part)
                    && assignment::operator(part).is_some_and(|at| {
                        part.len() == at + 2 && matches!(part[at + 1], Token::Word(_))
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
    let tail = &code[1..];
    if tail.len() >= 2
        && matches!(tail.first(), Some(Token::LParen))
        && matches!(tail.last(), Some(Token::RParen))
    {
        return tail.len() == 2
            || tail[1..tail.len() - 1]
                .split(|token| matches!(token, Token::Comma))
                .all(assignment::expression);
    }
    if tail.first().is_some_and(|token| word(token, "FOR")) {
        return query(&tail[1..]) || static_execute(&tail[1..]);
    }
    false
}

fn static_execute(code: &[&Token]) -> bool {
    code.len() == 2
        && code.first().is_some_and(|token| word(token, "EXECUTE"))
        && matches!(
            code.get(1),
            Some(Token::SingleQuotedString(_) | Token::DollarQuotedString(_))
        )
}

fn fetch(code: &[&Token]) -> bool {
    let Some(into) = code.iter().position(|token| word(token, "INTO")) else {
        return false;
    };
    let source = &code[..into];
    let target = &code[into + 1..];
    cursor_source(source) && assignment::target(target)
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
        || direction.len() == 2
            && ["ABSOLUTE", "RELATIVE", "FORWARD", "BACKWARD"]
                .iter()
                .any(|name| word(direction[0], name))
            && assignment::expression(&direction[1..])
}
