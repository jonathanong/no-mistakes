use super::{assignment, commands, word};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Token;

pub(super) fn matches(code: &[&Token]) -> bool {
    if code.len() < 2 || !matches!(code[0], Token::Word(_)) {
        return false;
    }
    let tail = &code[1..];
    if tail.first().is_some_and(|token| word(token, "ALIAS")) {
        return tail.len() == 3 && word(tail[1], "FOR") && matches!(tail[2], Token::Placeholder(_));
    }
    if let Some(cursor) = tail.iter().position(|token| word(token, "CURSOR")) {
        return cursor(tail, cursor);
    }
    let default = tail
        .iter()
        .position(|token| word(token, "DEFAULT") || matches!(token, Token::Assignment | Token::Eq));
    let (datatype, initializer) = match default {
        Some(at) => (&tail[..at], Some(&tail[at + 1..])),
        None => (tail, None),
    };
    type_shape(datatype) && initializer.is_none_or(assignment::expression)
}

fn cursor(tail: &[&Token], cursor: usize) -> bool {
    let prefix = &tail[..cursor];
    if !prefix.is_empty()
        && !(prefix.len() == 1 && word(prefix[0], "SCROLL"))
        && !(prefix.len() == 2 && word(prefix[0], "NO") && word(prefix[1], "SCROLL"))
    {
        return false;
    }
    let Some(start) = tail
        .iter()
        .position(|token| word(token, "FOR") || word(token, "IS"))
    else {
        return false;
    };
    let args = &tail[cursor + 1..start];
    let args_valid = args.is_empty()
        || args.len() >= 3
            && matches!(args.first(), Some(Token::LParen))
            && matches!(args.last(), Some(Token::RParen))
            && assignment::comma_parts(&args[1..args.len() - 1])
                .is_some_and(|parts| parts.into_iter().all(matches));
    args_valid && commands::query(&tail[start + 1..])
}

fn type_shape(code: &[&Token]) -> bool {
    let mut code = code;
    if code.first().is_some_and(|token| word(token, "CONSTANT")) {
        code = &code[1..];
    }
    if code.len() >= 2 && word(code[code.len() - 2], "NOT") && word(code[code.len() - 1], "NULL") {
        code = &code[..code.len() - 2];
    }
    if let Some(collate) = code.iter().position(|token| word(token, "COLLATE")) {
        let name = &code[collate + 1..];
        if name.is_empty()
            || !name
                .split(|token| matches!(token, Token::Period))
                .all(|part| part.len() == 1 && matches!(part[0], Token::Word(_)))
        {
            return false;
        }
        code = &code[..collate];
    }
    if code.is_empty() {
        return false;
    }
    if code.len() >= 3
        && matches!(code[code.len() - 2], Token::Mod)
        && (word(code[code.len() - 1], "TYPE") || word(code[code.len() - 1], "ROWTYPE"))
    {
        return code[..code.len() - 2]
            .split(|token| matches!(token, Token::Period))
            .all(|part| part.len() == 1 && matches!(part[0], Token::Word(_)));
    }
    let tokens = code.iter().map(|token| (**token).clone()).collect();
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens(tokens);
    parser.parse_data_type().is_ok() && parser.peek_token().token == Token::EOF
}
