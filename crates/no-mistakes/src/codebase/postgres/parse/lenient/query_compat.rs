//! Narrow PostgreSQL DML target compatibility for sqlparser 0.63.
//! Only target subscripts and field selectors are hidden from a parser clone;
//! the original tokens remain attached to the occurrence for diagnostics.
mod targets;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan, Whitespace};
use targets::{assignment_targets, insert_targets};

pub(super) fn rewrite(tokens: &mut [TokenWithSpan]) {
    let code: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| !matches!(token.token, Token::Whitespace(_)))
        .map(|(at, _)| at)
        .collect();
    if code.is_empty() {
        return;
    }
    let insert = word(&tokens[code[0]].token, "INSERT");
    let update = word(&tokens[code[0]].token, "UPDATE");
    if !insert && !update {
        return;
    }
    let original = tokens.to_vec();
    if insert {
        for position in 2..code.len().saturating_sub(3) {
            if word(&tokens[code[position]].token, "OVERRIDING")
                && (word(&tokens[code[position + 1]].token, "USER")
                    || word(&tokens[code[position + 1]].token, "SYSTEM"))
                && word(&tokens[code[position + 2]].token, "VALUE")
                && word(&tokens[code[position + 3]].token, "VALUES")
            {
                for at in &code[position..position + 3] {
                    tokens[*at].token = Token::Whitespace(Whitespace::Space);
                }
                break;
            }
        }
    }
    let insert_targets = if insert {
        insert_targets(tokens, &code)
    } else {
        Vec::new()
    };
    let assignment_targets = assignment_targets(tokens, &code);
    let mut clear = Vec::new();
    for (pos, at) in code.iter().copied().enumerate() {
        if tokens[at].token != Token::LBracket || pos == 0 {
            continue;
        }
        let in_insert_columns = insert_targets.contains(&(pos - 1));
        let in_assignment = assignment_targets.contains(&(pos - 1));
        if !in_insert_columns && !in_assignment {
            continue;
        }
        let Some(end) = bracket_end(tokens, &code, pos) else {
            continue;
        };
        let Some(Token::Word(_)) = code.get(pos - 1).map(|at| &tokens[*at].token) else {
            continue;
        };
        if !valid_subscript(tokens, &code, pos - 1, end) {
            continue;
        }
        let mut tail = end + 1;
        if code
            .get(tail)
            .is_some_and(|at| tokens[*at].token == Token::Period)
            && code
                .get(tail + 1)
                .is_some_and(|at| matches!(tokens[*at].token, Token::Word(_)))
        {
            tail += 2;
        }
        if in_assignment
            && !code
                .get(tail)
                .is_some_and(|at| tokens[*at].token == Token::Eq)
        {
            continue;
        }
        if in_insert_columns
            && !code
                .get(tail)
                .is_some_and(|at| matches!(tokens[*at].token, Token::Comma | Token::RParen))
        {
            continue;
        }
        clear.extend(code[pos..tail].iter().copied());
    }
    for at in clear {
        tokens[at].token = Token::Whitespace(Whitespace::Space);
    }
    if tokens != original.as_slice() {
        let mut parser =
            Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens.to_vec());
        if !(parser
            .parse_statement()
            .is_ok_and(|statement| super::pg_ast::valid(&statement))
            && parser.peek_token().token == Token::EOF)
        {
            tokens.clone_from_slice(&original);
        }
    }
}

fn bracket_end(tokens: &[TokenWithSpan], code: &[usize], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (pos, at) in code.iter().copied().enumerate().skip(open) {
        match tokens[at].token {
            Token::LBracket => depth += 1,
            Token::RBracket => {
                depth -= 1;
                if depth == 0 {
                    return Some(pos);
                }
            }
            _ => {}
        }
    }
    None
}

fn valid_subscript(tokens: &[TokenWithSpan], code: &[usize], name: usize, close: usize) -> bool {
    if close <= name + 2
        || close != name + 3
            && code[name + 2..close]
                .iter()
                .all(|at| tokens[*at].token == Token::Colon)
    {
        return false;
    }
    let expression = code[name..=close]
        .iter()
        .map(|at| tokens[*at].token.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    Parser::parse_sql(&PostgreSqlDialect {}, &format!("SELECT {expression}"))
        .is_ok_and(|statements| statements.len() == 1 && super::pg_ast::valid(&statements[0]))
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none()
        && word.value.eq_ignore_ascii_case(expected))
}

#[cfg(test)]
mod tests;
