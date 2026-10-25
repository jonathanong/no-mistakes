//! Complete, conservative recognition for PostgreSQL utility DDL not covered by sqlparser.
use cursor::Cursor;
use sqlparser::tokenizer::{Token, TokenWithSpan};
mod commands;
mod cursor;
mod extended;
mod types;

pub(super) fn valid(tokens: &[TokenWithSpan]) -> bool {
    let words = tokens
        .iter()
        .map(|item| &item.token)
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .collect::<Vec<_>>();
    let mut c = Cursor {
        tokens: &words,
        at: 0,
    };
    let matched = if c.words(&["CREATE", "FOREIGN", "DATA", "WRAPPER"]) {
        commands::create_fdw(&mut c)
    } else if c.words(&["CREATE", "FOREIGN", "TABLE"]) {
        extended::create_foreign_table(&mut c)
    } else if c.words(&["ALTER", "FOREIGN", "DATA", "WRAPPER"]) {
        commands::alter_fdw(&mut c)
    } else if c.words(&["ALTER", "FOREIGN", "TABLE"]) {
        extended::alter_foreign_table(&mut c)
    } else if c.words(&["ALTER", "SERVER"]) {
        extended::alter_server(&mut c)
    } else if c.words(&["ALTER", "USER", "MAPPING"]) {
        extended::alter_mapping(&mut c)
    } else if c.words(&["CREATE", "USER", "MAPPING"]) {
        commands::create_mapping(&mut c)
    } else if c.words(&["IMPORT", "FOREIGN", "SCHEMA"]) {
        commands::import_schema(&mut c)
    } else if c.words(&["DROP", "OWNED", "BY"]) {
        c.roles() && (c.word("CASCADE") || c.word("RESTRICT") || c.at == c.tokens.len())
    } else if c.words(&["REASSIGN", "OWNED", "BY"]) {
        c.roles() && c.word("TO") && c.role()
    } else if c.words(&["DROP", "USER", "MAPPING"]) {
        extended::drop_mapping(&mut c)
    } else if c.words(&["DROP", "FOREIGN", "DATA", "WRAPPER"]) {
        extended::drop_names(&mut c)
    } else if c.words(&["DROP", "FOREIGN", "TABLE"]) || c.words(&["DROP", "SERVER"]) {
        extended::drop_names(&mut c)
    } else if c.words(&["SECURITY", "LABEL"]) {
        commands::security_label(&mut c)
    } else if c.words(&["ALTER", "LARGE", "OBJECT"]) {
        c.number() && c.words(&["OWNER", "TO"]) && c.role()
    } else {
        false
    };
    matched && c.at == c.tokens.len()
}

#[cfg(test)]
#[path = "ddl_foreign_tests.rs"]
mod tests;
