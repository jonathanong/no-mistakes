//! Statements the narrow PostgreSQL grammar still owns.
//! sqlparser 0.64 parses them, but that parse is not a schema fact.
use super::keywords;
use sqlparser::ast::{
    visit_expressions, Expr, Ident, ObjectName, ObjectNamePart, ObjectType, RoleKeyword, Statement,
};
use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::ops::ControlFlow;

pub(super) fn reject(statement: &Statement, original: Option<&[TokenWithSpan]>) -> bool {
    grammar(statement)
        || reserved_name(statement)
        || bare_view_check(statement, original)
        || reserved_expression(statement)
}

fn grammar(statement: &Statement) -> bool {
    match statement {
        Statement::CreateAggregate(_)
        | Statement::CreateRule(_)
        | Statement::CreateExtension(_)
        | Statement::CreateUser(_) => true,
        Statement::CreateRole(role) => role.keyword == RoleKeyword::Group,
        Statement::Drop {
            object_type: ObjectType::Group,
            ..
        } => true,
        _ => false,
    }
}

fn reserved_name(statement: &Statement) -> bool {
    match statement {
        Statement::CreateIndex(index) => index.name.as_ref().is_some_and(object_reserved),
        Statement::CreatePolicy(policy) => ident_reserved(&policy.name),
        Statement::CreateView(view) => object_reserved(&view.name),
        Statement::CreateFunction(function) => object_reserved(&function.name),
        _ => false,
    }
}

/// `WITH CHECK` without `OPTION` is not a view option. sqlparser stops before it
/// and still returns the view.
fn bare_view_check(statement: &Statement, original: Option<&[TokenWithSpan]>) -> bool {
    if !matches!(statement, Statement::CreateView(_)) {
        return false;
    }
    let Some(tokens) = original else {
        return false;
    };
    let words: Vec<&Token> = tokens
        .iter()
        .map(|token| &token.token)
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .collect();
    words.iter().enumerate().any(|(at, token)| {
        let cascaded = word(token, "WITH")
            && words
                .get(at + 1)
                .is_some_and(|next| word(next, "CASCADED") || word(next, "LOCAL"))
            && words.get(at + 2).is_some_and(|next| word(next, "CHECK"))
            && !words.get(at + 3).is_some_and(|next| word(next, "OPTION"));
        let bare = word(token, "WITH")
            && words.get(at + 1).is_some_and(|next| word(next, "CHECK"))
            && !words
                .get(at + 2)
                .is_some_and(|next| word(next, "OPTION") || matches!(next, Token::LParen));
        cascaded || bare
    })
}

fn reserved_expression(statement: &Statement) -> bool {
    let mut reserved = false;
    let _ = visit_expressions(statement, |expr| {
        if reserved_expr(expr) {
            reserved = true;
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    reserved
}

fn reserved_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Collate { collation, .. } => object_reserved(collation),
        Expr::Named { name, .. } => ident_reserved(name),
        _ => false,
    }
}

fn object_reserved(name: &ObjectName) -> bool {
    name.0
        .iter()
        .any(|part| matches!(part, ObjectNamePart::Identifier(ident) if ident_reserved(ident)))
}

fn ident_reserved(ident: &Ident) -> bool {
    ident.quote_style.is_none() && keywords::reserved(&ident.value)
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
