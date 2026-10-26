use super::SUPPORTED_KINDS;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn new_kind(code: &[&TokenWithSpan]) -> Option<&'static str> {
    let first = *code.first()?;
    let mut object = 1;
    if word(first, "CREATE")
        && code.get(1).is_some_and(|token| word(token, "OR"))
        && code.get(2).is_some_and(|token| word(token, "REPLACE"))
    {
        object = 3;
    }
    if word(first, "CREATE") {
        if code.get(object).is_some_and(|token| {
            ["TEMP", "TEMPORARY", "UNLOGGED", "UNIQUE", "MATERIALIZED"]
                .iter()
                .any(|modifier| word(token, modifier))
        }) {
            object += 1;
        }
    } else if word(first, "ALTER")
        && code
            .get(object)
            .is_some_and(|token| word(token, "MATERIALIZED"))
    {
        object += 1;
    }
    if code
        .get(object)
        .is_some_and(|token| word(token, "CONSTRAINT"))
    {
        object += 1;
    }
    // A bare pair of keywords is not a statement with an object/operation.
    code.get(object + 1)?;
    let name = *code.get(object)?;
    let action = if word(first, "CREATE") {
        "CREATE"
    } else if word(first, "DROP") {
        "DROP"
    } else if word(first, "ALTER") {
        "ALTER"
    } else {
        return None;
    };
    let category = [
        "DATABASE",
        "SYSTEM",
        "SCHEMA",
        "TRIGGER",
        "FUNCTION",
        "PROCEDURE",
        "TABLE",
        "VIEW",
        "INDEX",
        "TYPE",
    ]
    .into_iter()
    .find(|kind| word(name, kind))?;
    if action == "CREATE" && object > 1 && !valid_modifier(code[object - 1], category) {
        return None;
    }
    if action == "ALTER"
        && ["VIEW", "TRIGGER", "FUNCTION", "PROCEDURE", "INDEX", "TYPE"].contains(&category)
        && !code[object + 2..].iter().any(|token| {
            [
                "RENAME",
                "OWNER",
                "SET",
                "RESET",
                "ADD",
                "DROP",
                "ALTER",
                "ENABLE",
                "DISABLE",
                "ATTACH",
                "DETACH",
                "DEPENDS",
                "NO",
                "CALLED",
                "RETURNS",
                "IMMUTABLE",
                "STABLE",
                "VOLATILE",
                "STRICT",
                "PARALLEL",
                "COST",
                "ROWS",
                "SECURITY",
                "LEAKPROOF",
                "SUPPORT",
            ]
            .iter()
            .any(|action| word(token, action))
        })
    {
        return None;
    }
    let kind = SUPPORTED_KINDS
        .iter()
        .copied()
        .find(|kind| kind.starts_with(action) && kind.ends_with(category))?;
    // Original categories remain owned by the established AST path.
    (!matches!(
        kind,
        "CREATE TABLE"
            | "ALTER TABLE"
            | "CREATE INDEX"
            | "CREATE VIEW"
            | "DROP INDEX"
            | "DROP VIEW"
    ))
    .then_some(kind)
}

fn valid_modifier(modifier: &TokenWithSpan, category: &str) -> bool {
    (word(modifier, "TEMP") || word(modifier, "TEMPORARY")) && ["TABLE", "VIEW"].contains(&category)
        || word(modifier, "UNLOGGED") && category == "TABLE"
        || word(modifier, "UNIQUE") && category == "INDEX"
        || word(modifier, "MATERIALIZED") && category == "VIEW"
        || word(modifier, "CONSTRAINT") && category == "TRIGGER"
        || word(modifier, "REPLACE")
}

pub(super) fn word(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
