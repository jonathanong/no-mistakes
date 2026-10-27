use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Token;

pub(super) fn matches(code: &[&Token]) -> bool {
    let Some(at) = operator(code) else {
        return false;
    };
    target(&code[..at]) && expression(&code[at + 1..])
}

pub(in super::super) fn target(code: &[&Token]) -> bool {
    parsed(code).is_some_and(|target| lvalue(&target))
}

pub(super) fn expression(code: &[&Token]) -> bool {
    parsed(code).is_some()
}

fn parsed(code: &[&Token]) -> Option<Expr> {
    (!code.is_empty()).then_some(())?;
    // A procedural range delimiter cannot occur inside an SQL expression.
    (!code.iter().any(|token| super::ranges::marker(token))).then_some(())?;
    let tokens = code.iter().map(|token| (**token).clone()).collect();
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens(tokens);
    let expression = parser.parse_expr().ok()?;
    (parser.peek_token().token == Token::EOF).then_some(expression)
}

fn lvalue(expr: &Expr) -> bool {
    match expr {
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) => true,
        Expr::CompoundFieldAccess { root, .. } => lvalue(root),
        _ => false,
    }
}

pub(super) fn operator(code: &[&Token]) -> Option<usize> {
    let mut nested = 0usize;
    for (at, token) in code.iter().enumerate() {
        match token {
            Token::LParen | Token::LBracket => nested += 1,
            Token::RParen | Token::RBracket => nested = nested.saturating_sub(1),
            Token::Assignment | Token::Eq if nested == 0 => return Some(at),
            _ => {}
        }
    }
    None
}

pub(in super::super) fn comma_parts<'a>(code: &'a [&'a Token]) -> Option<Vec<&'a [&'a Token]>> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    for (at, token) in code.iter().enumerate() {
        match token {
            Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket => depth = depth.checked_sub(1)?,
            Token::Comma if depth == 0 => {
                parts.push(&code[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    parts.push(&code[start..]);
    Some(parts)
}
use sqlparser::ast::Expr;
